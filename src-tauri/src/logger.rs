use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{fmt, prelude::*};

// The logger guard must stay alive for the program lifetime.
lazy_static! {
    static ref LOG_GUARD: std::sync::Mutex<Option<WorkerGuard>> = std::sync::Mutex::new(None);
}

pub fn init_logging(app: &tauri::App) {
    // Resolve the log directory inside the Tauri application data folder.
    let log_dir = app
        .path()
        .app_log_dir()
        .expect("Unable to obtain ESL Reporter log directory");
    // Ensure the directory exists.
    std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    // Create a non‑blocking writer that writes to a single file (error.log).
    let (file_writer, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::never(&log_dir, "error.log"));

    // Layer: errors to stderr.
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(LevelFilter::ERROR);

    // Layer: errors to file.
    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_filter(LevelFilter::ERROR);

    // Layer: info (and debug) to stdout.
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(LevelFilter::INFO);

    // Build the subscriber registry.
    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    // Store the guard so the non‑blocking writer is not dropped.
    *LOG_GUARD.lock().unwrap() = Some(guard);
}
