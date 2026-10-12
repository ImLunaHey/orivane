use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const LATENCY_BUCKETS_MS: [u64; 7] = [10, 25, 50, 100, 250, 500, 1_000];

#[derive(Clone)]
pub struct ServiceMetrics {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    service: &'static str,
    started: Instant,
    ready: AtomicBool,
    requests: AtomicU64,
    request_failures: AtomicU64,
    request_latency_micros: AtomicU64,
    request_latency_buckets: [AtomicU64; LATENCY_BUCKETS_MS.len()],
    gateway_events: AtomicU64,
    gateway_errors: AtomicU64,
    discord_interactions: AtomicU64,
    discord_interaction_failures: AtomicU64,
    interaction_latency_micros: AtomicU64,
    interaction_latency_buckets: [AtomicU64; LATENCY_BUCKETS_MS.len()],
    scheduled_ticks: AtomicU64,
    scheduled_failures: AtomicU64,
    request_sequence: AtomicU64,
}

impl ServiceMetrics {
    pub fn new(service: &'static str) -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                service,
                started: Instant::now(),
                ready: AtomicBool::new(false),
                requests: AtomicU64::new(0),
                request_failures: AtomicU64::new(0),
                request_latency_micros: AtomicU64::new(0),
                request_latency_buckets: std::array::from_fn(|_| AtomicU64::new(0)),
                gateway_events: AtomicU64::new(0),
                gateway_errors: AtomicU64::new(0),
                discord_interactions: AtomicU64::new(0),
                discord_interaction_failures: AtomicU64::new(0),
                interaction_latency_micros: AtomicU64::new(0),
                interaction_latency_buckets: std::array::from_fn(|_| AtomicU64::new(0)),
                scheduled_ticks: AtomicU64::new(0),
                scheduled_failures: AtomicU64::new(0),
                request_sequence: AtomicU64::new(0),
            }),
        }
    }

    pub fn set_ready(&self, ready: bool) {
        self.inner.ready.store(ready, Ordering::Relaxed);
    }

    pub fn is_ready(&self) -> bool {
        self.inner.ready.load(Ordering::Relaxed)
    }

    pub fn record_request(&self, failed: bool, latency: Duration) {
        self.inner.requests.fetch_add(1, Ordering::Relaxed);
        record_latency(
            &self.inner.request_latency_micros,
            &self.inner.request_latency_buckets,
            latency,
        );
        if failed {
            self.inner.request_failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_gateway_event(&self) {
        self.inner.gateway_events.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_gateway_error(&self) {
        self.inner.gateway_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_interaction(&self, failed: bool) {
        self.inner
            .discord_interactions
            .fetch_add(1, Ordering::Relaxed);
        if failed {
            self.inner
                .discord_interaction_failures
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_interaction_failure(&self) {
        self.inner
            .discord_interaction_failures
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_interaction_latency(&self, latency: Duration) {
        record_latency(
            &self.inner.interaction_latency_micros,
            &self.inner.interaction_latency_buckets,
            latency,
        );
    }

    pub fn record_scheduled_tick(&self, failed: bool) {
        self.inner.scheduled_ticks.fetch_add(1, Ordering::Relaxed);
        if failed {
            self.inner
                .scheduled_failures
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_scheduled_failure(&self) {
        self.inner
            .scheduled_failures
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn request_id(&self) -> String {
        let unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let sequence = self.inner.request_sequence.fetch_add(1, Ordering::Relaxed);
        format!("{}-{unix_ms:x}-{sequence:x}", self.inner.service)
    }

    pub fn render_prometheus(&self) -> String {
        let service = self.inner.service;
        let ready = u8::from(self.is_ready());
        let uptime = self.inner.started.elapsed().as_secs_f64();
        let mut output = format!(
            concat!(
                "# HELP orivane_service_ready Whether the service can accept work.\n",
                "# TYPE orivane_service_ready gauge\n",
                "orivane_service_ready{{service=\"{service}\"}} {ready}\n",
                "# HELP orivane_service_uptime_seconds Process uptime.\n",
                "# TYPE orivane_service_uptime_seconds gauge\n",
                "orivane_service_uptime_seconds{{service=\"{service}\"}} {uptime}\n",
                "# HELP orivane_http_requests_total HTTP requests handled.\n",
                "# TYPE orivane_http_requests_total counter\n",
                "orivane_http_requests_total{{service=\"{service}\"}} {requests}\n",
                "# HELP orivane_http_request_failures_total HTTP 5xx responses.\n",
                "# TYPE orivane_http_request_failures_total counter\n",
                "orivane_http_request_failures_total{{service=\"{service}\"}} {request_failures}\n",
                "# HELP orivane_gateway_events_total Discord gateway events received.\n",
                "# TYPE orivane_gateway_events_total counter\n",
                "orivane_gateway_events_total{{service=\"{service}\"}} {gateway_events}\n",
                "# HELP orivane_gateway_errors_total Discord gateway receive/close errors.\n",
                "# TYPE orivane_gateway_errors_total counter\n",
                "orivane_gateway_errors_total{{service=\"{service}\"}} {gateway_errors}\n",
                "# HELP orivane_discord_interactions_total Discord interactions received.\n",
                "# TYPE orivane_discord_interactions_total counter\n",
                "orivane_discord_interactions_total{{service=\"{service}\"}} {interactions}\n",
                "# HELP orivane_discord_interaction_failures_total Failed Discord interactions.\n",
                "# TYPE orivane_discord_interaction_failures_total counter\n",
                "orivane_discord_interaction_failures_total{{service=\"{service}\"}} {interaction_failures}\n",
                "# HELP orivane_scheduler_ticks_total Scheduler ticks executed.\n",
                "# TYPE orivane_scheduler_ticks_total counter\n",
                "orivane_scheduler_ticks_total{{service=\"{service}\"}} {scheduled_ticks}\n",
                "# HELP orivane_scheduler_failures_total Scheduler tick failures.\n",
                "# TYPE orivane_scheduler_failures_total counter\n",
                "orivane_scheduler_failures_total{{service=\"{service}\"}} {scheduled_failures}\n"
            ),
            service = service,
            ready = ready,
            uptime = uptime,
            requests = self.inner.requests.load(Ordering::Relaxed),
            request_failures = self.inner.request_failures.load(Ordering::Relaxed),
            gateway_events = self.inner.gateway_events.load(Ordering::Relaxed),
            gateway_errors = self.inner.gateway_errors.load(Ordering::Relaxed),
            interactions = self.inner.discord_interactions.load(Ordering::Relaxed),
            interaction_failures = self
                .inner
                .discord_interaction_failures
                .load(Ordering::Relaxed),
            scheduled_ticks = self.inner.scheduled_ticks.load(Ordering::Relaxed),
            scheduled_failures = self.inner.scheduled_failures.load(Ordering::Relaxed),
        );
        render_histogram(
            &mut output,
            "orivane_http_request_latency_seconds",
            "HTTP request latency.",
            service,
            self.inner.requests.load(Ordering::Relaxed),
            self.inner.request_latency_micros.load(Ordering::Relaxed),
            &self.inner.request_latency_buckets,
        );
        render_histogram(
            &mut output,
            "orivane_discord_interaction_latency_seconds",
            "Discord interaction handling latency.",
            service,
            self.inner.discord_interactions.load(Ordering::Relaxed),
            self.inner
                .interaction_latency_micros
                .load(Ordering::Relaxed),
            &self.inner.interaction_latency_buckets,
        );
        output
    }
}

fn record_latency(
    sum: &AtomicU64,
    buckets: &[AtomicU64; LATENCY_BUCKETS_MS.len()],
    latency: Duration,
) {
    let micros = latency.as_micros().min(u128::from(u64::MAX)) as u64;
    sum.fetch_add(micros, Ordering::Relaxed);
    let millis = latency.as_millis().min(u128::from(u64::MAX)) as u64;
    for (index, upper) in LATENCY_BUCKETS_MS.iter().enumerate() {
        if millis <= *upper {
            buckets[index].fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn render_histogram(
    output: &mut String,
    name: &str,
    help: &str,
    service: &str,
    count: u64,
    sum_micros: u64,
    buckets: &[AtomicU64; LATENCY_BUCKETS_MS.len()],
) {
    use std::fmt::Write as _;
    let _ = writeln!(output, "# HELP {name} {help}");
    let _ = writeln!(output, "# TYPE {name} histogram");
    for (index, upper) in LATENCY_BUCKETS_MS.iter().enumerate() {
        let seconds = *upper as f64 / 1_000.0;
        let value = buckets[index].load(Ordering::Relaxed);
        let _ = writeln!(
            output,
            "{name}_bucket{{service=\"{service}\",le=\"{seconds}\"}} {value}"
        );
    }
    let _ = writeln!(
        output,
        "{name}_bucket{{service=\"{service}\",le=\"+Inf\"}} {count}"
    );
    let _ = writeln!(
        output,
        "{name}_sum{{service=\"{service}\"}} {}",
        sum_micros as f64 / 1_000_000.0
    );
    let _ = writeln!(output, "{name}_count{{service=\"{service}\"}} {count}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_counters_without_high_cardinality_labels() {
        let metrics = ServiceMetrics::new("test");
        metrics.set_ready(true);
        metrics.record_request(true, Duration::from_millis(20));
        metrics.record_gateway_event();
        metrics.record_interaction(false);
        let output = metrics.render_prometheus();
        assert!(output.contains("orivane_service_ready{service=\"test\"} 1"));
        assert!(output.contains("orivane_http_request_failures_total{service=\"test\"} 1"));
        assert!(!output.contains("guild_id"));
        assert!(!output.contains("user_id"));
    }

    #[test]
    fn request_ids_are_unique() {
        let metrics = ServiceMetrics::new("api");
        assert_ne!(metrics.request_id(), metrics.request_id());
    }
}
