//! Structured logging initialisation.

/// Initialise the global tracing subscriber.
///
/// The filter accepts the levels `error`, `warn`, `info`, `debug`, and
/// `trace` (plus directives such as `vox=debug`). An invalid or unset filter
/// falls back to `info`. Output goes to stderr so stdout stays reserved for
/// program output. Calling this more than once is a no-op.
pub fn init(filter: &str) {
    let env_filter = tracing_subscriber::EnvFilter::try_new(filter)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .try_init();
}
