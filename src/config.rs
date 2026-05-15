use anyhow::{bail, Result};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 10;

fn require_env(name: &str) -> Result<String> {
    let value =
        env::var(name).map_err(|_| anyhow::anyhow!("{} environment variable is required", name))?;

    if value.trim().is_empty() {
        bail!("{} environment variable cannot be empty", name);
    }

    Ok(value)
}

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub file_path: String,
    pub timeout: Duration,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let api_key = require_env("AMBROGIO_LLM_API_KEY")?;
        let base_url = require_env("AMBROGIO_LLM_URL")?;
        let model = require_env("AMBROGIO_LLM_MODEL")?;
        let file_path = require_env("AMBROGIO_DAILY_ORGANISER_FILE")?;

        let timeout_secs = env::var("AMBROGIO_LLM_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);
        let timeout = Duration::from_secs(timeout_secs);

        Ok(Config {
            api_key,
            base_url,
            model,
            file_path,
            timeout,
        })
    }
}

/// Returns the value of `AMBROGIO_DAILY_ORGANISER_FILE` as a `PathBuf`.
/// Errors if the variable is unset or empty.
pub fn organiser_path_from_env() -> Result<PathBuf> {
    Ok(PathBuf::from(require_env("AMBROGIO_DAILY_ORGANISER_FILE")?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // `std::env::set_var` is not thread-safe and `cargo test` runs tests in
    // parallel. Tests that touch `AMBROGIO_DAILY_ORGANISER_FILE` serialise
    // on this mutex.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn default_timeout_is_10_seconds() {
        assert_eq!(DEFAULT_TIMEOUT_SECS, 10);
    }

    #[test]
    fn require_env_rejects_empty_string() {
        env::set_var("TEST_EMPTY_VAR", "");
        let result = require_env("TEST_EMPTY_VAR");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
        env::remove_var("TEST_EMPTY_VAR");
    }

    #[test]
    fn require_env_rejects_whitespace_only() {
        env::set_var("TEST_WHITESPACE_VAR", "   ");
        let result = require_env("TEST_WHITESPACE_VAR");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
        env::remove_var("TEST_WHITESPACE_VAR");
    }

    #[test]
    fn require_env_accepts_valid_value() {
        env::set_var("TEST_VALID_VAR", "valid-value");
        let result = require_env("TEST_VALID_VAR");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "valid-value");
        env::remove_var("TEST_VALID_VAR");
    }

    #[test]
    fn require_env_rejects_missing_var() {
        env::remove_var("TEST_MISSING_VAR");
        let result = require_env("TEST_MISSING_VAR");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("is required"));
    }

    #[test]
    fn organiser_path_from_env_returns_env_var_verbatim() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var(
            "AMBROGIO_DAILY_ORGANISER_FILE",
            "/home/user/notes/organiser.md",
        );
        let path = organiser_path_from_env().unwrap();
        assert_eq!(path, PathBuf::from("/home/user/notes/organiser.md"));
        env::remove_var("AMBROGIO_DAILY_ORGANISER_FILE");
    }

    #[test]
    fn organiser_path_from_env_errors_on_missing_var() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("AMBROGIO_DAILY_ORGANISER_FILE");
        let result = organiser_path_from_env();
        assert!(result.is_err());
    }
}
