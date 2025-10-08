use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use discv5::libp2p_identity::PeerId;
use libp2p::{
    Multiaddr, allow_block_list,
    core::{Endpoint, transport::PortUse},
    swarm::{ConnectionDenied, ConnectionId, FromSwarm, NetworkBehaviour},
};
use tracing::debug;

use crate::{
    metrics,
    scoring::peer_score_config::RETAIN_SCORE_EPOCH_MULTIPLIER,
};

/// Reasons why a peer might be blocked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockReason {
    /// Peer was blocked due to having a low gossipsub score
    LowScore,
    /// Peer was blocked due to failed handshake
    FailedHandshake,
    /// Peer was blocked due to outgoing connection errors (dial failures, timeouts, etc.)
    OutgoingConnectionError,
    /// Peer was blocked due to incoming connection errors
    IncomingConnectionError,
    /// Peer was blocked for other reasons
    Other,
}

/// Information about a blocked peer
#[derive(Debug, Clone)]
struct BlockedPeerInfo {
    /// When the peer was blocked
    blocked_at: tokio::time::Instant,
    /// Why the peer was blocked
    reason: BlockReason,
}

/// Manages peer blocking functionality
pub struct BlockingManager {
    /// Block list behaviour for actual connection denial
    block_list: allow_block_list::Behaviour<allow_block_list::BlockedPeers>,
    /// Tracking when peers were blocked and why for automatic unblocking
    blocked_peers_info: HashMap<PeerId, BlockedPeerInfo>,
    /// One epoch duration for calculating retain_score timeout
    one_epoch_duration: Duration,
}

impl BlockingManager {
    pub fn new(one_epoch_duration: Duration) -> Self {
        Self {
            block_list: allow_block_list::Behaviour::<allow_block_list::BlockedPeers>::default(),
            blocked_peers_info: HashMap::new(),
            one_epoch_duration,
        }
    }

    /// Block a peer with a reason and track timestamp for automatic unblocking
    pub fn block_peer(&mut self, peer_id: PeerId, reason: BlockReason) -> bool {
        if self.block_list.block_peer(peer_id) {
            self.blocked_peers_info.insert(
                peer_id,
                BlockedPeerInfo {
                    blocked_at: tokio::time::Instant::now(),
                    reason,
                },
            );
            debug!(?peer_id, ?reason, "Blocked peer");
            true
        } else {
            false
        }
    }

    /// Unblock a peer and remove from tracking
    pub fn unblock_peer(&mut self, peer_id: PeerId) -> bool {
        let was_removed = self.block_list.unblock_peer(peer_id);
        if was_removed {
            self.blocked_peers_info.remove(&peer_id);
            debug!(?peer_id, "Unblocked peer after retain_score duration");
        }
        was_removed
    }

    /// Get list of currently blocked peers
    pub fn blocked_peers(&self) -> &HashSet<PeerId> {
        self.block_list.blocked_peers()
    }

    /// Check and unblock peers that have been blocked long enough
    pub fn check_and_unblock_expired_peers(&mut self) {
        let retain_score_duration = self.one_epoch_duration * RETAIN_SCORE_EPOCH_MULTIPLIER;
        let now = tokio::time::Instant::now();

        let peers_to_unblock: Vec<PeerId> = self
            .blocked_peers_info
            .iter()
            .filter_map(|(&peer_id, info)| {
                if now.duration_since(info.blocked_at) >= retain_score_duration {
                    Some(peer_id)
                } else {
                    None
                }
            })
            .collect();

        for peer_id in peers_to_unblock {
            self.unblock_peer(peer_id);
        }
    }

    pub fn blocked_peers_count(&self) -> usize {
        self.blocked_peers_info.len()
    }

    /// Get the count of blocked peers grouped by reason
    pub fn blocked_peers_by_reason(&self) -> HashMap<BlockReason, usize> {
        let mut counts = HashMap::new();
        for info in self.blocked_peers_info.values() {
            *counts.entry(info.reason).or_insert(0) += 1;
        }
        counts
    }

