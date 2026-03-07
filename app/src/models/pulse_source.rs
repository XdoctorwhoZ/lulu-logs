use std::time::Instant;

// ---------------------------------------------------------------------------
// PulseSourceEntry
// ---------------------------------------------------------------------------

/// Tracks the last heartbeat received from a `lulu-pulse/{source}` topic.
#[derive(Debug, Clone)]
pub struct PulseSourceEntry {
    /// Source identifier derived from the topic (e.g. `"psu/channel-1"`).
    pub source: String,
    /// ISO 8601 timestamp string from the last received JSON payload.
    pub last_seen_ts: String,
    /// Monotonic instant at which the last pulse was received.
    pub last_seen_at: Instant,
    /// Optional version string reported by the source (e.g. `"1.2.3"`).
    pub version: Option<String>,
}

impl PulseSourceEntry {
    /// Returns `true` if a pulse has been received within the last 6 seconds
    /// (3× the nominal 2-second emission interval, as per spec §7.4).
    pub fn is_online(&self) -> bool {
        self.last_seen_at.elapsed().as_secs_f64() < 6.0
    }

    /// Returns a human-readable status label.
    pub fn status_label(&self) -> &'static str {
        if self.is_online() { "online" } else { "offline" }
    }

    /// Returns the CSS class for the status dot.
    pub fn dot_css_class(&self) -> &'static str {
        if self.is_online() { "pulse-dot online" } else { "pulse-dot offline" }
    }
}
