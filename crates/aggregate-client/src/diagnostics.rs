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
    if let Err(error) = std::fs::create_dir_all(&directory) {
        eprintln!(
            "Cannot create Aggregate log directory {}: {error}",
            directory.display()
        );
        return None;
    }
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
    app.insert_non_send(FileLogGuard { _worker: guard });
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
    static PANIC_LOGGING: std::sync::Once = std::sync::Once::new();
    PANIC_LOGGING.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |information| {
            tracing::error!(target: "aggregate_client::panic", panic = %information, "Application panic");
            previous(information);
        }));
    });
    info!(
        version = env!("CARGO_PKG_VERSION"),
        process_id = std::process::id(),
        session_id = %uuid::Uuid::now_v7(),
        operating_system = std::env::consts::OS,
        architecture = std::env::consts::ARCH,
        ruleset = aggregate_simulation_core::RULESET_VERSION,
        save_schema = aggregate_simulation_core::SAVE_SCHEMA_VERSION,
        preset_schema = aggregate_world::SCENARIO_SCHEMA_VERSION,
        "Aggregate started"
    );
}
