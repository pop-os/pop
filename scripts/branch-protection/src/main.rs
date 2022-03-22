use std::collections::HashMap;
use reqwest::blocking::Client;

// Need to maintain unrecognized fields? For future?
// `contexts` deprecated in favor of `checks`?

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Check {
    context: String,
    app_id: Option<String>, // XXX?
    #[serde(flatten)] _other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StatusChecks {
    strict: bool,
    contexts: Vec<String>,
    checks: Vec<Check>,
    #[serde(flatten)] _other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PullRequestReviews {
    dismiss_stale_reviews: bool,
    // XXX dismissal_restrictions
    require_code_owner_reviews: bool,
    required_approving_review_count: u8,
    #[serde(flatten)] _other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Enabled {
    enabled: bool,
    // XXX
    #[serde(flatten)] _other: HashMap<String, serde_json::Value>,
}

// omit urls
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Protection {
    required_status_checks: StatusChecks,
    required_pull_request_reviews: PullRequestReviews,
    required_signatures: Enabled,
    enforce_admins: Enabled,
    required_linear_history: Enabled,
    allow_force_pushes: Enabled,
    allow_deletions: Enabled,
    required_conversation_resolution: Enabled,
    #[serde(flatten)] _other: HashMap<String, serde_json::Value>,
}

fn protection_url(repo: &str, branch: &str) -> String {
    format!(
        "https://api.github.com/repos/pop-os/{}/branches/{}/protection",
        repo, branch
    )
}

fn main() {
    let client = Client::new();

    let url = protection_url("keyboard-configurator", "master");
    let resp = client
        .get(url)
        //.header("accept", "application/vnd.github.v3+json")
        .header("User-Agent", "pop-branch-protection")
        .basic_auth("", Some("")) // XXX
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<Protection>()
        .unwrap();
    println!("{:#?}", resp);
}
