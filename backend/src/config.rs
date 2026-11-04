use std::str::FromStr;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Json,
}

impl LogFormat {
    fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "pretty" => Ok(LogFormat::Pretty),
            "json" => Ok(LogFormat::Json),
            other => Err(anyhow!(
                "invalid LOG_FORMAT `{other}` (expected `pretty` or `json`)"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub log_format: LogFormat,
    pub cors_allowed_origins: Vec<String>,
    pub max_page_size: i64,
    pub default_page_size: i64,
    pub search_min_trgm_score: f64,
    pub import_max_bytes: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());

        let log_format = match std::env::var("LOG_FORMAT") {
            Ok(raw) => LogFormat::parse(&raw)?,
            Err(_) => LogFormat::Pretty,
        };

        let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173,http://localhost:8080".to_string())
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect();

        let max_page_size = env_parse("MAX_PAGE_SIZE", 100i64)?;
        let default_page_size = env_parse("DEFAULT_PAGE_SIZE", 20i64)?;
        let search_min_trgm_score = env_parse("SEARCH_MIN_TRGM_SCORE", 0.3f64)?;
        let import_max_bytes = env_parse("IMPORT_MAX_BYTES", 16_777_216usize)?;

        if max_page_size < 1 {
            return Err(anyhow!("MAX_PAGE_SIZE must be >= 1"));
        }
        if default_page_size < 1 || default_page_size > max_page_size {
            return Err(anyhow!(
                "DEFAULT_PAGE_SIZE must be between 1 and MAX_PAGE_SIZE"
            ));
        }

        Ok(Self {
            database_url,
            bind_addr,
            log_format,
            cors_allowed_origins,
            max_page_size,
            default_page_size,
            search_min_trgm_score,
            import_max_bytes,
        })
    }

    pub fn clamp_limit(&self, limit: Option<i64>) -> i64 {
        let value = limit.unwrap_or(self.default_page_size);
        value.clamp(1, self.max_page_size)
    }
}

fn env_parse<T>(key: &str, default: T) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(key) {
        Ok(raw) => raw
            .trim()
            .parse::<T>()
            .with_context(|| format!("invalid {key}")),
        Err(_) => Ok(default),
    }
}
