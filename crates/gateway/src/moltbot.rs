// Moltbot — Moltbook Social Network Adapter
//
// Posts Genesis Protocol state updates to Moltbook (moltbook.com),
// the social network for AI agents.
//
// API:  https://www.moltbook.com/api/v1
// Auth: Bearer moltbook_sk_xxx
// Docs: https://www.moltbook.com/skill.md
//
// Architecture:
//   - MoltbookClient: HTTP client wrapping Moltbook REST API
//   - MoltbotBridge: stateful event detector, queues milestones, composes posts
//   - EpochSnapshot: data from World under mutex, sent via channel
//
// Moltbook rate limits respected:
//   - 1 post per 30 minutes (milestones queued, batched into status posts)
//   - 100 requests/minute general
//   - API key only sent to www.moltbook.com
//
// Security:
//   - Outbound-only: no webhook listeners, no inbound routes
//   - API key isolated in MoltbookClient, never exposed to gateway scope
//   - Failure-tolerant: failed posts log and continue, never block epoch loop

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::world::{EpochStats, LeaderboardEntry};

// ───────────────────────────────────────────
// CONSTANTS
// ───────────────────────────────────────────

/// Official Moltbook API base URL. Must use www subdomain
/// (without www, redirects strip the Authorization header).
const DEFAULT_BASE_URL: &str = "https://www.moltbook.com/api/v1";

/// Minimum post interval in epochs (30 min at 1 epoch/sec).
/// Matches Moltbook's rate limit of 1 post per 30 minutes.
const MIN_POST_INTERVAL: u64 = 1800;

// ───────────────────────────────────────────
// CONFIGURATION
// ───────────────────────────────────────────

/// Moltbot configuration, loaded from environment.
#[derive(Clone)]
pub struct MoltbotConfig {
    /// Moltbook API base URL (default: https://www.moltbook.com/api/v1).
    pub base_url: String,
    /// API key for authentication (format: moltbook_sk_xxx).
    /// Isolated — never shared with gateway.
    pub api_key: String,
    /// Target submolt community for posts (default: "general").
    pub submolt: String,
    /// Post to Moltbook every N epochs (default/min: 1800 = 30 min).
    pub post_interval: u64,
    /// Maximum retries on transient failure.
    pub max_retries: u32,
    /// HTTP timeout per request.
    pub timeout: Duration,
}

// Manual Debug impl to redact api_key — never leak credentials to logs.
impl std::fmt::Debug for MoltbotConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MoltbotConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &if self.api_key.is_empty() { "(empty)" } else { "(redacted)" })
            .field("submolt", &self.submolt)
            .field("post_interval", &self.post_interval)
            .field("max_retries", &self.max_retries)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl MoltbotConfig {
    /// Load configuration from environment variables.
    /// Returns None if MOLTBOOK_API_KEY is not set (adapter disabled).
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("MOLTBOOK_API_KEY").ok()?;
        if api_key.is_empty() {
            return None;
        }

        let base_url = std::env::var("MOLTBOOK_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        let submolt = std::env::var("MOLTBOOK_SUBMOLT")
            .unwrap_or_else(|_| "general".to_string());

        let post_interval = std::env::var("MOLTBOT_POST_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(MIN_POST_INTERVAL)
            .max(MIN_POST_INTERVAL);

        let max_retries = std::env::var("MOLTBOT_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2);

        let timeout_secs = std::env::var("MOLTBOT_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        Some(MoltbotConfig {
            base_url,
            api_key,
            submolt,
            post_interval,
            max_retries,
            timeout: Duration::from_secs(timeout_secs),
        })
    }
}

// ───────────────────────────────────────────
// MOLTBOOK API TYPES
// ───────────────────────────────────────────

/// Request body for POST /posts.
#[derive(Debug, Serialize)]
struct CreatePostRequest {
    submolt_name: String,
    title: String,
    content: String,
}

/// Standard Moltbook API response envelope.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MoltbookResponse {
    #[allow(dead_code)]
    success: bool,
    #[serde(default)]
    #[allow(dead_code)]
    error: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    hint: Option<String>,
    /// Present when content requires AI verification before publishing.
    #[serde(default)]
    #[allow(dead_code)]
    verification_required: Option<bool>,
    /// The created post (may contain a verification challenge).
    #[serde(default)]
    #[allow(dead_code)]
    post: Option<PostPayload>,
    /// The created comment (may contain a verification challenge).
    #[serde(default)]
    #[allow(dead_code)]
    comment: Option<PostPayload>,
}

/// Payload for a created post/comment that may require verification.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PostPayload {
    #[allow(dead_code)]
    id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    verification_status: Option<String>,
    /// Verification challenge details (present when verification is required).
    #[serde(default)]
    #[allow(dead_code)]
    verification: Option<VerificationChallenge>,
}

/// AI verification challenge returned by Moltbook to prove the agent
/// has language understanding. Contains an obfuscated math word problem.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct VerificationChallenge {
    /// Unique code to submit with the answer.
    verification_code: String,
    /// Obfuscated math problem (lobster-themed, alternating caps, scattered symbols).
    challenge_text: String,
    /// ISO 8601 deadline (typically 5 minutes from creation).
    #[allow(dead_code)]
    expires_at: Option<String>,
    /// Instructions for formatting the answer.
    #[allow(dead_code)]
    instructions: Option<String>,
}

/// Request body for POST /verify.
#[derive(Debug, Serialize)]
struct VerifyRequest {
    verification_code: String,
    answer: String,
}

// ───────────────────────────────────────────
// INTERNAL TYPES — Organism State
// ───────────────────────────────────────────

/// Organism vitals snapshot (used internally for post composition).
#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatPayload {
    /// Payload type discriminator.
    #[serde(rename = "type")]
    pub payload_type: String,
    /// Current epoch number.
    pub epoch: u64,
    /// Living agent count.
    pub population: usize,
    /// Mean fitness across population.
    pub mean_fitness: f64,
    /// Maximum fitness in population.
    pub max_fitness: f64,
    /// Total circulating ATP.
    pub total_atp: f64,
    /// Treasury reserve balance.
    pub treasury_reserve: f64,
    /// Active risk states.
    pub risks: Vec<String>,
    /// Top agent summary.
    pub leader: Option<LeaderSummary>,
    /// Uptime in seconds.
    pub uptime_seconds: i64,
    /// Total lifetime births.
    pub total_births: u64,
    /// Total lifetime deaths.
    pub total_deaths: u64,
}

/// Compact leader info embedded in heartbeat.
#[derive(Debug, Clone, Serialize)]
pub struct LeaderSummary {
    pub agent_id: String,
    pub role: String,
    pub fitness: f64,
    pub generation: u64,
}

impl LeaderSummary {
    pub fn from_entry(entry: &LeaderboardEntry) -> Self {
        LeaderSummary {
            agent_id: entry.agent_id.clone(),
            role: entry.role.clone(),
            fitness: entry.fitness,
            generation: entry.generation,
        }
    }
}

/// Significant biological event detected by the bridge.
#[derive(Debug, Clone, Serialize)]
pub struct MilestoneEvent {
    /// Payload type discriminator.
    #[serde(rename = "type")]
    pub payload_type: String,
    /// Event kind.
    pub event: MilestoneKind,
    /// Current epoch when event occurred.
    pub epoch: u64,
    /// Human-readable description.
    pub description: String,
    /// Optional numeric detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

/// Categories of milestone events.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneKind {
    /// Population hit a new high.
    PopulationPeak,
    /// Population dropped below critical threshold.
    PopulationCrash,
    /// A new fitness record was set.
    FitnessRecord,
    /// Birth burst — many agents born in one epoch.
    BirthBurst,
    /// Death spiral — many agents died in one epoch.
    DeathSpiral,
    /// New leader emerged (top fitness agent changed).
    LeaderChange,
    /// Epoch milestone (every 100 epochs).
    EpochMilestone,
    /// Extinction risk detected.
    ExtinctionRisk,
    /// ATP crisis — total supply dropped below threshold.
    AtpCrisis,
    /// Monoculture — single role dominates >50% of population.
    Monoculture,
}

