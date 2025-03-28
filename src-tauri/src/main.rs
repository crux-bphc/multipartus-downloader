// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use log::{info, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    Config,
};
use multipartus_downloader_lib::run;

fn clear_logs(dir: &PathBuf) -> std::io::Result<()> {
    // find all log files
    let mut logs: Vec<(PathBuf, SystemTime)> = std::fs::read_dir(&dir)?
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                    // Should this check for modification or creation time?
                    if let Ok(modified) = entry.metadata().and_then(|metadata| metadata.modified())
                    {
                        return Some((path, modified));
                    }
                }
            }
            None
        })
        .collect();
    logs.sort_by_key(|&(_, modified)| modified);

    // max 10 logs
    let max_logs = 10;
    let log_count = logs.len();
    if log_count >= max_logs {
        for (path, _) in logs.into_iter().take(log_count - max_logs + 1) {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}

fn main() {
    // outfile
    let mut temp = std::env::temp_dir()
        .join("multipartus-downloader")
        .join("logs");

    // Remove old log files to not waste space
    // Ignore errors created by removing files, since it's not necessary
    clear_logs(&temp).unwrap_or(());

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Create a new file every time this app is launched
    temp.push(format!("log-{time}.txt"));

    // Create an appender to log to a file
    let file_appender = FileAppender::builder()
        .build(temp)
        .expect("Failed to initialize file appender!");

    let config = Config::builder()
        .appender(Appender::builder().build("file-logging", Box::new(file_appender)))
        .build(
            Root::builder()
                .appender("file-logging")
                .build(LevelFilter::max()),
        )
        .expect("Failed to initialize logging!");

    // Initialize log4rs
    let _ = log4rs::init_config(config);

    info!("Starting multipartus-downloader");

    run()
}
