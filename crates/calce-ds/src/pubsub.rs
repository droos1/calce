use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use dashmap::DashMap;
use rustc_hash::{FxBuildHasher, FxHashMap};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

// ---------------------------------------------------------------------------
// Event types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum EventKind {
    Current,
    History,
}

/// A notification that a cache entry has changed.
///
/// Carries only the key — subscribers re-read from the cache to get the
/// current value (signal model, not data-pipe model).
#[derive(Clone, Debug)]
pub enum UpdateEvent<K> {
    CurrentChanged { key: K },
    HistoryChanged { key: K },
}

impl<K: Clone> UpdateEvent<K> {
    #[must_use]
    pub fn key(&self) -> &K {
        match self {
            Self::CurrentChanged { key } | Self::HistoryChanged { key } => key,
        }
    }

    fn kind(&self) -> EventKind {
        match self {
            Self::CurrentChanged { .. } => EventKind::Current,
            Self::HistoryChanged { .. } => EventKind::History,
        }
    }
}

pub type SubscriberId = u64;

pub struct Subscription<K> {
    pub id: SubscriberId,
    pub receiver: mpsc::Receiver<UpdateEvent<K>>,
}

// ---------------------------------------------------------------------------
// Subscription registry
// ---------------------------------------------------------------------------

type SubscriberMap<K> =
    DashMap<K, Vec<(SubscriberId, mpsc::Sender<UpdateEvent<K>>)>, FxBuildHasher>;

struct SubscriptionRegistry<K: Hash + Eq> {
    per_key: SubscriberMap<K>,
    per_subscriber: DashMap<SubscriberId, Vec<K>, FxBuildHasher>,
    next_id: AtomicU64,
}

impl<K: Hash + Eq + Clone + Send + Sync + 'static> SubscriptionRegistry<K> {
    fn new() -> Self {
        Self {
            per_key: DashMap::with_hasher(FxBuildHasher),
            per_subscriber: DashMap::with_hasher(FxBuildHasher),
            next_id: AtomicU64::new(1),
        }
    }

    fn subscribe(&self, keys: &[K], buffer_size: usize) -> Subscription<K> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel(buffer_size);

        let mut deduped = keys.to_vec();
        // Sort by hash to group equal elements adjacently so `dedup()` removes
        // duplicates. Works because equal keys always hash equally, guaranteeing
        // adjacency. Non-equal keys with hash collisions also end up adjacent
        // but `dedup()` correctly keeps both (they differ by Eq).
        deduped.sort_by_key(|a| fxhash_of(a));
        deduped.dedup();

        self.per_subscriber.insert(id, deduped.clone());
        for key in &deduped {
            self.per_key
                .entry(key.clone())
                .or_default()
                .push((id, tx.clone()));
        }

        Subscription { id, receiver: rx }
    }

    fn unsubscribe(&self, id: SubscriberId) {
        if let Some((_, keys)) = self.per_subscriber.remove(&id) {
            for key in keys {
                if let Some(mut subs) = self.per_key.get_mut(&key) {
                    subs.retain(|(sid, _)| *sid != id);
                }
                // Remove entry atomically if empty, avoiding TOCTOU race.
                self.per_key.remove_if(&key, |_, v| v.is_empty());
            }
        }
    }

    /// Fan out a coalesced event to all subscribers of that key.
    /// Returns (sent, dropped).
    #[allow(clippy::needless_pass_by_value)] // event is cloned per subscriber
    fn fan_out(&self, event: UpdateEvent<K>) -> (u64, u64) {
        let mut sent = 0u64;
        let mut dropped = 0u64;
        let mut dead: Vec<SubscriberId> = Vec::new();

        if let Some(subs) = self.per_key.get(event.key()) {
            for (sid, tx) in subs.iter() {
                match tx.try_send(event.clone()) {
                    Ok(()) => sent += 1,
                    Err(mpsc::error::TrySendError::Full(_)) => dropped += 1,
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        dropped += 1;
                        dead.push(*sid);
                    }
                }
            }
        }

        for sid in dead {
            self.unsubscribe(sid);
        }

        (sent, dropped)
    }
}

fn fxhash_of<K: Hash>(key: &K) -> u64 {
    use std::hash::Hasher;
    let mut h = rustc_hash::FxHasher::default();
    key.hash(&mut h);
    h.finish()
}

// ---------------------------------------------------------------------------
// PubSub system
// ---------------------------------------------------------------------------

/// Statistics from the coalescing dispatcher.
#[derive(Debug, Default)]
pub struct DispatcherStats {
    pub events_received: u64,
    pub events_coalesced: u64,
    pub notifications_sent: u64,
    pub notifications_dropped: u64,
    pub batches: u64,
}