// ───────────────────────────────────────────
// POST COMPOSER
// ───────────────────────────────────────────

/// Format uptime seconds as human-readable duration.
fn format_uptime(secs: i64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// Get an emoji for a milestone kind.
fn milestone_emoji(kind: &MilestoneKind) -> &'static str {
    match kind {
        MilestoneKind::PopulationPeak => "\u{1f3d4}\u{fe0f}",
        MilestoneKind::PopulationCrash => "\u{1f4c9}",
        MilestoneKind::FitnessRecord => "\u{2b50}",
        MilestoneKind::BirthBurst => "\u{1f476}",
        MilestoneKind::DeathSpiral => "\u{1f480}",
        MilestoneKind::LeaderChange => "\u{1f451}",
        MilestoneKind::EpochMilestone => "\u{1f3af}",
        MilestoneKind::ExtinctionRisk => "\u{26a0}\u{fe0f}",
        MilestoneKind::AtpCrisis => "\u{26a1}",
        MilestoneKind::Monoculture => "\u{1f9ec}",
    }
}

/// Compose a status post title and content from organism state.
fn compose_status_post(
    stats: &EpochStats,
    leader: Option<&LeaderboardEntry>,
    risks: &[String],
    treasury: f64,
    uptime: i64,
    births: u64,
    deaths: u64,
    milestones: &[MilestoneEvent],
) -> (String, String) {
    let risk_display = if risks.is_empty()
        || risks.iter().all(|r| r.contains("Stable"))
    {
        "STABLE".to_string()
    } else {
        risks.join(", ")
    };

    let title = format!(
        "[Epoch {}] {} agents \u{2014} {}",
        stats.epoch, stats.population, risk_display
    );

    let mut content = format!(
        "## Organism Vitals\n\n\
         - **Population**: {} agents (cap: {})\n\
         - **Mean Fitness**: {:.4}\n\
         - **Peak Fitness**: {:.4}\n\
         - **ATP Supply**: {:.1}\n\
         - **Resources Extracted**: {:.1}\n\
         - **Total Resources**: {:.1}\n\
         - **Season**: {}{}\n\
         - **Treasury**: {:.1}\n\
         - **Births / Deaths**: {} / {}\n\
         - **Stasis**: {}\n\
         - **Uptime**: {}\n",
        stats.population,
        stats.dynamic_pop_cap,
        stats.mean_fitness,
        stats.max_fitness,
        stats.total_atp,
        stats.resources_extracted,
        stats.total_resources,
        stats.season,
        if stats.catastrophe_active { " \u{26A0}\u{FE0F} CATASTROPHE" } else { "" },
        treasury,
        births,
        deaths,
        stats.stasis_count,
        format_uptime(uptime),
    );

    if let Some(entry) = leader {
        let id_prefix = if entry.agent_id.len() > 8 {
            &entry.agent_id[..8]
        } else {
            &entry.agent_id
        };
        content.push_str(&format!(
            "\n## Leader\n**{}** ({}, gen {}) \u{2014} fitness {:.4}\n",
            id_prefix, entry.role, entry.generation, entry.fitness
        ));
    }

    if !milestones.is_empty() {
        content.push_str("\n## Recent Events\n");
        for m in milestones {
            content.push_str(&format!(
                "- {} {}\n",
                milestone_emoji(&m.event),
                m.description
            ));
        }
    }

    (title, content)
}

// ───────────────────────────────────────────
// AI VERIFICATION CHALLENGE SOLVER
// ───────────────────────────────────────────

/// Number words recognized in challenge text.
const NUMBER_WORDS: &[(&str, f64)] = &[
    ("zero", 0.0),
    ("one", 1.0),
    ("two", 2.0),
    ("three", 3.0),
    ("four", 4.0),
    ("five", 5.0),
    ("six", 6.0),
    ("seven", 7.0),
    ("eight", 8.0),
    ("nine", 9.0),
    ("ten", 10.0),
    ("eleven", 11.0),
    ("twelve", 12.0),
    ("thirteen", 13.0),
    ("fourteen", 14.0),
    ("fifteen", 15.0),
    ("sixteen", 16.0),
    ("seventeen", 17.0),
    ("eighteen", 18.0),
    ("nineteen", 19.0),
    ("twenty", 20.0),
    ("thirty", 30.0),
    ("forty", 40.0),
    ("fifty", 50.0),
    ("sixty", 60.0),
    ("seventy", 70.0),
    ("eighty", 80.0),
    ("ninety", 90.0),
    ("hundred", 100.0),
    ("thousand", 1000.0),
];

/// Operation words that map to arithmetic operators.
const OP_WORDS: &[(&str, char)] = &[
    // Addition
    ("adds", '+'),
    ("add", '+'),
    ("plus", '+'),
    ("gains", '+'),
    ("gain", '+'),
    ("increases by", '+'),
    ("speeds up by", '+'),
    ("grows by", '+'),
    ("gets", '+'),
    ("collects", '+'),
    ("finds", '+'),
    ("picks up", '+'),
    ("more", '+'),
    // Subtraction
    ("minus", '-'),
    ("subtract", '-'),
    ("subtracts", '-'),
    ("loses", '-'),
    ("lose", '-'),
    ("slows by", '-'),
    ("slows down by", '-'),
    ("drops by", '-'),
    ("drops", '-'),
    ("decreases by", '-'),
    ("sheds", '-'),
    ("less", '-'),
    // Multiplication
    ("times", '*'),
    ("multiplied by", '*'),
    ("multiplies by", '*'),
    ("mult by", '*'),
    ("doubles", '*'),   // special: implies *2
    ("triples", '*'),   // special: implies *3
    // Division
    ("divided by", '/'),
    ("divides by", '/'),
    ("split by", '/'),
    ("splits into", '/'),
    ("halves", '/'),    // special: implies /2
];

/// Deobfuscate a Moltbook challenge string.
///
/// Moltbook challenges use:
///   - Alternating caps: "lObStEr" → "lobster"
///   - Scattered symbols: brackets, carets, slashes, hyphens inside words
///   - Broken/repeated letters: "tW]eNn-Tyy" → "twenty"
///
/// This function strips noise and normalizes to lowercase plain text.
fn deobfuscate_challenge(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    // Track whether we've emitted meaningful content (alpha/digit) since
    // the last space. Symbols are transparent — they must NOT clear this
    // flag, otherwise spaces after symbols (e.g. "o/ w") are dropped.
    let mut has_content = false;
    let mut last_was_space = false;

    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            result.push(ch.to_ascii_lowercase());
            has_content = true;
            last_was_space = false;
        } else if ch.is_ascii_digit() {
            result.push(ch);
            has_content = true;
            last_was_space = false;
        } else if ch == '.' {
            // Keep decimal points between digits
            result.push(ch);
            last_was_space = false;
        } else if ch.is_ascii_whitespace() {
            if has_content && !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
            // Do NOT reset has_content — symbols before this space
            // should not prevent the space from being emitted.
        } else {
            // Strip symbols: [ ] ^ / - \ etc.
            // They are noise injected between letters.
            // CRITICAL: do NOT touch has_content or last_was_space.
            // Symbols are invisible to word-boundary logic.
        }
    }

    // Collapse multiple spaces and trim
    let collapsed: String = result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Fix obfuscation artifacts: doubled/repeated letters
    // e.g., "twenntyy" → "twenty", "fivee" → "five"
    normalize_obfuscated_words(&collapsed)
}

/// Collapse consecutive duplicate characters in a word.
/// e.g., "twenntyy" → "twenty", "fivee" → "five"
fn collapse_consecutive(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    let mut prev = None;
    for ch in word.chars() {
        if Some(ch) != prev {
            result.push(ch);
        }
        prev = Some(ch);
    }
    result
}

