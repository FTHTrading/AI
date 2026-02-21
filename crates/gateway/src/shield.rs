// Shield — API Defense Layer
//
// Layered protections between the public internet and Genesis Core:
//   - Per-IP rate limiting (token bucket)
//   - Emergency lockdown controls
//   - Request validation & size enforcement
//   - Security headers
//   - Anomaly logging
//
// No treasury or state mutations happen here. Shield only filters.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

// ───────────────────────────────────────────
// EMERGENCY CONTROLS
// ───────────────────────────────────────────

/// Operational mode for the Genesis gateway.
#[derive(Debug, Clone, PartialEq)]
pub enum GatewayMode {
    /// Normal operation — all endpoints active.
    Normal,
    /// Lockdown — only read-only endpoints respond (GET /status, GET /genesis).
    /// All mutation endpoints return 503.
    Lockdown,
    /// Full shutdown — all endpoints return 503.
    Shutdown,
}

/// Emergency controls shared across all handlers.
#[derive(Debug, Clone)]
pub struct EmergencyControls {
    pub mode: GatewayMode,
    /// When true, POST /register returns 503 regardless of mode.
    pub intake_disabled: bool,
    /// When true, treasury mutations are blocked in the runtime.
    pub treasury_frozen: bool,
}

impl Default for EmergencyControls {
    fn default() -> Self {
        EmergencyControls {
            mode: GatewayMode::Normal,
            intake_disabled: false,
            treasury_frozen: false,
        }
    }
}

impl EmergencyControls {
    /// Create from environment variables.
    pub fn from_env() -> Self {
        let mode = match std::env::var("GENESIS_MODE").as_deref() {
            Ok("lockdown") => GatewayMode::Lockdown,
            Ok("shutdown") => GatewayMode::Shutdown,
            _ => GatewayMode::Normal,
        };

        let intake_disabled = std::env::var("GENESIS_INTAKE_DISABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let treasury_frozen = std::env::var("GENESIS_TREASURY_FROZEN")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        EmergencyControls {
            mode,
            intake_disabled,
            treasury_frozen,
        }
    }
}

pub type SharedControls = Arc<Mutex<EmergencyControls>>;

// ───────────────────────────────────────────
// RATE LIMITER (per-IP token bucket)
// ───────────────────────────────────────────

/// Per-IP token bucket entry.
#[derive(Debug, Clone)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

/// Route-specific rate limit configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum burst tokens.
    pub burst: f64,
    /// Tokens refilled per second.
    pub refill_rate: f64,
}

impl RateLimitConfig {
    pub fn new(requests_per_minute: f64, burst: f64) -> Self {
        RateLimitConfig {
            burst,
            refill_rate: requests_per_minute / 60.0,
        }
    }
}

/// Shared rate limiter state.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<IpAddr, Bucket>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        RateLimiter {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request from this IP is allowed. Returns true if allowed.
    pub fn check(&self, ip: IpAddr) -> bool {
        let mut buckets = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        let bucket = buckets.entry(ip).or_insert_with(|| Bucket {
            tokens: self.config.burst,
            last_refill: now,
        });

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.config.refill_rate)
            .min(self.config.burst);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Evict stale entries older than `max_age_secs` to prevent unbounded growth.
    pub fn evict_stale(&self, max_age_secs: u64) {
        let mut buckets = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        let max_age = std::time::Duration::from_secs(max_age_secs);
        buckets.retain(|_, b| now.duration_since(b.last_refill) < max_age);
    }

    /// Number of tracked IPs (for monitoring).
    pub fn tracked_ips(&self) -> usize {
        self.buckets.lock().unwrap_or_else(|e| e.into_inner()).len()
    }
}

/// Default rate limiter for read endpoints: 60 req/min, burst 120.
pub fn read_limiter() -> RateLimiter {
    RateLimiter::new(RateLimitConfig::new(60.0, 120.0))
}

/// Strict rate limiter for mutation endpoints: 5 req/min, burst 10.
pub fn write_limiter() -> RateLimiter {
    RateLimiter::new(RateLimitConfig::new(5.0, 10.0))
}

// ───────────────────────────────────────────
// AXUM MIDDLEWARE: Rate Limiting
// ───────────────────────────────────────────

