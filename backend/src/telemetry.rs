use tracing_subscriber::{fmt, EnvFilter};

use crate::config::LogFormat;

/// Initialise the global tracing subscriber. Idempotent-safe (ignores repeat init).
pub fn init(format: LogFormat) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match format {
        LogFormat::Json => {
            let _ = fmt().json().with_env_filter(filter).try_init();
        }
        LogFormat::Pretty => {
            let _ = fmt().with_env_filter(filter).try_init();
        }
    }
}
