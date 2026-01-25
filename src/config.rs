use anyhow::Result;
use std::env;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 10;

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
        let api_key = env::var("AMBROGIO_LLM_API_KEY")
            .map_err(|_| anyhow::anyhow!("AMBROGIO_LLM_API_KEY environment variable is required"))?;

        let base_url = env::var("AMBROGIO_LLM_URL")
            .map_err(|_| anyhow::anyhow!("AMBROGIO_LLM_URL environment variable is required"))?;

        let model = env::var("AMBROGIO_LLM_MODEL")
            .map_err(|_| anyhow::anyhow!("AMBROGIO_LLM_MODEL environment variable is required"))?;

        let file_path = env::var("AMBROGIO_DAILY_ORGANISER_FILE").map_err(|_| {
            anyhow::anyhow!("AMBROGIO_DAILY_ORGANISER_FILE environment variable is required")
        })?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_timeout_is_10_seconds() {
        assert_eq!(DEFAULT_TIMEOUT_SECS, 10);
    }
}
