use codex_http_client::TransportError;
use http::StatusCode;
use std::time::Duration;

/// API specific telemetry.
pub trait RequestTelemetry: Send + Sync {
    /// Called immediately before an HTTP request attempt is dispatched.
    ///
    /// The default implementation keeps existing telemetry implementations
    /// source-compatible while allowing callers to observe request starts.
    fn on_request_start(&self, _attempt: u64, _request: &codex_http_client::Request) {}

    fn on_request(
        &self,
        attempt: u64,
        status: Option<StatusCode>,
        error: Option<&TransportError>,
        duration: Duration,
    );
}