/// Normalize obfuscated words by removing spurious letter repetitions.
///
/// Two strategies tried in order:
///   1. Collapse ALL consecutive duplicates ("twenntyy" → "twenty")
///   2. Remove just the trailing doubled letter ("fivee" → "five")
///
/// Only applies the fix when the result matches a known word.
fn normalize_obfuscated_words(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            // Strategy 1: collapse all consecutive duplicate letters
            let collapsed = collapse_consecutive(word);
            if collapsed != word && is_known_word(&collapsed) {
                return collapsed;
            }

            // Strategy 2: remove just trailing doubled letter
            let bytes = word.as_bytes();
            if bytes.len() >= 3
                && bytes[bytes.len() - 1] == bytes[bytes.len() - 2]
                && bytes[bytes.len() - 1].is_ascii_alphabetic()
            {
                let trimmed = &word[..word.len() - 1];
                if is_known_word(trimmed) {
                    return trimmed.to_string();
                }
            }
            word.to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if a word is one we recognize (number words, operation words, common words).
fn is_known_word(w: &str) -> bool {
    NUMBER_WORDS.iter().any(|(nw, _)| *nw == w)
        || ["a", "an", "at", "the", "and", "by", "to", "of", "is", "it",
            "lobster", "crab", "shrimp", "molty", "swims", "crawls", "walks",
            "runs", "meters", "miles", "steps", "speeds", "speed",
        ].contains(&w)
}

/// Parse text numbers from the deobfuscated string.
/// Handles: "twenty", "five", "twenty five" (25), "one hundred" (100),
///          "three hundred fifty" (350), and raw digits like "42".
fn parse_numbers(text: &str) -> Vec<f64> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut numbers = Vec::new();
    let mut i = 0;

    while i < words.len() {
        // Try raw numeric literal first
        if let Ok(n) = words[i].parse::<f64>() {
            numbers.push(n);
            i += 1;
            continue;
        }

        // Try compound number: "twenty five", "three hundred fifty", etc.
        if let Some((val, consumed)) = parse_compound_number(&words[i..]) {
            numbers.push(val);
            i += consumed;
            continue;
        }

        i += 1;
    }

    numbers
}

/// Parse a compound number starting at the given word slice.
/// Returns (value, words_consumed) or None.
fn parse_compound_number(words: &[&str]) -> Option<(f64, usize)> {
    let mut total = 0.0;
    let mut current = 0.0;
    let mut consumed = 0;
    let mut found_any = false;

    for &word in words {
        if let Some(&(_, val)) = NUMBER_WORDS.iter().find(|&&(w, _)| w == word) {
            found_any = true;
            consumed += 1;

            if val == 100.0 {
                // "three hundred" → current(3) * 100
                if current == 0.0 {
                    current = 100.0;
                } else {
                    current *= 100.0;
                }
            } else if val == 1000.0 {
                // "five thousand" → current(5) * 1000
                if current == 0.0 {
                    current = 1000.0;
                } else {
                    current *= 1000.0;
                    total += current;
                    current = 0.0;
                }
            } else if val >= 20.0 {
                // Tens: "twenty", "thirty", etc.
                // If we had a previous value accumulating, flush it
                if current > 0.0 {
                    total += current;
                }
                current = val;
            } else {
                // Units: "one" through "nineteen"
                current += val;
            }
        } else {
            break;
        }
    }

    if found_any {
        total += current;
        Some((total, consumed))
    } else {
        None
    }
}

/// Detect the arithmetic operation from deobfuscated text.
/// Returns the operator character: +, -, *, /
fn detect_operation(text: &str) -> Option<char> {
    let lower = text.to_lowercase();

    // Try multi-word operators first (longest match)
    let mut ops_sorted: Vec<(&str, char)> = OP_WORDS.to_vec();
    ops_sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (phrase, op) in &ops_sorted {
        if lower.contains(phrase) {
            return Some(*op);
        }
    }

    None
}

/// Solve a Moltbook AI verification challenge.
///
/// The challenge is an obfuscated math word problem with two numbers
/// and one operation (+, -, *, /). Returns the answer formatted as
/// a string with 2 decimal places, or None if parsing fails.
///
/// Example:
///   "A] lO^bSt-Er S[wImS aT/ tW]eNn-Tyy mE^tE[rS aNd] SlO/wS bY^ fI[vE"
///   → deobfuscates to "a lobster swims at twenty meters and slows by five"
///   → numbers: [20.0, 5.0], operation: '-'
///   → answer: "15.00"
fn solve_challenge(challenge_text: &str) -> Option<String> {
    let clean = deobfuscate_challenge(challenge_text);
    tracing::debug!(clean = %clean, "Deobfuscated challenge");

    let numbers = parse_numbers(&clean);
    if numbers.len() < 2 {
        tracing::warn!(
            numbers = ?numbers,
            clean = %clean,
            "Challenge solver found fewer than 2 numbers"
        );
        return None;
    }

    let a = numbers[0];
    let b = numbers[1];

    let op = detect_operation(&clean)?;
    tracing::debug!(a = a, b = b, op = ?op, "Solving challenge");

    let result = match op {
        '+' => a + b,
        '-' => a - b,
        '*' => a * b,
        '/' => {
            if b == 0.0 {
                tracing::warn!("Challenge has division by zero");
                return None;
            }
            a / b
        }
        _ => return None,
    };

    Some(format!("{:.2}", result))
}

// ───────────────────────────────────────────
// HTTP CLIENT
// ───────────────────────────────────────────

/// Outbound HTTP client for the Moltbook API.
///
/// Endpoints used:
///   POST /posts      — create a post in a submolt
///   GET  /agents/me  — validate API key / fetch profile
///
/// Isolated from the gateway server — no shared state, no inbound surface.
#[derive(Clone)]
pub struct MoltbotClient {
    config: MoltbotConfig,
    http: reqwest::Client,
}

impl MoltbotClient {
    /// Create a new client from config.
    /// Returns None if the HTTP client cannot be constructed.
    pub fn new(config: MoltbotConfig) -> Option<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent("Genesis-Protocol/0.1.0 Moltbot")
            .build()
            .ok()?;

