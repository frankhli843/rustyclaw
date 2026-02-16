use tracing_subscriber::{fmt, EnvFilter};

/// Initialize the logging/tracing subsystem.
pub fn init_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("rustyclaw=debug,tower_http=debug")
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("rustyclaw=info,tower_http=info"))
    };

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .compact()
        .init();
}
