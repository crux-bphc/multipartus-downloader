// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tracing::info;
use tracing_appender::rolling::{Rotation, RollingFileAppender};

use multipartus_downloader_lib::run;

fn main() {
    // outfile
    let temp = std::env::temp_dir()
        .join("multipartus-downloader")
        .join("logs");

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("multipartus-downloader-log")
        .build(temp)
        .expect("Failed to initialize rolling file appender");
        
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(non_blocking)
        .init();

    info!("Starting multipartus-downloader");

    run()
}