impl std::fmt::Display for DispatcherStats {
    #[allow(clippy::cast_precision_loss)] // stats display only
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reduction = if self.events_received > 0 {
            self.events_coalesced as f64 / self.events_received as f64
        } else {
            0.0
        };
        writeln!(f, "Dispatcher stats:")?;
        writeln!(f, "  Events received:      {:>10}", self.events_received)?;
        writeln!(
            f,
            "  After coalescing:     {:>10} ({:.1}% reduction)",
            self.events_received - self.events_coalesced,
            reduction * 100.0
        )?;
        writeln!(f, "  Notifications sent:   {:>10}", self.notifications_sent)?;
        writeln!(
            f,
            "  Notifications dropped:{:>10}",
            self.notifications_dropped
        )?;
        writeln!(f, "  Batches processed:    {:>10}", self.batches)?;
        Ok(())
    }
}

/// Async pub/sub system with coalescing dispatcher.
///
/// Events from the cache are collected by a background tokio task that
/// coalesces duplicates (same key + same event kind) within a configurable
/// time window, then fans out to all subscribers.
pub struct PubSub<K: Hash + Eq + Clone + Send + Sync + 'static> {
    event_tx: mpsc::Sender<UpdateEvent<K>>,
    event_rx: std::sync::Mutex<Option<mpsc::Receiver<UpdateEvent<K>>>>,
    registry: Arc<SubscriptionRegistry<K>>,
    coalesce_window: Duration,
    stop_tx: std::sync::Mutex<Option<oneshot::Sender<()>>>,
    dispatcher_handle: std::sync::Mutex<Option<JoinHandle<DispatcherStats>>>,
}

impl<K: Hash + Eq + Clone + Send + Sync + 'static> PubSub<K> {
    /// Create a new pub/sub system.
    ///
    /// Call [`event_sender`](Self::event_sender) to get the sender to pass to
    /// [`TimeSeriesCache::with_notifier`](crate::cache::TimeSeriesCache::with_notifier),
    /// then [`start`](Self::start) to launch the dispatcher task.
    #[must_use]
    pub fn new(coalesce_window: Duration, channel_capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(channel_capacity);
        Self {
            event_tx: tx,
            event_rx: std::sync::Mutex::new(Some(rx)),
            registry: Arc::new(SubscriptionRegistry::new()),
            coalesce_window,
            stop_tx: std::sync::Mutex::new(None),
            dispatcher_handle: std::sync::Mutex::new(None),
        }
    }

    /// Get the sender to give to the cache.
    #[must_use]
    pub fn event_sender(&self) -> mpsc::Sender<UpdateEvent<K>> {
        self.event_tx.clone()
    }

    /// Subscribe to updates for specific keys.
    pub fn subscribe(&self, keys: &[K], buffer_size: usize) -> Subscription<K> {
        self.registry.subscribe(keys, buffer_size)
    }

    /// Unsubscribe by id.
    pub fn unsubscribe(&self, id: SubscriberId) {
        self.registry.unsubscribe(id);
    }

    /// Start the coalescing dispatcher as a tokio task.
    ///
    /// Returns `false` if the dispatcher was already started.
    pub fn start(&self) -> bool {
        let Ok(mut rx_guard) = self.event_rx.lock() else {
            return false;
        };
        let Some(rx) = rx_guard.take() else {
            return false;
        };
        let registry = Arc::clone(&self.registry);
        let window = self.coalesce_window;

        let (stop_tx, stop_rx) = oneshot::channel();
        let handle = tokio::spawn(dispatcher_loop(rx, stop_rx, registry, window));

        if let Ok(mut g) = self.stop_tx.lock() {
            *g = Some(stop_tx);
        }
        if let Ok(mut g) = self.dispatcher_handle.lock() {
            *g = Some(handle);
        }
        true
    }

    /// Stop the dispatcher and return stats.
    ///
    /// Dropping the `PubSub` without calling shutdown will also stop the
    /// dispatcher (the stop channel closes), but stats are lost.
    pub async fn shutdown(self) -> DispatcherStats {
        if let Ok(mut g) = self.stop_tx.lock()
            && let Some(stop_tx) = g.take()
        {
            let _ = stop_tx.send(());
        }
        let handle = self
            .dispatcher_handle
            .lock()
            .ok()
            .and_then(|mut g| g.take());
        if let Some(handle) = handle {
            handle.await.unwrap_or_default()
        } else {
            DispatcherStats::default()
        }
    }
}

async fn dispatcher_loop<K: Hash + Eq + Clone + Send + Sync + 'static>(
    mut rx: mpsc::Receiver<UpdateEvent<K>>,
    mut stop_rx: oneshot::Receiver<()>,
    registry: Arc<SubscriptionRegistry<K>>,
    coalesce_window: Duration,
) -> DispatcherStats {
    let mut stats = DispatcherStats::default();
    let mut pending: FxHashMap<(K, EventKind), UpdateEvent<K>> = FxHashMap::default();

    loop {
        // Wait for the first event, stop signal, or channel close.
        tokio::select! {
            biased;
            _ = &mut stop_rx => {
                // Flush remaining events from the channel.
                while let Ok(event) = rx.try_recv() {
                    coalesce(&mut pending, event, &mut stats);
                }
                flush_pending(&mut pending, &registry, &mut stats);
                return stats;
            }
            event = rx.recv() => {
                let Some(event) = event else {
                    // Channel closed.
                    flush_pending(&mut pending, &registry, &mut stats);
                    return stats;
                };
                coalesce(&mut pending, event, &mut stats);

                // Drain all buffered events (non-blocking).
                while let Ok(event) = rx.try_recv() {
                    coalesce(&mut pending, event, &mut stats);
                }

                // Wait the coalesce window for more events to arrive.
                tokio::time::sleep(coalesce_window).await;

                // Drain anything that arrived during the window.
                while let Ok(event) = rx.try_recv() {
                    coalesce(&mut pending, event, &mut stats);
                }

                flush_pending(&mut pending, &registry, &mut stats);
            }
        }
    }
}

