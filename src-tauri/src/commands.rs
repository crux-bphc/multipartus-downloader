pub mod downloader;

use downloader::{download_playlist, Resolution};
use log::{error, info};
use tokio_util::sync::CancellationToken;

use std::{ops::DerefMut, path::PathBuf, sync::Arc};
use tauri::{ipc::Channel, AppHandle, Manager, State};
use tauri_plugin_shell::{
    process::{CommandEvent, TerminatedPayload},
    ShellExt,
};
use tokio::{io::AsyncWriteExt, sync::Mutex, task::JoinSet};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Video {
    pub ttid: i32,
    pub topic: String,
    pub number: i32,
}

#[derive(Clone, serde::Serialize)]
pub struct DownloadProgressEvent {
    percent: f32,
}

#[derive(Clone, serde::Serialize)]
pub struct DownloadErrorEvent {
    errors: Vec<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    resolution: Resolution,
}

fn remove_special(string: &str) -> String {
    string
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || c == &'_' || c == &'.')
        .collect()
}

// TODO: Improve error handling
async fn download_mp4(
    resolution: Resolution,
    nth: usize,
    tx: Arc<tokio::sync::mpsc::Sender<(usize, f32)>>,
    video: &Video,
    token: Arc<String>,
    folder: Arc<String>,
    app: Arc<AppHandle>,
) -> Result<(), (i32, String)> {
    let video_file = &format!("{}_{}", remove_special(&video.topic), video.number);

    info!("download_mp4 invoked");

    let (itx, mut irx) = tokio::sync::watch::channel(0f32);

    // Monitor progress of the download function, and send it out to the
    // mpsc channel waiting for a progress-report of each download task running
    let tx_clone = Arc::clone(&tx);
    tokio::spawn(async move {
        while irx.changed().await.is_ok() {
            let progress = *irx.borrow();
            let _ = tx_clone.send((nth, progress * 0.5)).await;
        }
    });

    info!("Starting download of m3u8 playlist");

    let (side1, side2) =
        download_playlist(resolution, itx, &token, video.ttid as usize, video_file)
            .await
            .map_err(|e| (video.number, e.to_string()))?;

    info!("m3u8 playlist download complete");

    let mut location = PathBuf::new().join(format!("{folder}/{video_file}_{resolution}.mp4"));
    let mut i = 1;

    // Creates a new file instead of attempting to replace it
    // since ffmpeg puts up a y/n prompt and waits till input,
    // This is an easier solution to that problem
    while location.exists() {
        location.pop();
        location.push(format!("{video_file}_{resolution} ({i}).mp4"));
        i += 1;
    }

    info!("Creating output video file at {:#?}", location);

    let ffmpeg = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| (video.number, e.to_string()))?;

    let location_str = location.to_str().ok_or(()).map_err(|_| {
        (
            video.number,
            "Failed to access provided download location!".to_string(),
        )
    })?;

    let mut args = vec![
        "-allowed_extensions",
        "ALL",
        "-i",
        &side1,
        "-c",
        "copy",
        location_str,
    ];
    if let Some(side2) = &side2 {
        args = vec![
            "-allowed_extensions",
            "ALL",
            "-i",
            &side1,
            "-allowed_extensions",
            "ALL",
            "-i",
            side2,
            "-map",
            "0",
            "-map",
            "1",
            "-c",
            "copy",
            location_str,
        ]
    }

    info!("Spawning ffmpeg");

    let ffmpeg = ffmpeg.args(args.as_slice());

    let mut ffmpeg_errors = String::new();
    let (mut rx, _child) = ffmpeg.spawn().map_err(|e| (video.number, e.to_string()))?;

    info!("ffmpeg spawned");

    let _ = tx.try_send((nth, 50.0));

    // Approximately 691 (for 2 sides) or 345 (for 1 side) files exist for ffmpeg to
    // compile to mp4, so including the other messages it outputs, it ends up being
    // about 2800 or 1400 outputs that count as an increment for the percentage
    // A more correct way to go about this would be to pass `-progress` to ffmpeg,
    // parse out_time and compare against "Duration: xx:xx:xx.xxx" parameter that's
    // produced by it when starting. But this works well enough.
    let max_count_output = (if side2.is_some() { 700.0 } else { 350.0 }) * 4.0;
    let mut output_count = 0;
    while let Some(event) = rx.recv().await {
        match event {
            // WHY DOES STDERR HAVE STDOUT AS WELL??!?!?
            CommandEvent::Stderr(bytes) => {
                let line = String::from_utf8_lossy(&bytes);
                ffmpeg_errors.push_str(&line);
                ffmpeg_errors += "\n";
                let _ = tx.try_send((nth, 50.0 + (output_count as f32 * 50.0 / max_count_output)));
                output_count += 1;
                // Unecessary most probably, but here just in case
                if output_count > max_count_output as usize {
                    output_count = max_count_output as usize;
                }
            }

            CommandEvent::Error(str) => {
                ffmpeg_errors.push_str(&str);
                ffmpeg_errors += "\n";
            }

            // 0 = successful exit, 4 = user cancelled
            CommandEvent::Terminated(TerminatedPayload {
                code: Some(0 | 4), ..
            }) => {
                ffmpeg_errors.clear();
            }

            _ => (),
        }
    }

    if !ffmpeg_errors.is_empty() {
        info!("ffmpeg failed with: \n{ffmpeg_errors}");
        return Err((video.number, ffmpeg_errors));
    }

    info!(
        "ffmpeg completed generation of output mp4 for {} at `{}`",
        video.ttid,
        location.to_str().unwrap_or("")
    );

    let _ = tx.try_send((nth, 100.0));
    Ok(())
}