/// Extract the client IP from the request, falling back to 127.0.0.1.
fn extract_ip(req: &Request<Body>) -> IpAddr {
    // Try X-Forwarded-For first (reverse proxy / Cloudflare)
    if let Some(xff) = req.headers().get("x-forwarded-for") {
        if let Ok(val) = xff.to_str() {
            if let Some(first) = val.split(',').next() {
                if let Ok(ip) = first.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(xri) = req.headers().get("x-real-ip") {
        if let Ok(val) = xri.to_str() {
            if let Ok(ip) = val.trim().parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Fallback
    "127.0.0.1".parse().unwrap()
}

/// Rate limit middleware — returns 429 if bucket empty.
pub async fn rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<RateLimiter>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let ip = extract_ip(&req);

    if !limiter.check(ip) {
        tracing::warn!(ip = %ip, path = %req.uri().path(), "Rate limited");
        let body = serde_json::json!({
            "error": "Too many requests. Slow down.",
            "retry_after_seconds": 10
        });
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, "10")],
            Json(body),
        ).into_response().into();
    }

    next.run(req).await
}

// ───────────────────────────────────────────
// AXUM MIDDLEWARE: Security Headers
// ───────────────────────────────────────────

/// Add defensive security headers to every response.
pub async fn security_headers_middleware(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let mut resp = next.run(req).await;
    let headers = resp.headers_mut();

    // Prevent MIME sniffing
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        header::HeaderValue::from_static("nosniff"),
    );
    // Prevent clickjacking
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        header::HeaderValue::from_static("DENY"),
    );
    // XSS protection
    headers.insert(
        header::HeaderName::from_static("x-xss-protection"),
        header::HeaderValue::from_static("1; mode=block"),
    );
    // No referrer leakage
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        header::HeaderValue::from_static("no-referrer"),
    );
    // Strict transport security (when behind TLS)
    headers.insert(
        header::HeaderName::from_static("strict-transport-security"),
        header::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    // Content security policy (restrict sources)
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        header::HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline'; img-src 'self'"),
    );
    // Server identity — reveal nothing
    headers.insert(
        header::SERVER,
        header::HeaderValue::from_static("Genesis"),
    );

    resp
}

// ───────────────────────────────────────────
// AXUM MIDDLEWARE: Emergency Mode
// ───────────────────────────────────────────

/// Block requests based on emergency controls.
pub async fn emergency_middleware(
    axum::extract::State(controls): axum::extract::State<SharedControls>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let ctrl = controls.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    match ctrl.mode {
        GatewayMode::Shutdown => {
            tracing::warn!("Request blocked — SHUTDOWN mode");
            return shutdown_response();
        }
        GatewayMode::Lockdown => {
            // Allow read-only GET endpoints, block everything else
            let is_read = method == axum::http::Method::GET;
            if !is_read {
                tracing::warn!(path = %path, "Request blocked — LOCKDOWN mode");
                return lockdown_response();
            }
        }
        GatewayMode::Normal => {}
    }

    // Even in Normal mode, check intake_disabled for /register
    if ctrl.intake_disabled && path == "/register" {
        tracing::warn!("Registration blocked — intake disabled");
        return intake_disabled_response();
    }

    next.run(req).await
}

fn shutdown_response() -> Response<Body> {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "Genesis Protocol is in shutdown mode.",
            "retry": false
        })),
    ).into_response()
}

fn lockdown_response() -> Response<Body> {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "Genesis Protocol is in lockdown mode. Read-only access only.",
            "retry": false
        })),
    ).into_response()
}

fn intake_disabled_response() -> Response<Body> {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "Agent intake is currently disabled.",
            "retry": true
        })),
    ).into_response()
}

// ───────────────────────────────────────────
// INPUT VALIDATION
// ───────────────────────────────────────────

/// Maximum allowed length for string fields in registration requests.
pub const MAX_FIELD_LENGTH: usize = 256;

/// Maximum request body size (32 KB).
pub const MAX_BODY_SIZE: usize = 32 * 1024;

/// Validate registration request fields for safety.
pub fn validate_registration(external_id: &str, public_key: &str) -> Result<(), String> {
    if external_id.is_empty() {
        return Err("external_id is required".to_string());
    }
    if public_key.is_empty() {
        return Err("public_key is required".to_string());
    }
    if external_id.len() > MAX_FIELD_LENGTH {
        return Err(format!(
            "external_id exceeds maximum length of {} characters",
            MAX_FIELD_LENGTH,
        ));
    }
    if public_key.len() > MAX_FIELD_LENGTH {
        return Err(format!(
            "public_key exceeds maximum length of {} characters",
            MAX_FIELD_LENGTH,
        ));
    }

    // Block control characters (prevent log injection, prompt injection basics)
    let has_control = |s: &str| s.chars().any(|c| c.is_control() && c != '\n');
    if has_control(external_id) {
        return Err("external_id contains invalid characters".to_string());
    }
    if has_control(public_key) {
        return Err("public_key contains invalid characters".to_string());
    }

    // Block obvious injection patterns
    let suspicious = ["<script", "javascript:", "eval(", "exec(", "__proto__", "constructor"];
    let lower_id = external_id.to_lowercase();
    let lower_pk = public_key.to_lowercase();
    for pattern in &suspicious {
        if lower_id.contains(pattern) || lower_pk.contains(pattern) {
            return Err("Input contains suspicious content".to_string());
        }
    }

    Ok(())
}