    /// Update Prometheus metrics with current blocked peer counts by reason
    pub fn update_blocked_peer_metrics(&self) {
        let counts = self.blocked_peers_by_reason();

        if let Ok(gauge) = metrics::PEERS_BLOCKED_LOW_SCORE.as_ref() {
            gauge.set(counts.get(&BlockReason::LowScore).copied().unwrap_or(0) as i64);
        }
        if let Ok(gauge) = metrics::PEERS_BLOCKED_FAILED_HANDSHAKE.as_ref() {
            gauge.set(counts.get(&BlockReason::FailedHandshake).copied().unwrap_or(0) as i64);
        }
        if let Ok(gauge) = metrics::PEERS_BLOCKED_OUTGOING_CONNECTION_ERROR.as_ref() {
            gauge.set(counts.get(&BlockReason::OutgoingConnectionError).copied().unwrap_or(0) as i64);
        }
        if let Ok(gauge) = metrics::PEERS_BLOCKED_INCOMING_CONNECTION_ERROR.as_ref() {
            gauge.set(counts.get(&BlockReason::IncomingConnectionError).copied().unwrap_or(0) as i64);
        }
        if let Ok(gauge) = metrics::PEERS_BLOCKED_OTHER.as_ref() {
            gauge.set(counts.get(&BlockReason::Other).copied().unwrap_or(0) as i64);
        }
    }

    // Delegation methods for connection handling
    pub fn handle_pending_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<(), ConnectionDenied> {
        self.block_list
            .handle_pending_inbound_connection(connection_id, local_addr, remote_addr)
    }

    pub fn handle_established_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<(), ConnectionDenied> {
        self.block_list
            .handle_established_inbound_connection(connection_id, peer, local_addr, remote_addr)
            .map(|_| ()) // Discard the handler, we just want to know if connection is allowed
    }