fn get_temp() -> PathBuf {
    std::env::temp_dir()
        .join("multipartus-downloader")
        .join("videos")
}

#[tauri::command]
pub async fn clear_cache() -> Result<(), String> {
    info!("clear_cache command invoked");
    let temp = get_temp();
    tokio::fs::remove_dir_all(temp.as_path().to_str().unwrap_or("./tmp"))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Should this run on another thread?
#[tauri::command]
pub fn get_cache_size() -> Result<String, String> {
    info!("get_cache_size command invoked");
    let temp = get_temp();
    if !temp.exists() {
        info!("Temp file for multipartus-downloader does not exist");
        return Ok("0K".to_string());
    }
    dir_size::get_size_in_abbr_human_bytes(temp.as_path()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    info!("save_settings command invoked");
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;

    tokio::fs::create_dir_all(&app_data)
        .await
        .map_err(|e| e.to_string())?;

    let mut out = tokio::fs::File::create(app_data.join("settings.json"))
        .await
        .map_err(|e| e.to_string())?;

    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;

    out.write(json.as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    info!("Saved new settings");

    Ok(())
}

async fn get_settings(app: &AppHandle) -> Result<Settings, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;

    tokio::fs::create_dir_all(&app_data)
        .await
        .map_err(|e| e.to_string())?;

    let out = serde_json::from_slice(
        tokio::fs::read(app_data.join("settings.json"))
            .await
            .map_err(|e| e.to_string())?
            .as_slice(),
    )
    .map_err(|e| e.to_string())?;

    info!("Settings loaded");

    Ok(out)
}

async fn get_resolution(app: &AppHandle) -> Resolution {
    if let Ok(Settings { resolution }) = get_settings(app).await {
        resolution
    } else {
        Resolution::HighRes
    }
}

#[tauri::command]
pub async fn load_settings(app: AppHandle) -> Result<Settings, String> {
    info!("load_settings command invoked");
    get_settings(&app).await
}

#[tauri::command]
pub async fn download(
    cancellation_token: State<'_, Mutex<CancellationToken>>,
    app: AppHandle,
    token: String,
    folder: String,
    videos: Vec<Video>,
    on_progress: Channel<DownloadProgressEvent>,
    on_error: Channel<DownloadErrorEvent>,
) -> Result<(), String> {
    info!("download command invoked");

    // Reset the cancellation token
    let cancellation_token = {
        let mut old_cancellation_token = cancellation_token.lock().await;
        *(old_cancellation_token.deref_mut()) = CancellationToken::new();
        old_cancellation_token.clone()
    };

    let resolution = get_resolution(&app).await;

    let token = Arc::new(token);
    let folder = Arc::new(folder);
    let app = Arc::new(app);

    let mut set = JoinSet::new();

    let num_videos = videos.len();

    let (tx, mut rx) = tokio::sync::mpsc::channel(videos.len());

    let tx = Arc::new(tx);

    for (i, video) in videos.into_iter().enumerate() {
        info!("Queuing download of {}", video.ttid);
        let local_token = Arc::clone(&token);
        let app_ref = Arc::clone(&app);
        let folder_ref = Arc::clone(&folder);
        let cancellation_token = cancellation_token.clone();

        let tx_clone = Arc::clone(&tx);
        set.spawn(async move {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    info!("Cancelled download of {}", video.ttid);
                    Err((video.number, "Cancelled".to_string()))
                }
                // does this need to be cancel safe?
                result = download_mp4(resolution, i, tx_clone, &video, local_token, folder_ref, app_ref) => result,
            }
        });
    }

    // Send progress as each download task sends a message through the mpsc channel
    tokio::spawn(async move {
        let mut channels = vec![0.0; num_videos];

        // The channel recieves a progress message from any one of the channels
        while let Some((nth, progress)) = rx.recv().await {
            channels[nth] = progress;
            let avg_progress = channels.iter().sum::<f32>() / (num_videos as f32);
            let _ = on_progress.send(DownloadProgressEvent {
                percent: avg_progress,
            });
        }
    });

    while let Some(res) = set.join_next().await {
        if let Err((number, err)) = res.map_err(|e| e.to_string())? {
            error!("Failed to download Lecture-{number}: {err}");
            let _ = on_error.send(DownloadErrorEvent {
                errors: vec![format!("Failed to download Lecture-{number}"), err],
            });
        };
    }

    drop(set);

    Ok(())
}

#[tauri::command]
pub async fn cancel_download(
    cancellation_token: State<'_, Mutex<CancellationToken>>,
) -> Result<(), ()> {
    info!("Cancelling all running download tasks");
    let token = cancellation_token.lock().await;
    token.cancel();
    info!("Cancelled all download tasks");
    Ok(())
}