// ───────────────────────────────────────────
// MONITORING RESPONSE
// ───────────────────────────────────────────

/// Shield status for diagnostic endpoint (internal use).
#[derive(Debug, Serialize)]
pub struct ShieldStatus {
    pub mode: String,
    pub intake_disabled: bool,
    pub treasury_frozen: bool,
    pub tracked_ips: usize,
}

// ───────────────────────────────────────────
// TESTS
// ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_normal_traffic() {
        let limiter = RateLimiter::new(RateLimitConfig::new(60.0, 10.0));
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        // Should allow up to burst size
        for _ in 0..10 {
            assert!(limiter.check(ip));
        }
        // Next request should be denied (bucket empty)
        assert!(!limiter.check(ip));
    }

    #[test]
    fn test_rate_limiter_different_ips_independent() {
        let limiter = RateLimiter::new(RateLimitConfig::new(60.0, 2.0));
        let ip1: IpAddr = "10.0.0.1".parse().unwrap();
        let ip2: IpAddr = "10.0.0.2".parse().unwrap();

        assert!(limiter.check(ip1));
        assert!(limiter.check(ip1));
        assert!(!limiter.check(ip1)); // ip1 exhausted

        assert!(limiter.check(ip2)); // ip2 still has tokens
        assert!(limiter.check(ip2));
        assert!(!limiter.check(ip2));
    }

    #[test]
    fn test_rate_limiter_evict_stale() {
        let limiter = RateLimiter::new(RateLimitConfig::new(60.0, 5.0));
        let ip: IpAddr = "10.0.0.1".parse().unwrap();
        limiter.check(ip);
        assert_eq!(limiter.tracked_ips(), 1);

        // Evict with max_age=0 should not remove fresh entries
        // (they were just created, so less than max_age)
        limiter.evict_stale(3600);
        assert_eq!(limiter.tracked_ips(), 1);
    }

    #[test]
    fn test_emergency_controls_default() {
        let ctrl = EmergencyControls::default();
        assert_eq!(ctrl.mode, GatewayMode::Normal);
        assert!(!ctrl.intake_disabled);
        assert!(!ctrl.treasury_frozen);
    }

    #[test]
    fn test_validate_registration_valid() {
        assert!(validate_registration("moltbook:agent-1", "pk_abc123").is_ok());
    }

    #[test]
    fn test_validate_registration_empty() {
        assert!(validate_registration("", "pk").is_err());
        assert!(validate_registration("id", "").is_err());
    }

    #[test]
    fn test_validate_registration_too_long() {
        let long = "x".repeat(MAX_FIELD_LENGTH + 1);
        assert!(validate_registration(&long, "pk").is_err());
        assert!(validate_registration("id", &long).is_err());
    }

    #[test]
    fn test_validate_registration_control_chars() {
        assert!(validate_registration("id\x00hidden", "pk").is_err());
        assert!(validate_registration("id", "pk\x07bell").is_err());
    }

    #[test]
    fn test_validate_registration_injection() {
        assert!(validate_registration("<script>alert(1)</script>", "pk").is_err());
        assert!(validate_registration("id", "javascript:void(0)").is_err());
        assert!(validate_registration("__proto__", "pk").is_err());
        assert!(validate_registration("id", "eval(something)").is_err());
    }

    #[test]
    fn test_write_limiter_stricter() {
        let read = read_limiter();
        let write = write_limiter();
        let ip: IpAddr = "10.0.0.1".parse().unwrap();

        // Read allows 120 burst
        let mut read_count = 0;
        for _ in 0..200 {
            if read.check(ip) { read_count += 1; }
        }

        // Write allows only 10 burst
        let mut write_count = 0;
        for _ in 0..200 {
            if write.check(ip) { write_count += 1; }
        }

        assert!(read_count > write_count);
        assert_eq!(read_count, 120);
        assert_eq!(write_count, 10);
    }
}
