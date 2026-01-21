use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Security headers middleware for SOC2 compliance
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;

    // Add security headers
    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // Enable XSS protection
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Enforce HTTPS
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Content Security Policy
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' https://api.spotify.com https://api.music.apple.com"
        ),
    );

    // Referrer Policy
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions Policy
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    Ok(response)
}

/// Rate limiting and suspicious activity detection
#[derive(Clone)]
pub struct SecurityMonitor {
    failed_attempts: Arc<RwLock<HashMap<String, FailedAttemptTracker>>>,
    suspicious_ips: Arc<RwLock<HashMap<String, SuspiciousActivity>>>,
}

impl Default for SecurityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct FailedAttemptTracker {
    count: u32,
    first_attempt: chrono::DateTime<chrono::Utc>,
    last_attempt: chrono::DateTime<chrono::Utc>,
    locked_until: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
struct SuspiciousActivity {
    request_count: u32,
    first_request: chrono::DateTime<chrono::Utc>,
    last_request: chrono::DateTime<chrono::Utc>,
    blocked_until: Option<chrono::DateTime<chrono::Utc>>,
}

impl SecurityMonitor {
    pub fn new() -> Self {
        Self {
            failed_attempts: Arc::new(RwLock::new(HashMap::new())),
            suspicious_ips: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a failed login attempt
    pub async fn record_failed_attempt(&self, identifier: String) -> bool {
        let mut attempts = self.failed_attempts.write().await;
        let now = chrono::Utc::now();

        let tracker = attempts.entry(identifier).or_insert(FailedAttemptTracker {
            count: 0,
            first_attempt: now,
            last_attempt: now,
            locked_until: None,
        });

        // Reset counter if more than 1 hour has passed
        if now.signed_duration_since(tracker.first_attempt).num_hours() > 1 {
            tracker.count = 0;
            tracker.first_attempt = now;
        }

        tracker.count += 1;
        tracker.last_attempt = now;

        // Lock account after 5 failed attempts
        if tracker.count >= 5 {
            tracker.locked_until = Some(now + chrono::Duration::hours(1));
            return true; // Account is locked
        }

        false
    }

    /// Check if an account is locked
    pub async fn is_locked(&self, identifier: &str) -> bool {
        let attempts = self.failed_attempts.read().await;
        if let Some(tracker) = attempts.get(identifier) {
            if let Some(locked_until) = tracker.locked_until {
                return chrono::Utc::now() < locked_until;
            }
        }
        false
    }

    /// Clear failed attempts for successful login
    pub async fn clear_failed_attempts(&self, identifier: &str) {
        let mut attempts = self.failed_attempts.write().await;
        attempts.remove(identifier);
    }

    /// Record suspicious activity from IP
    pub async fn record_suspicious_activity(&self, ip: String) -> bool {
        let mut activities = self.suspicious_ips.write().await;
        let now = chrono::Utc::now();

        let activity = activities.entry(ip).or_insert(SuspiciousActivity {
            request_count: 0,
            first_request: now,
            last_request: now,
            blocked_until: None,
        });

        // Reset counter if more than 1 hour has passed
        if now
            .signed_duration_since(activity.first_request)
            .num_hours()
            > 1
        {
            activity.request_count = 0;
            activity.first_request = now;
        }

        activity.request_count += 1;
        activity.last_request = now;

        // Block IP after 100 requests in 1 hour
        if activity.request_count >= 100 {
            activity.blocked_until = Some(now + chrono::Duration::hours(24));
            return true; // IP is blocked
        }

        false
    }

    /// Check if an IP is blocked
    pub async fn is_ip_blocked(&self, ip: &str) -> bool {
        let activities = self.suspicious_ips.read().await;
        if let Some(activity) = activities.get(ip) {
            if let Some(blocked_until) = activity.blocked_until {
                return chrono::Utc::now() < blocked_until;
            }
        }
        false
    }

    /// Get security statistics
    pub async fn get_security_stats(&self) -> SecurityStats {
        let attempts = self.failed_attempts.read().await;
        let activities = self.suspicious_ips.read().await;

        let active_lockouts = attempts
            .values()
            .filter(|tracker| {
                tracker
                    .locked_until
                    .is_some_and(|until| chrono::Utc::now() < until)
            })
            .count();

        let blocked_ips = activities
            .values()
            .filter(|activity| {
                activity
                    .blocked_until
                    .is_some_and(|until| chrono::Utc::now() < until)
            })
            .count();

        SecurityStats {
            total_failed_attempts: attempts.len(),
            active_lockouts,
            total_suspicious_ips: activities.len(),
            blocked_ips,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SecurityStats {
    pub total_failed_attempts: usize,
    pub active_lockouts: usize,
    pub total_suspicious_ips: usize,
    pub blocked_ips: usize,
}

/// Vulnerability scanner integration
#[derive(Clone)]
pub struct VulnerabilityScanner {
    scan_results: Arc<RwLock<Vec<VulnerabilityScanResult>>>,
}

impl Default for VulnerabilityScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VulnerabilityScanResult {
    pub scan_id: uuid::Uuid,
    pub scan_type: String,
    pub scan_tool: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub vulnerabilities: Vec<Vulnerability>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub severity: VulnerabilitySeverity,
    pub title: String,
    pub description: String,
    pub affected_component: String,
    pub fix_available: bool,
    pub fix_description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl VulnerabilityScanner {
    pub fn new() -> Self {
        Self {
            scan_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start a vulnerability scan
    pub async fn start_scan(&self, scan_type: String, scan_tool: String) -> uuid::Uuid {
        let scan_id = uuid::Uuid::new_v4();
        let scan_result = VulnerabilityScanResult {
            scan_id,
            scan_type,
            scan_tool,
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: "running".to_string(),
            vulnerabilities: Vec::new(),
        };

        let mut results = self.scan_results.write().await;
        results.push(scan_result);

        scan_id
    }

    /// Complete a vulnerability scan
    pub async fn complete_scan(
        &self,
        scan_id: uuid::Uuid,
        vulnerabilities: Vec<Vulnerability>,
    ) -> Result<(), String> {
        let mut results = self.scan_results.write().await;

        if let Some(scan) = results.iter_mut().find(|s| s.scan_id == scan_id) {
            scan.completed_at = Some(chrono::Utc::now());
            scan.status = "completed".to_string();
            scan.vulnerabilities = vulnerabilities;
            Ok(())
        } else {
            Err("Scan not found".to_string())
        }
    }

    /// Get latest scan results
    pub async fn get_latest_scan_results(
        &self,
        limit: Option<usize>,
    ) -> Vec<VulnerabilityScanResult> {
        let results = self.scan_results.read().await;
        let mut sorted_results = results.clone();
        sorted_results.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        if let Some(limit) = limit {
            sorted_results.truncate(limit);
        }

        sorted_results
    }

    /// Get vulnerability summary
    pub async fn get_vulnerability_summary(&self) -> VulnerabilitySummary {
        let results = self.scan_results.read().await;
        let latest_scan = results
            .iter()
            .filter(|s| s.status == "completed")
            .max_by_key(|s| s.started_at);

        if let Some(scan) = latest_scan {
            let mut summary = VulnerabilitySummary {
                total: scan.vulnerabilities.len(),
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
                info: 0,
                last_scan: Some(scan.started_at),
            };

            for vuln in &scan.vulnerabilities {
                match vuln.severity {
                    VulnerabilitySeverity::Critical => summary.critical += 1,
                    VulnerabilitySeverity::High => summary.high += 1,
                    VulnerabilitySeverity::Medium => summary.medium += 1,
                    VulnerabilitySeverity::Low => summary.low += 1,
                    VulnerabilitySeverity::Info => summary.info += 1,
                }
            }

            summary
        } else {
            VulnerabilitySummary {
                total: 0,
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
                info: 0,
                last_scan: None,
            }
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct VulnerabilitySummary {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
}
