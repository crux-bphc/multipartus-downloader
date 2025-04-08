pub mod downloader;

use crate::prelude::*;
use downloader::{download_playlist, Resolution};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use tokio::sync::mpsc;

use std::{ops::DerefMut, path::PathBuf, sync::Arc};
use tauri::{ipc::Channel, AppHandle, Manager, State};
use tauri_plugin_shell::{
    process::{CommandEvent, TerminatedPayload},
    ShellExt,
};
use tokio::{io::AsyncWriteExt, sync::Mutex, task::JoinSet};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    ttid: i32,
    topic: String,
    subject_name: String,
    number: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgressEvent {
    percent: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadErrorEvent {
    errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    resolution: Resolution,
    base: Option<String>,
}

fn remove_special(string: impl AsRef<str>) -> String {
    string
        .as_ref()
        .replace(['/', '|'], "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || c == &'_' || c == &'.')
        .collect()
}

// TODO: Improve error handling
#[instrument(fields(nth, ?video, %token, %folder, ?resolution), skip_all)]
async fn download_mp4(
    resolution: Resolution,
    base: Arc<Option<String>>,
    nth: usize,
    tx: Arc<mpsc::Sender<(usize, f32)>>,
    video: &Video,
    token: Arc<String>,
    folder: Arc<String>,
    app: Arc<AppHandle>,
) -> Result<i32, (i32, String)> {
    let video_file = &format!("{}_{}", video.number, remove_special(&video.topic));

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

    info!("Checking download location");

    let mut location = PathBuf::new().join(folder.to_string());
    let subject_name = remove_special(&video.subject_name);

    // Download in the given folder if the filename of the folder is the subject name
    if location
        .file_name()
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("")
        != subject_name
    {
        info!(
            "Given folder is not in folder with subject name {}. Adding subject folder",
            subject_name
        );
        location.push(subject_name);
    }

    // Create directory to store current subject lectures if not already created
    tokio::fs::create_dir_all(&location)
        .await
        .context("creating subject download location")
        .map_err(|e| (video.number, e.to_string()))?;

    location.push(format!("{video_file}_{resolution}.mp4"));

    // Skip this download if it exists
    if location.exists() {
        // Say it's at 100%
        let _ = tx.send((nth, 100.0)).await;
        return Ok(video.ttid);
    }

    info!("Starting download of m3u8 playlist");

    let (side1, side2) = download_playlist(
        resolution,
        base,
        itx,
        &token,
        video.ttid as usize,
        video_file,
    )
    .await
    .with_context(|| format!("downloading playlist: {}", video.number))
    .map_err(|e| (video.number, e.to_string()))?;

    info!("m3u8 playlist download complete");

    info!("Creating output video file at {:#?}", location);

    let ffmpeg = app
        .shell()
        .sidecar("ffmpeg")
        .context("ffmpeg command create")
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
    let (mut rx, _child) = ffmpeg
        .spawn()
        .context("spawn ffmpeg")
        .map_err(|e| (video.number, e.to_string()))?;

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
    Ok(video.ttid)
}

fn get_temp() -> PathBuf {
    std::env::temp_dir()
        .join("multipartus-downloader")
        .join("videos")
}

#[tauri::command]
#[instrument(skip_all)]
pub async fn clear_cache() -> Result<(), String> {
    info!("clear_cache command invoked");
    let temp = get_temp();
    tokio::fs::remove_dir_all(temp.as_path().to_str().unwrap_or("./tmp"))
        .await
        .inspect_err(|e| error!("error clearing cache: {e}"))
        .context("removing cache dir")
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Should this run on another thread?
#[tauri::command]
#[instrument(skip_all)]
pub fn get_cache_size() -> Result<String, String> {
    info!("get_cache_size command invoked");
    let temp = get_temp();
    if !temp.exists() {
        info!("Temp file for multipartus-downloader does not exist");
        return Ok("0KiB".to_string());
    }
    dir_size::get_size_in_human_bytes(temp.as_path())
        .inspect_err(|e| error!("failed getting temp dir size: {e}"))
        .context("getting temp dir size")
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[instrument(skip_all)]
pub async fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    info!("save_settings command invoked");

    // the .inspect_err.context.map_err waterfall is a lil fucked, but this is an top level fn.
    // we have to log the errors here

    let app_data = app
        .path()
        .app_data_dir()
        .inspect_err(|e| error!("error reading app data dir: {e}"))
        .context("reading app data dir")
        .map_err(|e| e.to_string())?;

    tokio::fs::create_dir_all(&app_data)
        .await
        .inspect_err(|e| error!("failed creating app data dir: {e}"))
        .context("creating app data dir")
        .map_err(|e| e.to_string())?;

    let mut out = tokio::fs::File::create(app_data.join("settings.json"))
        .await
        .inspect_err(|e| error!("failed creating settings.json: {e}"))
        .context("creating settings.json")
        .map_err(|e| e.to_string())?;

    let json = serde_json::to_string(&settings)
        .inspect_err(|e| {
            error!("failed at serializing settings: {e}");
            trace!(?settings);
        })
        .context("serializing settings to json")
        .map_err(|e| e.to_string())?;

    out.write(json.as_bytes())
        .await
        .inspect_err(|e| error!("failed writing settings.json: {e}"))
        .context("writing settings.json")
        .map_err(|e| e.to_string())?;

    info!("Saved new settings");

    Ok(())
}

#[instrument(skip_all)]
async fn get_settings(app: &AppHandle) -> Result<Settings, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .context("reading app data dir path")
        .map_err(|e| e.to_string())?;

    let out = serde_json::from_slice(
        tokio::fs::read(app_data.join("settings.json"))
            .await
            .context("reading settings.json")
            .map_err(|e| e.to_string())?
            .as_slice(),
    )
    .context("deserializing settings.json")
    .map_err(|e| e.to_string())?;

    info!("Settings loaded");

    Ok(out)
}

#[instrument(skip_all)]
async fn get_resolved_settings(app: &AppHandle) -> (Resolution, Option<String>) {
    let settings = get_settings(app).await;
    if let Ok(Settings { resolution, base }) = settings {
        (resolution, base)
    } else {
        (Resolution::HighRes, None)
    }
}

#[tauri::command]
#[instrument(skip_all)]
pub async fn load_settings(app: AppHandle) -> Result<Settings, String> {
    info!("load_settings command invoked");
    get_settings(&app).await
}

#[tauri::command]
#[instrument(fields(token, folder), skip_all)]
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

    let (resolution, base) = get_resolved_settings(&app).await;
    let base = Arc::new(base);

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
        let base_clone = Arc::clone(&base);

        let tx_clone = Arc::clone(&tx);
        set.spawn(async move {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    info!("Cancelled download of {}", video.ttid);
                    Err((video.number, "Cancelled".to_string()))
                }
                // does this need to be cancel safe?
                result = download_mp4(resolution, base_clone, i, tx_clone, &video, local_token, folder_ref, app_ref) => result,
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
        match res.map_err(|e| e.to_string())? {
            Err((number, err)) => {
                error!("Failed to download Lecture-{number}: {err}");
                let _ = on_error.send(DownloadErrorEvent {
                    errors: vec![format!("Failed to download Lecture-{number}"), err],
                });
            }

            Ok(ttid) => {
                info!("Deleting lecture {} from cache", ttid);
                // This lecture download has completed, remove it from the cache
                let remove_loc = get_temp().join(format!("Lecture_{}", ttid));

                let _ = tokio::fs::remove_dir_all(remove_loc)
                    .await
                    .inspect_err(|error| {
                        error!("Failed to remove download folder of lecture {ttid}: {error}");
                    });
            }
        };
    }

    Ok(())
}

#[tauri::command]
#[instrument(skip_all)]
pub async fn cancel_download(
    cancellation_token: State<'_, Mutex<CancellationToken>>,
) -> Result<(), ()> {
    info!("Cancelling all running download tasks");
    let token = cancellation_token.lock().await;
    token.cancel();
    info!("Cancelled all download tasks");
    Ok(())
}