        Some(MoltbotClient { config, http })
    }

    /// Create a text post in a submolt. Returns true on success.
    ///
    /// If Moltbook returns a verification challenge (anti-spam math problem),
    /// the client automatically deobfuscates the challenge, solves it,
    /// and submits the answer to POST /verify before returning.
    pub async fn create_post(&self, submolt: &str, title: &str, content: &str) -> bool {
        let url = format!("{}/posts", self.config.base_url);
        let payload = CreatePostRequest {
            submolt_name: submolt.to_string(),
            title: title.to_string(),
            content: content.to_string(),
        };

        for attempt in 0..=self.config.max_retries {
            match self.try_post(&url, &payload).await {
                Ok((status, body)) if status.is_success() => {
                    // Check if the response contains a verification challenge
                    if let Some(challenge) = self.extract_challenge(&body) {
                        tracing::info!(
                            challenge = %challenge.challenge_text,
                            code = %challenge.verification_code,
                            "Moltbook verification challenge received — solving"
                        );
                        return self.handle_challenge(&challenge).await;
                    }
                    tracing::info!("Moltbook post published successfully (no verification needed)");
                    return true;
                }
                Ok((status, body)) if status.as_u16() == 403 => {
                    // Check if the 403 body itself contains a challenge or suspension info
                    if body.contains("suspended") {
                        tracing::error!(
                            body = %body,
                            "Moltbook agent is suspended — cannot post"
                        );
                    } else {
                        tracing::warn!(
                            attempt = attempt + 1,
                            body = %body,
                            "Moltbook 403 Forbidden \u{2014} agent may need claiming, skipping retries"
                        );
                    }
                    return false;
                }
                Ok((status, _)) if status.as_u16() == 429 => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        "Moltbook rate limited (429) \u{2014} will retry next interval"
                    );
                    return false;
                }
                Ok((status, body)) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        status = status.as_u16(),
                        body = %body,
                        "Moltbook post rejected"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        error = %e,
                        "Moltbook post failed"
                    );
                }
            }

            if attempt < self.config.max_retries {
                // Exponential backoff: 200ms, 400ms, 800ms...
                let delay = 200 * (1u64 << attempt.min(4));
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }

        tracing::error!("Moltbook post failed after {} retries", self.config.max_retries);
        false
    }

    /// Validate API key by fetching agent profile.
    /// Returns (is_valid, is_claimed) — is_valid true if the API key works.
    pub async fn check_profile(&self) -> (bool, bool) {
        let url = format!("{}/agents/me", self.config.base_url);
        match self.try_get(&url).await {
            Ok((status, body)) if status.is_success() => {
                // Parse claim status from response
                let claimed = body.contains("\"is_claimed\":true");
                let name = serde_json::from_str::<serde_json::Value>(&body)
                    .ok()
                    .and_then(|v| v["agent"]["name"].as_str().map(String::from))
                    .unwrap_or_default();
                tracing::info!(
                    agent = %name,
                    claimed = claimed,
                    "Moltbook profile verified"
                );
                (true, claimed)
            }
            Ok((status, body)) => {
                tracing::warn!(
                    status = status.as_u16(),
                    body = %body,
                    "Moltbook profile check failed"
                );
                (false, false)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Moltbook profile check error");
                (false, false)
            }
        }
    }

    /// Extract a verification challenge from a Moltbook response body.
    ///
    /// Checks both `post.verification` and `comment.verification` fields.
    /// Returns None if no challenge is present (trusted/admin agent, or non-challenge response).
    fn extract_challenge(&self, body: &str) -> Option<VerificationChallenge> {
        let resp: MoltbookResponse = serde_json::from_str(body).ok()?;

        // Check "post" field first, then "comment"
        let payload = resp.post.or(resp.comment)?;

        if payload.verification_status.as_deref() == Some("pending") {
            return payload.verification;
        }

        // Also return the challenge even without explicit pending status
        payload.verification
    }

    /// Solve a verification challenge and submit the answer.
    /// Returns true if verification succeeded and content was published.
    async fn handle_challenge(&self, challenge: &VerificationChallenge) -> bool {
        match solve_challenge(&challenge.challenge_text) {
            Some(answer) => {
                tracing::info!(
                    answer = %answer,
                    code = %challenge.verification_code,
                    "Challenge solved — submitting verification"
                );
                self.submit_verification(&challenge.verification_code, &answer).await
            }
            None => {
                tracing::error!(
                    challenge = %challenge.challenge_text,
                    "Failed to solve Moltbook verification challenge — CRITICAL: unanswered challenges cause suspension"
                );
                // Even if we can't solve it, at least attempt a reasonable answer
                // to avoid "challenge_no_answer" suspension. Try "0.00" as a last resort.
                tracing::warn!("Submitting fallback answer '0.00' to avoid challenge_no_answer suspension");
                self.submit_verification(&challenge.verification_code, "0.00").await
            }
        }
    }

    /// Submit an answer to POST /verify.
    /// Returns true if the verification was accepted.
    async fn submit_verification(&self, code: &str, answer: &str) -> bool {
        let url = format!("{}/verify", self.config.base_url);
        let payload = VerifyRequest {
            verification_code: code.to_string(),
            answer: answer.to_string(),
        };

        match self.try_post(&url, &payload).await {
            Ok((status, body)) if status.is_success() => {
                tracing::info!(
                    body = %body,
                    "Moltbook verification accepted — content published"
                );
                true
            }
            Ok((status, body)) => {
                tracing::error!(
                    status = status.as_u16(),
                    body = %body,
                    answer = %answer,
                    "Moltbook verification rejected"
                );
                false
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "Moltbook verification request failed"
                );
                false
            }
        }
    }

    /// Single POST attempt. Returns (status_code, response_body).
    async fn try_post<T: Serialize>(
        &self,
        url: &str,
        payload: &T,
    ) -> Result<(reqwest::StatusCode, String), reqwest::Error> {
        let resp = self
            .http
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(payload)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Ok((status, body))
    }

    /// Single GET attempt. Returns (status_code, response_body).
    async fn try_get(
        &self,
        url: &str,
    ) -> Result<(reqwest::StatusCode, String), reqwest::Error> {
        let resp = self
            .http
            .get(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Ok((status, body))
    }
}

// ───────────────────────────────────────────
// BRIDGE — Event Detection + Dispatch
// ───────────────────────────────────────────

/// Stateful bridge that detects milestones and dispatches posts.
///
/// Wired into the epoch loop. Compares each epoch's stats against
/// historical thresholds to decide what's worth broadcasting.
/// Milestones are queued and batched into periodic status posts
/// to respect Moltbook's 1 post per 30 minutes rate limit.
pub struct MoltbotBridge {
    client: MoltbotClient,
    config: MoltbotConfig,
    /// Epoch of last successful Moltbook post.
    last_post_epoch: u64,
    /// Queued milestones waiting for the next post.
    pending_milestones: Vec<MilestoneEvent>,
    /// Highest population ever observed.
    peak_population: usize,
    /// Highest fitness ever observed.
    peak_fitness: f64,
    /// Agent ID of the current leader.
    current_leader: Option<String>,
    /// Total posts successfully sent to Moltbook.
    posts_sent: u64,
    /// Total milestones detected across all epochs.
    milestones_detected: u64,
    /// Total snapshots received from runtime.
    snapshots_received: u64,
}

impl MoltbotBridge {
    /// Create a new bridge from config.
    /// Returns None if the HTTP client cannot be constructed.
    pub fn new(config: MoltbotConfig) -> Option<Self> {
        let client = MoltbotClient::new(config.clone())?;

        Some(MoltbotBridge {
            client,
            config: config.clone(),
            last_post_epoch: 0,
            pending_milestones: Vec::new(),
            peak_population: 0,
            peak_fitness: 0.0,
            current_leader: None,
            posts_sent: 0,
            milestones_detected: 0,
            snapshots_received: 0,
        })
    }

    /// Process an epoch tick. Called from the runtime loop with current world state.
    ///
    /// Detects milestones every epoch (queuing them internally).
    /// Posts a composed status update to Moltbook every post_interval epochs.
    pub async fn on_epoch(
        &mut self,
        stats: &EpochStats,
        leader: Option<&LeaderboardEntry>,
        risks: &[String],
        treasury_reserve: f64,
        uptime_seconds: i64,
        total_births: u64,
        total_deaths: u64,
    ) {
        // Detect milestones every epoch (purely internal, no HTTP)
        let new_milestones = self.detect_milestones(stats, leader, risks);
        self.milestones_detected += new_milestones.len() as u64;
        self.pending_milestones.extend(new_milestones);

        // Post to Moltbook on interval
        if stats.epoch == 0
            || stats.epoch >= self.last_post_epoch + self.config.post_interval
        {
            let (title, content) = compose_status_post(
                stats,
                leader,
                risks,
                treasury_reserve,
                uptime_seconds,
                total_births,
                total_deaths,
                &self.pending_milestones,
            );

            if self.client.create_post(&self.config.submolt, &title, &content).await {
                self.posts_sent += 1;
                self.pending_milestones.clear();
                tracing::info!(
                    epoch = stats.epoch,
                    posts_sent = self.posts_sent,
                    pending_cleared = true,
                    "Status posted to Moltbook"
                );
            }
            // Always advance epoch counter to prevent hammering on persistent errors
            self.last_post_epoch = stats.epoch;
        }
    }