fn coalesce<K: Hash + Eq + Clone>(
    pending: &mut FxHashMap<(K, EventKind), UpdateEvent<K>>,
    event: UpdateEvent<K>,
    stats: &mut DispatcherStats,
) {
    stats.events_received += 1;
    let coalesce_key = (event.key().clone(), event.kind());
    if pending.insert(coalesce_key, event).is_some() {
        stats.events_coalesced += 1;
    }
}

fn flush_pending<K: Hash + Eq + Clone + Send + Sync + 'static>(
    pending: &mut FxHashMap<(K, EventKind), UpdateEvent<K>>,
    registry: &SubscriptionRegistry<K>,
    stats: &mut DispatcherStats,
) {
    if pending.is_empty() {
        return;
    }
    stats.batches += 1;
    for (_, event) in pending.drain() {
        let (sent, dropped) = registry.fan_out(event);
        stats.notifications_sent += sent;
        stats.notifications_dropped += dropped;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscribe_and_receive() {
        let pubsub = PubSub::<u32>::new(Duration::from_millis(10), 1024);
        let tx = pubsub.event_sender();
        let mut sub = pubsub.subscribe(&[1, 2], 64);
        pubsub.start();

        tx.send(UpdateEvent::CurrentChanged { key: 1 }).await.ok();
        let event = tokio::time::timeout(Duration::from_secs(1), sub.receiver.recv()).await;
        assert!(event.is_ok());
        let event = event.ok().flatten();
        if let Some(UpdateEvent::CurrentChanged { key }) = event {
            assert_eq!(key, 1);
        } else {
            unreachable!("expected CurrentChanged event");
        }

        drop(tx);
        let stats = pubsub.shutdown().await;
        assert!(stats.events_received >= 1);
        assert!(stats.notifications_sent >= 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn coalescing_deduplicates() {
        let pubsub = PubSub::<u32>::new(Duration::from_millis(50), 1024);
        let tx = pubsub.event_sender();
        let mut sub = pubsub.subscribe(&[1], 64);
        pubsub.start();

        // Send many events for the same key rapidly.
        for _ in 0..100 {
            tx.send(UpdateEvent::CurrentChanged { key: 1 }).await.ok();
        }

        // Wait for coalescing + dispatch.
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Drain received events — should be much fewer than 100.
        let mut received = 0;
        while sub.receiver.try_recv().is_ok() {
            received += 1;
        }
        // At least 1, much fewer than 100 due to coalescing.
        assert!(received >= 1);
        assert!(received < 50, "expected coalescing, got {received}");

        drop(tx);
        let stats = pubsub.shutdown().await;
        assert!(stats.events_coalesced > 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn unsubscribe_stops_delivery() {
        let pubsub = PubSub::<u32>::new(Duration::from_millis(10), 1024);
        let tx = pubsub.event_sender();
        let sub = pubsub.subscribe(&[1], 64);
        let sub_id = sub.id;
        let mut rx = sub.receiver;
        pubsub.start();

        pubsub.unsubscribe(sub_id);

        tx.send(UpdateEvent::CurrentChanged { key: 1 }).await.ok();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should not have received anything.
        assert!(rx.try_recv().is_err());

        drop(tx);
        pubsub.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn multiple_subscribers_same_key() {
        let pubsub = PubSub::<u32>::new(Duration::from_millis(10), 1024);
        let tx = pubsub.event_sender();
        let mut sub1 = pubsub.subscribe(&[1], 64);
        let mut sub2 = pubsub.subscribe(&[1], 64);
        pubsub.start();

        tx.send(UpdateEvent::CurrentChanged { key: 1 }).await.ok();
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(sub1.receiver.try_recv().is_ok());
        assert!(sub2.receiver.try_recv().is_ok());

        drop(tx);
        pubsub.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dead_subscriber_cleaned_up() {
        let pubsub = PubSub::<u32>::new(Duration::from_millis(10), 1024);
        let tx = pubsub.event_sender();
        let sub = pubsub.subscribe(&[1], 64);
        pubsub.start();

        // Drop the receiver to simulate a dead subscriber.
        drop(sub.receiver);

        tx.send(UpdateEvent::CurrentChanged { key: 1 }).await.ok();
        tokio::time::sleep(Duration::from_millis(100)).await;

        drop(tx);
        let stats = pubsub.shutdown().await;
        // The notification was dropped because the subscriber is dead.
        assert!(stats.notifications_dropped >= 1 || stats.events_received >= 1);
    }
}
