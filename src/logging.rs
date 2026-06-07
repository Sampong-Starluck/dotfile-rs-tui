use std::fs::File;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init() {
    let file = File::create("debug.log").expect("failed to create debug.log");

    fmt()
        .with_writer(file)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("debug")),  // fallback if RUST_LOG not set
        )
        .with_ansi(false)
        .with_target(true)
        .with_line_number(true)
        .init();
}