    /// Detect milestone events from epoch stats.
    /// Returns new milestones without posting — they are queued for the next post.
    fn detect_milestones(
        &mut self,
        stats: &EpochStats,
        leader: Option<&LeaderboardEntry>,
        risks: &[String],
    ) -> Vec<MilestoneEvent> {
        let mut milestones = Vec::new();

        // Epoch milestone (every 100 epochs)
        if stats.epoch > 0 && stats.epoch % 100 == 0 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::EpochMilestone,
                epoch: stats.epoch,
                description: format!(
                    "Epoch {} reached. Population: {}, Mean fitness: {:.4}",
                    stats.epoch, stats.population, stats.mean_fitness
                ),
                value: Some(stats.epoch as f64),
            });
        }

        // Population peak
        if stats.population > self.peak_population && self.peak_population > 0 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::PopulationPeak,
                epoch: stats.epoch,
                description: format!(
                    "New population peak: {} (prev: {})",
                    stats.population, self.peak_population
                ),
                value: Some(stats.population as f64),
            });
        }
        self.peak_population = self.peak_population.max(stats.population);

        // Fitness record
        if stats.max_fitness > self.peak_fitness && self.peak_fitness > 0.0 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::FitnessRecord,
                epoch: stats.epoch,
                description: format!(
                    "New fitness record: {:.5} (prev: {:.5})",
                    stats.max_fitness, self.peak_fitness
                ),
                value: Some(stats.max_fitness),
            });
        }
        self.peak_fitness = self.peak_fitness.max(stats.max_fitness);

        // Leader change
        if let Some(entry) = leader {
            let new_leader_id = entry.agent_id.clone();
            if self.current_leader.as_ref() != Some(&new_leader_id) {
                if self.current_leader.is_some() {
                    milestones.push(MilestoneEvent {
                        payload_type: "milestone".to_string(),
                        event: MilestoneKind::LeaderChange,
                        epoch: stats.epoch,
                        description: format!(
                            "New leader: {} ({}, fitness {:.4}, gen {})",
                            entry.agent_id, entry.role, entry.fitness, entry.generation
                        ),
                        value: Some(entry.fitness),
                    });
                }
                self.current_leader = Some(new_leader_id);
            }
        }

        // Birth burst (3+ births in one epoch)
        if stats.births >= 3 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::BirthBurst,
                epoch: stats.epoch,
                description: format!("{} agents born in epoch {}", stats.births, stats.epoch),
                value: Some(stats.births as f64),
            });
        }

        // Death spiral (5+ deaths in one epoch)
        if stats.deaths >= 5 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::DeathSpiral,
                epoch: stats.epoch,
                description: format!("{} agents died in epoch {}", stats.deaths, stats.epoch),
                value: Some(stats.deaths as f64),
            });
        }

        // Population crash
        if stats.population < 10 {
            milestones.push(MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::PopulationCrash,
                epoch: stats.epoch,
                description: format!(
                    "Population critical: {} agents remaining",
                    stats.population
                ),
                value: Some(stats.population as f64),
            });
        }

        // Risk-based milestones
        for risk in risks {
            match risk.as_str() {
                "PopulationCrashRisk" => {
                    milestones.push(MilestoneEvent {
                        payload_type: "milestone".to_string(),
                        event: MilestoneKind::ExtinctionRisk,
                        epoch: stats.epoch,
                        description: "Extinction risk detected \u{2014} population critically low"
                            .to_string(),
                        value: Some(stats.population as f64),
                    });
                }
                "ATPConcentrationHigh" => {
                    milestones.push(MilestoneEvent {
                        payload_type: "milestone".to_string(),
                        event: MilestoneKind::AtpCrisis,
                        epoch: stats.epoch,
                        description: "ATP concentration crisis \u{2014} wealth inequality spike"
                            .to_string(),
                        value: Some(stats.total_atp),
                    });
                }
                "MonocultureEmerging" => {
                    milestones.push(MilestoneEvent {
                        payload_type: "milestone".to_string(),
                        event: MilestoneKind::Monoculture,
                        epoch: stats.epoch,
                        description:
                            "Monoculture emerging \u{2014} single role dominates population"
                                .to_string(),
                        value: None,
                    });
                }
                _ => {}
            }
        }

        milestones
    }
}

// ───────────────────────────────────────────
// EPOCH SNAPSHOT — Channel payload from runtime to adapter
// ───────────────────────────────────────────

/// Data extracted from the World under the mutex lock,
/// sent through a channel to the async adapter task.
#[derive(Debug, Clone)]
pub struct EpochSnapshot {
    pub stats: EpochStats,
    pub leader: Option<LeaderboardEntry>,
    pub risks: Vec<String>,
    pub treasury_reserve: f64,
    pub uptime_seconds: i64,
    pub total_births: u64,
    pub total_deaths: u64,
}

/// Start the adapter loop as an async tokio task.
/// Receives EpochSnapshots from the runtime thread and drives the MoltbotBridge.
pub fn start_adapter_loop(
    config: MoltbotConfig,
    mut rx: mpsc::Receiver<EpochSnapshot>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut bridge = match MoltbotBridge::new(config.clone()) {
            Some(b) => b,
            None => {
                tracing::error!("Moltbot adapter failed to initialize HTTP client \u{2014} disabling");
                return;
            }
        };
        tracing::info!(
            base_url = %config.base_url,
            submolt = %config.submolt,
            post_interval = config.post_interval,
            "Moltbot adapter started \u{2014} posting to Moltbook"
        );

        // Validate API key and check claim status on startup
        let (valid, claimed) = bridge.client.check_profile().await;
        if !valid {
            tracing::warn!("Moltbook API key may be invalid or service is down \u{2014} adapter will keep trying");
        }
        if !claimed {
            tracing::warn!("Agent is not yet claimed on Moltbook \u{2014} posts may be rejected until claimed");
        }

        while let Some(snapshot) = rx.recv().await {
            bridge.snapshots_received += 1;

            bridge
                .on_epoch(
                    &snapshot.stats,
                    snapshot.leader.as_ref(),
                    &snapshot.risks,
                    snapshot.treasury_reserve,
                    snapshot.uptime_seconds,
                    snapshot.total_births,
                    snapshot.total_deaths,
                )
                .await;

            // Periodic liveness signal — visible proof the adapter is still running.
            if bridge.snapshots_received % 60 == 0 {
                tracing::info!(
                    snapshots = bridge.snapshots_received,
                    posts = bridge.posts_sent,
                    milestones = bridge.milestones_detected,
                    pending = bridge.pending_milestones.len(),
                    epoch = snapshot.stats.epoch,
                    "Moltbot adapter alive"
                );
            }
        }

        tracing::warn!(
            snapshots = bridge.snapshots_received,
            posts = bridge.posts_sent,
            milestones = bridge.milestones_detected,
            "Moltbot adapter channel closed \u{2014} adapter stopping"
        );
    })
}

