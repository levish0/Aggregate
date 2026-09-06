//! Extend Bevy's existing subscriber; retain the writer until application shutdown.
use bevy::{
    log::{BoxedLayer, LogPlugin},
    prelude::*,
};
use std::path::PathBuf;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::fmt::format::FmtSpan;

struct FileLogGuard {
    _worker: WorkerGuard,
}

pub fn log_plugin() -> LogPlugin {
    LogPlugin {
        filter: format!(
            "{},aggregate_client=info,aggregate_map_view=info,aggregate_simulation_core=info",
            bevy::log::DEFAULT_FILTER
        ),
        custom_layer: file_log_layer,
        ..default()
    }
}

fn file_log_layer(app: &mut App) -> Option<BoxedLayer> {
    let directory = std::env::var_os("AGGREGATE_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("Aggregate/logs")
        });
    let writer = match RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("aggregate")
        .filename_suffix("jsonl")
        .max_log_files(14)
        .build(&directory)
    {
        Ok(writer) => writer,
        Err(error) => {
            // The global subscriber is not installed yet; console must remain usable.
            eprintln!(
                "Cannot create Aggregate file log at {}: {error}",
                directory.display()
            );
            return None;
        }
    };
    let (writer, guard) = tracing_appender::non_blocking(writer);
    app.insert_non_send_resource(FileLogGuard { _worker: guard });
    Some(Box::new(
        tracing_subscriber::fmt::layer()
            .json()
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(writer),
    ))
}

pub fn log_startup() {
    info!(
        version = env!("CARGO_PKG_VERSION"),
        process_id = std::process::id(),
        "Aggregate started"
    );
}