    pub fn handle_pending_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        maybe_peer: Option<PeerId>,
        addresses: &[Multiaddr],
        effective_role: Endpoint,
    ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
        self.block_list.handle_pending_outbound_connection(
            connection_id,
            maybe_peer,
            addresses,
            effective_role,
        )
    }

    pub fn handle_established_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        addr: &Multiaddr,
        role_override: Endpoint,
        port_use: PortUse,
    ) -> Result<(), ConnectionDenied> {
        self.block_list
            .handle_established_outbound_connection(
                connection_id,
                peer,
                addr,
                role_override,
                port_use,
            )
            .map(|_| ()) // Discard the handler, we just want to know if connection is allowed
    }

    pub fn on_swarm_event(&mut self, event: FromSwarm) {
        self.block_list.on_swarm_event(event);
    }

    pub fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<libp2p::swarm::ToSwarm<std::convert::Infallible, std::convert::Infallible>>
    {
        // Forward CloseConnection events from allow_block_list to close connections for blocked
        // peers
        self.block_list.poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use libp2p::identity::Keypair;

    use super::*;

    /// Test helper to create a test BlockingManager
    fn create_test_blocking_manager() -> BlockingManager {
        let one_epoch_duration = Duration::from_secs(384); // 32 slots * 12 seconds
        BlockingManager::new(one_epoch_duration)
    }

    /// Test helper to create a test peer ID
    fn create_test_peer_id() -> PeerId {
        let keypair = Keypair::generate_ed25519();
        keypair.public().to_peer_id()
    }

    #[tokio::test(start_paused = true)]
    async fn test_peer_blocking() {
        let mut blocking_manager = create_test_blocking_manager();
        let peer_id = create_test_peer_id();

        // Initially, peer should not be blocked
        assert!(!blocking_manager.blocked_peers().contains(&peer_id));
        assert!(blocking_manager.blocked_peers_info.is_empty());

        // Block the peer with LowScore reason
        let was_blocked = blocking_manager.block_peer(peer_id, BlockReason::LowScore);
        assert!(was_blocked);

        // Verify peer is now blocked
        assert!(blocking_manager.blocked_peers().contains(&peer_id));
        assert!(blocking_manager.blocked_peers_info.contains_key(&peer_id));

        // Verify the block time and reason were recorded
        let info = blocking_manager.blocked_peers_info.get(&peer_id).unwrap();
        let expected_time = tokio::time::Instant::now();
        assert_eq!(info.blocked_at, expected_time);
        assert_eq!(info.reason, BlockReason::LowScore);
    }

    #[tokio::test(start_paused = true)]
    async fn test_peer_unblocking_after_timeout() {
        let mut blocking_manager = create_test_blocking_manager();
        let peer_id = create_test_peer_id();

        // Block the peer
        blocking_manager.block_peer(peer_id, BlockReason::LowScore);
        assert!(blocking_manager.blocked_peers().contains(&peer_id));

        // Advance time beyond the retain_score period
        let retain_score_duration =
            blocking_manager.one_epoch_duration * RETAIN_SCORE_EPOCH_MULTIPLIER;
        tokio::time::advance(retain_score_duration + Duration::from_secs(1)).await;

        // Check and unblock expired peers
        blocking_manager.check_and_unblock_expired_peers();

        // Verify peer is now unblocked
        assert!(!blocking_manager.blocked_peers().contains(&peer_id));
        assert!(!blocking_manager.blocked_peers_info.contains_key(&peer_id));
    }

    #[tokio::test(start_paused = true)]
    async fn test_peer_not_unblocked_before_timeout() {
        let mut blocking_manager = create_test_blocking_manager();
        let peer_id = create_test_peer_id();

        // Block the peer
        blocking_manager.block_peer(peer_id, BlockReason::LowScore);
        assert!(blocking_manager.blocked_peers().contains(&peer_id));

        // Advance time but not enough to trigger unblocking
        let retain_score_duration =
            blocking_manager.one_epoch_duration * RETAIN_SCORE_EPOCH_MULTIPLIER;
        tokio::time::advance(retain_score_duration - Duration::from_secs(10)).await;

        // Check and unblock expired peers
        blocking_manager.check_and_unblock_expired_peers();

        // Verify peer is still blocked
        assert!(blocking_manager.blocked_peers().contains(&peer_id));
        assert!(blocking_manager.blocked_peers_info.contains_key(&peer_id));
    }

    #[tokio::test(start_paused = true)]
    async fn test_multiple_peers_blocking_and_unblocking() {
        let mut blocking_manager = create_test_blocking_manager();
        let peer_id_1 = create_test_peer_id();
        let peer_id_2 = create_test_peer_id();
        let peer_id_3 = create_test_peer_id();

        // Block peer_1 first
        blocking_manager.block_peer(peer_id_1, BlockReason::LowScore);

        // Advance time a bit
        tokio::time::advance(Duration::from_secs(100)).await;

        // Block peer_2 and peer_3 with different reasons
        blocking_manager.block_peer(peer_id_2, BlockReason::FailedHandshake);
        blocking_manager.block_peer(peer_id_3, BlockReason::OutgoingConnectionError);

        // Verify all are blocked
        assert_eq!(blocking_manager.blocked_peers().len(), 3);
        assert_eq!(blocking_manager.blocked_peers_info.len(), 3);

        // Advance time enough to unblock only peer_1 (it was blocked earlier)
        let retain_score_duration =
            blocking_manager.one_epoch_duration * RETAIN_SCORE_EPOCH_MULTIPLIER;
        tokio::time::advance(retain_score_duration - Duration::from_secs(50)).await;

        // Check and unblock expired peers
        blocking_manager.check_and_unblock_expired_peers();

        // Only peer_1 should be unblocked
        assert!(!blocking_manager.blocked_peers().contains(&peer_id_1));
        assert!(blocking_manager.blocked_peers().contains(&peer_id_2));
        assert!(blocking_manager.blocked_peers().contains(&peer_id_3));
        assert_eq!(blocking_manager.blocked_peers().len(), 2);
        assert_eq!(blocking_manager.blocked_peers_info.len(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn test_manual_unblock_peer() {
        let mut blocking_manager = create_test_blocking_manager();
        let peer_id = create_test_peer_id();

        // Block the peer
        blocking_manager.block_peer(peer_id, BlockReason::LowScore);
        assert!(blocking_manager.blocked_peers().contains(&peer_id));

        // Manually unblock the peer
        let was_unblocked = blocking_manager.unblock_peer(peer_id);
        assert!(was_unblocked);

        // Verify peer is now unblocked
        assert!(!blocking_manager.blocked_peers().contains(&peer_id));
        assert!(
            !blocking_manager
                .blocked_peers_info
                .contains_key(&peer_id)
        );

        // Trying to unblock again should return false
        let was_unblocked_again = blocking_manager.unblock_peer(peer_id);
        assert!(!was_unblocked_again);
    }

    #[tokio::test]
    async fn test_block_and_unblock() {
        let mut blocking_manager = create_test_blocking_manager();
        let peer_id = create_test_peer_id();

        // Test block_peer method
        let was_blocked = blocking_manager.block_peer(peer_id, BlockReason::Other);
        assert!(was_blocked);
        assert!(blocking_manager.blocked_peers().contains(&peer_id));
        assert!(blocking_manager.blocked_peers_info.contains_key(&peer_id));

        // Test unblock
        let was_unblocked = blocking_manager.unblock_peer(peer_id);
        assert!(was_unblocked);
        assert!(!blocking_manager.blocked_peers().contains(&peer_id));
    }

    #[tokio::test]
    async fn test_blocked_peers_count() {
        let mut blocking_manager = create_test_blocking_manager();
        let peer_id_1 = create_test_peer_id();
        let peer_id_2 = create_test_peer_id();

        // Initially no blocked peers
        assert_eq!(blocking_manager.blocked_peers_count(), 0);

        // Block one peer
        blocking_manager.block_peer(peer_id_1, BlockReason::LowScore);
        assert_eq!(blocking_manager.blocked_peers_count(), 1);

        // Block another peer
        blocking_manager.block_peer(peer_id_2, BlockReason::FailedHandshake);
        assert_eq!(blocking_manager.blocked_peers_count(), 2);

        // Unblock one peer
        blocking_manager.unblock_peer(peer_id_1);
        assert_eq!(blocking_manager.blocked_peers_count(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn test_no_double_blocking() {
        let mut blocking_manager = create_test_blocking_manager();
        let peer_id = create_test_peer_id();

        // Block the peer for the first time
        blocking_manager.block_peer(peer_id, BlockReason::IncomingConnectionError);
        assert!(blocking_manager.blocked_peers().contains(&peer_id));
        assert_eq!(blocking_manager.blocked_peers_info.len(), 1);

        let first_block_info = blocking_manager
            .blocked_peers_info
            .get(&peer_id)
            .unwrap()
            .clone();

        // Advance time slightly
        tokio::time::advance(Duration::from_secs(10)).await;

        // Try to block the same peer again with different reason
        blocking_manager.block_peer(peer_id, BlockReason::LowScore);

        // Should still be blocked but time/reason shouldn't change (no double blocking)
        assert!(blocking_manager.blocked_peers().contains(&peer_id));
        assert_eq!(blocking_manager.blocked_peers_info.len(), 1);

        let second_block_info = blocking_manager.blocked_peers_info.get(&peer_id).unwrap();
        assert_eq!(first_block_info.blocked_at, second_block_info.blocked_at);
        assert_eq!(first_block_info.reason, second_block_info.reason);
    }
}