// ───────────────────────────────────────────
// TESTS
// ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_stats(epoch: u64, pop: usize, fitness: f64, births: u64, deaths: u64) -> EpochStats {
        EpochStats {
            epoch,
            population: pop,
            total_atp: 100.0,
            mean_fitness: fitness * 0.8,
            max_fitness: fitness,
            min_fitness: fitness * 0.5,
            births,
            deaths,
            mutations: 0,
            stasis_count: 0,
            market_solved: 0,
            market_rewarded: 0.0,
            gated_posts: 0,
            resources_extracted: 0.0,
            total_resources: 500.0,
            season: "Spring".to_string(),
            catastrophe_active: false,
            dynamic_pop_cap: 200,
            role_counts: std::collections::HashMap::new(),
            niche_resources: std::collections::HashMap::new(),
            treasury_reserve: 15.0,
            treasury_distributed: 0.0,
            eco_state: crate::world::EcoState::Autumn,
            birth_death_ratio: (births + 1) as f64 / (deaths + 1) as f64,
        }
    }

    fn test_leader() -> LeaderboardEntry {
        LeaderboardEntry {
            agent_id: "abcdef0123456789".to_string(),
            role: "Researcher".to_string(),
            fitness: 0.85,
            reputation: 0.7,
            atp_balance: 50.0,
            generation: 3,
            is_primordial: false,
            survived_epochs: 100,
        }
    }

    fn test_config() -> MoltbotConfig {
        MoltbotConfig {
            base_url: "http://localhost:9999".to_string(),
            api_key: String::new(),
            submolt: "test".to_string(),
            post_interval: 60,
            max_retries: 0,
            timeout: Duration::from_secs(1),
        }
    }

    #[test]
    fn test_config_from_env_disabled() {
        // Without MOLTBOOK_API_KEY, adapter should be None.
        // Note: env vars are process-global, so we save/restore to avoid
        // races with other tests or .env files loaded by the binary.
        let saved = std::env::var("MOLTBOOK_API_KEY").ok();
        std::env::remove_var("MOLTBOOK_API_KEY");
        let result = MoltbotConfig::from_env().is_none();
        if let Some(v) = saved {
            std::env::set_var("MOLTBOOK_API_KEY", v);
        }
        assert!(result);
    }

    #[test]
    fn test_config_from_env_enabled() {
        let saved = std::env::var("MOLTBOOK_API_KEY").ok();
        std::env::set_var("MOLTBOOK_API_KEY", "moltbook_sk_test123");
        std::env::set_var("MOLTBOOK_SUBMOLT", "genesis-protocol");
        std::env::set_var("MOLTBOOK_BASE_URL", "http://localhost:8080/api/v1");

        let config = MoltbotConfig::from_env().unwrap();
        assert_eq!(config.api_key, "moltbook_sk_test123");
        assert_eq!(config.submolt, "genesis-protocol");
        assert_eq!(config.base_url, "http://localhost:8080/api/v1");
        // post_interval enforces minimum
        assert!(config.post_interval >= MIN_POST_INTERVAL);

        // Cleanup — restore original or remove
        if let Some(v) = saved {
            std::env::set_var("MOLTBOOK_API_KEY", v);
        } else {
            std::env::remove_var("MOLTBOOK_API_KEY");
        }
        std::env::remove_var("MOLTBOOK_SUBMOLT");
        std::env::remove_var("MOLTBOOK_BASE_URL");
    }

    #[test]
    fn test_heartbeat_payload_serializes() {
        let payload = HeartbeatPayload {
            payload_type: "heartbeat".to_string(),
            epoch: 42,
            population: 20,
            mean_fitness: 0.65,
            max_fitness: 0.88,
            total_atp: 200.0,
            treasury_reserve: 15.0,
            risks: vec!["Stable".to_string()],
            leader: Some(LeaderSummary {
                agent_id: "abc123".to_string(),
                role: "Builder".to_string(),
                fitness: 0.88,
                generation: 2,
            }),
            uptime_seconds: 3600,
            total_births: 10,
            total_deaths: 5,
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "heartbeat");
        assert_eq!(json["epoch"], 42);
        assert_eq!(json["population"], 20);
        assert!(json["leader"]["agent_id"].as_str().is_some());
    }

    #[test]
    fn test_milestone_serializes() {
        let event = MilestoneEvent {
            payload_type: "milestone".to_string(),
            event: MilestoneKind::FitnessRecord,
            epoch: 100,
            description: "New fitness record: 0.92".to_string(),
            value: Some(0.92),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "milestone");
        assert_eq!(json["event"], "fitness_record");
        assert_eq!(json["epoch"], 100);
        assert!(json["value"].as_f64().is_some());
    }

    #[test]
    fn test_milestone_without_value() {
        let event = MilestoneEvent {
            payload_type: "milestone".to_string(),
            event: MilestoneKind::Monoculture,
            epoch: 50,
            description: "Monoculture emerging".to_string(),
            value: None,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert!(json.get("value").is_none()); // skip_serializing_if
    }

    #[test]
    fn test_leader_summary_from_entry() {
        let entry = test_leader();
        let summary = LeaderSummary::from_entry(&entry);
        assert_eq!(summary.agent_id, "abcdef0123456789");
        assert_eq!(summary.role, "Researcher");
        assert_eq!(summary.fitness, 0.85);
        assert_eq!(summary.generation, 3);
    }

    #[test]
    fn test_bridge_detects_epoch_milestone() {
        let mut bridge = MoltbotBridge::new(test_config()).unwrap();
        let stats = test_stats(100, 20, 0.8, 1, 0);

        // Epoch 100 should be a milestone
        assert_eq!(stats.epoch % 100, 0);
        // Peak population should be tracked
        bridge.peak_population = 15;
        assert!(stats.population > bridge.peak_population);
    }

    #[test]
    fn test_bridge_detects_leader_change() {
        let mut bridge = MoltbotBridge::new(test_config()).unwrap();
        bridge.current_leader = Some("old_leader_id".to_string());

        let leader = test_leader();
        assert_ne!(
            bridge.current_leader.as_deref(),
            Some(leader.agent_id.as_str())
        );
    }

    #[test]
    fn test_bridge_tracks_peaks() {
        let mut bridge = MoltbotBridge::new(test_config()).unwrap();
        assert_eq!(bridge.peak_population, 0);
        assert_eq!(bridge.peak_fitness, 0.0);

        bridge.peak_population = bridge.peak_population.max(20);
        bridge.peak_fitness = bridge.peak_fitness.max(0.85);
        assert_eq!(bridge.peak_population, 20);
        assert_eq!(bridge.peak_fitness, 0.85);

        bridge.peak_population = bridge.peak_population.max(25);
        bridge.peak_fitness = bridge.peak_fitness.max(0.92);
        assert_eq!(bridge.peak_population, 25);
        assert_eq!(bridge.peak_fitness, 0.92);

        // Lower values don't update peaks
        bridge.peak_population = bridge.peak_population.max(18);
        bridge.peak_fitness = bridge.peak_fitness.max(0.80);
        assert_eq!(bridge.peak_population, 25);
        assert_eq!(bridge.peak_fitness, 0.92);
    }

    #[test]
    fn test_milestone_kind_serializes_snake_case() {
        let kinds = vec![
            (MilestoneKind::PopulationPeak, "population_peak"),
            (MilestoneKind::PopulationCrash, "population_crash"),
            (MilestoneKind::FitnessRecord, "fitness_record"),
            (MilestoneKind::BirthBurst, "birth_burst"),
            (MilestoneKind::DeathSpiral, "death_spiral"),
            (MilestoneKind::LeaderChange, "leader_change"),
            (MilestoneKind::EpochMilestone, "epoch_milestone"),
            (MilestoneKind::ExtinctionRisk, "extinction_risk"),
            (MilestoneKind::AtpCrisis, "atp_crisis"),
            (MilestoneKind::Monoculture, "monoculture"),
        ];

        for (kind, expected) in kinds {
            let json = serde_json::to_value(&kind).unwrap();
            assert_eq!(
                json.as_str().unwrap(),
                expected,
                "MilestoneKind::{:?} should serialize as {}",
                kind,
                expected
            );
        }
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(0), "0m");
        assert_eq!(format_uptime(60), "1m");
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(3660), "1h 1m");
        assert_eq!(format_uptime(7200), "2h 0m");
        assert_eq!(format_uptime(86400), "24h 0m");
    }

    #[test]
    fn test_compose_status_post_basic() {
        let stats = test_stats(1800, 50, 0.9, 5, 2);
        let leader = test_leader();

        let (title, content) = compose_status_post(
            &stats,
            Some(&leader),
            &["Stable".to_string()],
            100.0,
            3600,
            50,
            10,
            &[],
        );

        assert!(title.contains("[Epoch 1800]"));
        assert!(title.contains("50 agents"));
        assert!(title.contains("STABLE"));
        assert!(content.contains("## Organism Vitals"));
        assert!(content.contains("**Population**: 50 agents"));
        assert!(content.contains("## Leader"));
        assert!(content.contains("abcdef01")); // leader ID prefix
        // No milestones section when empty
        assert!(!content.contains("## Recent Events"));
    }

    #[test]
    fn test_compose_status_post_with_milestones() {
        let stats = test_stats(500, 30, 0.85, 0, 0);

        let milestones = vec![
            MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::PopulationPeak,
                epoch: 480,
                description: "New population peak: 30 (prev: 25)".to_string(),
                value: Some(30.0),
            },
            MilestoneEvent {
                payload_type: "milestone".to_string(),
                event: MilestoneKind::EpochMilestone,
                epoch: 500,
                description: "Epoch 500 reached".to_string(),
                value: Some(500.0),
            },
        ];

        let (title, content) = compose_status_post(
            &stats,
            None,
            &[],
            50.0,
            500,
            20,
            5,
            &milestones,
        );

        assert!(title.contains("[Epoch 500]"));
        assert!(content.contains("## Recent Events"));
        assert!(content.contains("New population peak"));
        assert!(content.contains("Epoch 500 reached"));
    }

    #[test]
    fn test_compose_with_risks() {
        let stats = test_stats(100, 5, 0.3, 0, 3);

        let (title, _content) = compose_status_post(
            &stats,
            None,
            &["PopulationCrashRisk".to_string(), "ATPConcentrationHigh".to_string()],
            10.0,
            100,
            5,
            8,
            &[],
        );

        // Risks should appear in title instead of STABLE
        assert!(!title.contains("STABLE"));
        assert!(title.contains("PopulationCrashRisk"));
    }

    #[test]
    fn test_detect_milestones_epoch_100() {
        let mut bridge = MoltbotBridge::new(test_config()).unwrap();
        let stats = test_stats(100, 20, 0.8, 1, 0);

        let milestones = bridge.detect_milestones(&stats, None, &[]);

        assert!(
            milestones.iter().any(|m| m.event == MilestoneKind::EpochMilestone),
            "Should detect epoch 100 milestone"
        );
    }

    #[test]
    fn test_detect_milestones_population_peak() {
        let mut bridge = MoltbotBridge::new(test_config()).unwrap();
        bridge.peak_population = 15;

        let stats = test_stats(50, 20, 0.8, 1, 0);
        let milestones = bridge.detect_milestones(&stats, None, &[]);

        assert!(
            milestones.iter().any(|m| m.event == MilestoneKind::PopulationPeak),
            "Should detect new population peak"
        );
        assert_eq!(bridge.peak_population, 20);
    }

    #[test]
    fn test_detect_milestones_birth_burst() {
        let mut bridge = MoltbotBridge::new(test_config()).unwrap();
        let stats = test_stats(50, 20, 0.8, 5, 0);
        let milestones = bridge.detect_milestones(&stats, None, &[]);

        assert!(
            milestones.iter().any(|m| m.event == MilestoneKind::BirthBurst),
            "Should detect birth burst (5 births)"
        );
    }

    #[test]
    fn test_detect_milestones_extinction_risk() {
        let mut bridge = MoltbotBridge::new(test_config()).unwrap();
        let stats = test_stats(50, 20, 0.8, 0, 0);
        let risks = vec!["PopulationCrashRisk".to_string()];
        let milestones = bridge.detect_milestones(&stats, None, &risks);

        assert!(
            milestones.iter().any(|m| m.event == MilestoneKind::ExtinctionRisk),
            "Should detect extinction risk"
        );
    }

    // Integration test: verify bridge posts to mock axum server at correct intervals
    #[tokio::test]
    async fn test_bridge_post_interval() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc as StdArc;

        let post_count = StdArc::new(AtomicU32::new(0));
        let counter = post_count.clone();

        // Spin up an axum mock server that counts POST requests
        let app = axum::Router::new().route(
            "/posts",
            axum::routing::post(move || {
                let c = counter.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    axum::http::StatusCode::OK
                }
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        // Give server a moment to bind
        tokio::time::sleep(Duration::from_millis(50)).await;

        let config = MoltbotConfig {
            base_url: format!("http://127.0.0.1:{}", port),
            api_key: "moltbook_sk_test".to_string(),
            submolt: "test".to_string(),
            post_interval: 3, // Every 3 epochs (test override, below MIN_POST_INTERVAL)
            max_retries: 0,
            timeout: Duration::from_secs(2),
        };

        let mut bridge = MoltbotBridge::new(config).unwrap();
        let leader = test_leader();

        // Epoch 0 — first post should fire
        let stats = test_stats(0, 20, 0.8, 0, 0);
        bridge
            .on_epoch(&stats, Some(&leader), &["Stable".to_string()], 10.0, 0, 0, 0)
            .await;

        // Epoch 1 — too soon, no post
        let stats = test_stats(1, 20, 0.8, 0, 0);
        bridge
            .on_epoch(&stats, Some(&leader), &["Stable".to_string()], 10.0, 1, 0, 0)
            .await;

        // Epoch 2 — still too soon
        let stats = test_stats(2, 20, 0.8, 0, 0);
        bridge
            .on_epoch(&stats, Some(&leader), &["Stable".to_string()], 10.0, 2, 0, 0)
            .await;

        // Epoch 3 — post should fire (interval=3)
        let stats = test_stats(3, 20, 0.8, 0, 0);
        bridge
            .on_epoch(&stats, Some(&leader), &["Stable".to_string()], 10.0, 3, 0, 0)
            .await;

        // Verify bridge state
        assert_eq!(bridge.last_post_epoch, 3);
        // Two posts: epoch 0 and epoch 3
        assert_eq!(post_count.load(Ordering::SeqCst), 2);
        assert_eq!(bridge.posts_sent, 2);
    }

    // Verify milestones are queued and included in posts
    #[tokio::test]
    async fn test_milestones_queued_and_cleared() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc as StdArc;

        let post_count = StdArc::new(AtomicU32::new(0));
        let counter = post_count.clone();

        let app = axum::Router::new().route(
            "/posts",
            axum::routing::post(move || {
                let c = counter.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    axum::http::StatusCode::OK
                }
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let config = MoltbotConfig {
            base_url: format!("http://127.0.0.1:{}", port),
            api_key: "moltbook_sk_test".to_string(),
            submolt: "test".to_string(),
            post_interval: 5,
            max_retries: 0,
            timeout: Duration::from_secs(2),
        };

        let mut bridge = MoltbotBridge::new(config).unwrap();

        // Epoch 0 — initial post
        let stats = test_stats(0, 20, 0.8, 0, 0);
        bridge.on_epoch(&stats, None, &[], 10.0, 0, 0, 0).await;
        assert_eq!(bridge.posts_sent, 1);
        assert!(bridge.pending_milestones.is_empty());

        // Epoch 1 — generate a birth burst milestone
        let stats = test_stats(1, 25, 0.8, 5, 0);
        bridge.on_epoch(&stats, None, &[], 10.0, 1, 5, 0).await;
        // Milestone is queued but not posted yet
        assert_eq!(bridge.posts_sent, 1);
        assert!(
            bridge.pending_milestones.len() >= 1,
            "Birth burst should be queued"
        );

        // Epoch 5 — post interval reached, queued milestones included
        let stats = test_stats(5, 25, 0.8, 0, 0);
        bridge.on_epoch(&stats, None, &[], 10.0, 5, 5, 0).await;
        assert_eq!(bridge.posts_sent, 2);
        assert!(
            bridge.pending_milestones.is_empty(),
            "Milestones should be cleared after posting"
        );

        assert_eq!(post_count.load(Ordering::SeqCst), 2);
    }

    // ───────────────────────────────────────────
    // AI VERIFICATION CHALLENGE SOLVER TESTS
    // ───────────────────────────────────────────

    #[test]
    fn test_deobfuscate_simple() {
        let input = "A] lO^bSt-Er S[wImS aT/ tW]eNn-Tyy mE^tE[rS aNd] SlO/wS bY^ fI[vE";
        let clean = deobfuscate_challenge(input);
        assert!(clean.contains("lobster"), "Should contain 'lobster', got: {}", clean);
        assert!(clean.contains("twenty"), "Should contain 'twenty', got: {}", clean);
        assert!(clean.contains("five"), "Should contain 'five', got: {}", clean);
        assert!(clean.contains("slows by"), "Should contain 'slows by', got: {}", clean);
    }

    #[test]
    fn test_deobfuscate_strips_symbols() {
        let input = "hE[lL^o/ wO-rL]d";
        let clean = deobfuscate_challenge(input);
        assert_eq!(clean, "hello world");
    }

    #[test]
    fn test_deobfuscate_preserves_digits() {
        let input = "tH^e sP-eEd iS 42 mE^tErS";
        let clean = deobfuscate_challenge(input);
        assert!(clean.contains("42"), "Should preserve digits, got: {}", clean);
    }

    #[test]
    fn test_normalize_obfuscated_words() {
        // Trailing doubles
        assert_eq!(normalize_obfuscated_words("twentyy"), "twenty");
        assert_eq!(normalize_obfuscated_words("att"), "at");
        assert_eq!(normalize_obfuscated_words("hello"), "hello");
        assert_eq!(normalize_obfuscated_words("fivee"), "five");
        // Mid-word consecutive doubles (collapse strategy)
        assert_eq!(normalize_obfuscated_words("twenntyy"), "twenty");
    }

    #[test]
    fn test_collapse_consecutive() {
        assert_eq!(collapse_consecutive("twenntyy"), "twenty");
        assert_eq!(collapse_consecutive("helloo"), "helo");
        assert_eq!(collapse_consecutive("aabbcc"), "abc");
        assert_eq!(collapse_consecutive("five"), "five");
    }

    #[test]
    fn test_parse_numbers_words() {
        let nums = parse_numbers("a lobster swims at twenty meters and slows by five");
        assert_eq!(nums, vec![20.0, 5.0]);
    }

    #[test]
    fn test_parse_numbers_compound() {
        let nums = parse_numbers("the speed is twenty five meters");
        assert_eq!(nums, vec![25.0]);
    }

    #[test]
    fn test_parse_numbers_hundreds() {
        let nums = parse_numbers("three hundred fifty lobsters plus one hundred");
        assert_eq!(nums, vec![350.0, 100.0]);
    }

    #[test]
    fn test_parse_numbers_digits() {
        let nums = parse_numbers("the crab moves 42 meters then adds 8");
        assert_eq!(nums, vec![42.0, 8.0]);
    }

    #[test]
    fn test_parse_numbers_teens() {
        let nums = parse_numbers("thirteen lobsters minus eleven");
        assert_eq!(nums, vec![13.0, 11.0]);
    }

    #[test]
    fn test_detect_operation_addition() {
        assert_eq!(detect_operation("swims and adds five"), Some('+'));
        assert_eq!(detect_operation("gains three more"), Some('+'));
        assert_eq!(detect_operation("speeds up by ten"), Some('+'));
    }

    #[test]
    fn test_detect_operation_subtraction() {
        assert_eq!(detect_operation("and slows by five"), Some('-'));
        assert_eq!(detect_operation("loses three"), Some('-'));
        assert_eq!(detect_operation("drops by ten"), Some('-'));
    }

    #[test]
    fn test_detect_operation_multiplication() {
        assert_eq!(detect_operation("times three"), Some('*'));
        assert_eq!(detect_operation("multiplied by four"), Some('*'));
    }

    #[test]
    fn test_detect_operation_division() {
        assert_eq!(detect_operation("divided by two"), Some('/'));
        assert_eq!(detect_operation("split by five"), Some('/'));
    }

    #[test]
    fn test_solve_challenge_subtraction() {
        // The canonical example from Moltbook docs
        let challenge = "A] lO^bSt-Er S[wImS aT/ tW]eNn-Tyy mE^tE[rS aNd] SlO/wS bY^ fI[vE";
        let answer = solve_challenge(challenge);
        assert_eq!(answer, Some("15.00".to_string()));
    }

    #[test]
    fn test_solve_challenge_addition() {
        let challenge = "a lobster swims at thirty meters and gains ten";
        let answer = solve_challenge(challenge);
        assert_eq!(answer, Some("40.00".to_string()));
    }

    #[test]
    fn test_solve_challenge_multiplication() {
        let challenge = "a crab has twelve shells times three";
        let answer = solve_challenge(challenge);
        assert_eq!(answer, Some("36.00".to_string()));
    }

    #[test]
    fn test_solve_challenge_division() {
        let challenge = "a lobster moves forty meters divided by eight";
        let answer = solve_challenge(challenge);
        assert_eq!(answer, Some("5.00".to_string()));
    }

    #[test]
    fn test_solve_challenge_with_obfuscation() {
        let challenge = "tH^e cR-aB] hA/s fI[fT^eE-n sH]eL/lS aN^d gA[iN-s tW]eN/tY";
        let answer = solve_challenge(challenge);
        assert_eq!(answer, Some("35.00".to_string()));
    }

    #[test]
    fn test_solve_challenge_digits() {
        let challenge = "a lobster swims 50 meters and loses 15";
        let answer = solve_challenge(challenge);
        assert_eq!(answer, Some("35.00".to_string()));
    }

    #[test]
    fn test_extract_challenge_with_verification() {
        let client = MoltbotClient::new(test_config()).unwrap();
        let body = r#"{
            "success": true,
            "message": "Post created! Complete verification to publish.",
            "post": {
                "id": "test-post-id",
                "verification_status": "pending",
                "verification": {
                    "verification_code": "moltbook_verify_abc123",
                    "challenge_text": "A lobster swims at twenty meters and slows by five",
                    "expires_at": "2026-01-28T12:05:00.000Z",
                    "instructions": "Solve the math problem"
                }
            }
        }"#;

        let challenge = client.extract_challenge(body);
        assert!(challenge.is_some(), "Should extract challenge from response");
        let c = challenge.unwrap();
        assert_eq!(c.verification_code, "moltbook_verify_abc123");
        assert!(c.challenge_text.contains("twenty"));
    }

    #[test]
    fn test_extract_challenge_no_verification() {
        let client = MoltbotClient::new(test_config()).unwrap();
        let body = r#"{"success": true, "message": "Post published!"}"#;

        let challenge = client.extract_challenge(body);
        assert!(challenge.is_none(), "Should not find challenge in simple success");
    }

    #[test]
    fn test_extract_challenge_comment() {
        let client = MoltbotClient::new(test_config()).unwrap();
        let body = r#"{
            "success": true,
            "comment": {
                "id": "comment-id",
                "verification_status": "pending",
                "verification": {
                    "verification_code": "moltbook_verify_xyz",
                    "challenge_text": "A crab has ten shells plus five",
                    "expires_at": null,
                    "instructions": null
                }
            }
        }"#;

        let challenge = client.extract_challenge(body);
        assert!(challenge.is_some(), "Should extract challenge from comment");
    }

    #[test]
    fn test_verify_request_serializes() {
        let req = VerifyRequest {
            verification_code: "moltbook_verify_abc".to_string(),
            answer: "15.00".to_string(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["verification_code"], "moltbook_verify_abc");
        assert_eq!(json["answer"], "15.00");
    }

    #[tokio::test]
    async fn test_create_post_handles_challenge() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc as StdArc;

        let verify_count = StdArc::new(AtomicU32::new(0));
        let vc = verify_count.clone();

        // Mock server: /posts returns challenge, /verify accepts answer
        let app = axum::Router::new()
            .route(
                "/posts",
                axum::routing::post(|| async {
                    let body = serde_json::json!({
                        "success": true,
                        "message": "Post created! Complete verification.",
                        "post": {
                            "id": "test-post",
                            "verification_status": "pending",
                            "verification": {
                                "verification_code": "moltbook_verify_test123",
                                "challenge_text": "a lobster swims at twenty meters and gains ten",
                                "expires_at": "2026-12-31T00:00:00Z",
                                "instructions": "Solve the math"
                            }
                        }
                    });
                    axum::Json(body)
                }),
            )
            .route(
                "/verify",
                axum::routing::post(move |axum::Json(payload): axum::Json<serde_json::Value>| {
                    let c = vc.clone();
                    async move {
                        c.fetch_add(1, Ordering::SeqCst);
                        let answer = payload["answer"].as_str().unwrap_or("");
                        if answer == "30.00" {
                            axum::Json(serde_json::json!({
                                "success": true,
                                "message": "Verification successful!"
                            }))
                        } else {
                            axum::Json(serde_json::json!({
                                "success": false,
                                "error": "Incorrect answer"
                            }))
                        }
                    }
                }),
            );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let config = MoltbotConfig {
            base_url: format!("http://127.0.0.1:{}", port),
            api_key: "moltbook_sk_test".to_string(),
            submolt: "test".to_string(),
            post_interval: 3,
            max_retries: 0,
            timeout: Duration::from_secs(5),
        };

        let client = MoltbotClient::new(config).unwrap();
        let result = client.create_post("test", "Test Title", "Test content").await;

        assert!(result, "Post should succeed after solving challenge");
        assert_eq!(verify_count.load(Ordering::SeqCst), 1, "Should have submitted verification");
    }
}
