use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    let stdout_layer = fmt::layer().with_target(true).with_ansi(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .init();

    log_panics::init();
}

#[cfg(test)]
pub fn setup_test_logging() {
    let filter = EnvFilter::new("debug");

    let stdout_layer = fmt::layer().with_target(true).with_ansi(true);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .try_init();

    log_panics::init();
}
