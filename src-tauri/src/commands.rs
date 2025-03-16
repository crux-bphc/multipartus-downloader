pub mod downloader;

use downloader::download_playlist;
use tokio_util::sync::CancellationToken;

use std::{ops::DerefMut, path::PathBuf, sync::Arc};
use tauri::{ipc::Channel, AppHandle, State};
use tauri_plugin_shell::{
    process::{CommandEvent, TerminatedPayload},
    ShellExt,
};
use tokio::{task::JoinSet, sync::Mutex};

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

fn remove_special(string: &str) -> String {
    string
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || c == &'_' || c == &'.')
        .collect()
}

// TODO: Improve error handling
async fn download_mp4(
    video: &Video,
    token: Arc<String>,
    folder: Arc<String>,
    app: Arc<AppHandle>,
) -> Result<(), (i32, String)> {
    let video_file = &format!("{}-{}", remove_special(&video.topic), video.number);
    println!("Attempting to download `{video_file}`...");
    let (side1, side2) = download_playlist(&token, video.ttid as usize, video_file)
        .await
        .map_err(|e| (video.number, e.to_string()))?;

    println!("Downloaded video playlist. Attempting to generate output mp4...");

    let mut location = PathBuf::new().join(format!("{folder}/{video_file}.mp4"));
    let mut i = 1;
    // Creates a new file instead of attempting to replace it
    // since ffmpeg puts up a y/n prompt and waits till input,
    // This is an easier solution to that problem
    while location.exists() {
        location.pop();
        location.push(format!("{video_file} ({i}).mp4"));
        i += 1;
    }

    let ffmpeg = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| (video.number, e.to_string()))?
        .args([
            "-allowed_extensions",
            "ALL",
            "-i",
            &side1,
            "-allowed_extensions",
            "ALL",
            "-i",
            &side2,
            "-map",
            "0",
            "-map",
            "1",
            "-c",
            "copy",
            location.to_str().ok_or(()).map_err(|_| {
                (
                    video.number,
                    "Failed to access provided download location!".to_string(),
                )
            })?,
        ]);

    let mut ffmpeg_errors = String::new();
    let (mut rx, _child) = ffmpeg.spawn().map_err(|e| (video.number, e.to_string()))?;
    while let Some(event) = rx.recv().await {
        match event {
            // WHY DOES STDERR HAVE STDOUT AS WELL??!?!?
            CommandEvent::Stderr(bytes) => {
                let line = String::from_utf8_lossy(&bytes);
                ffmpeg_errors.push_str(&line);
                ffmpeg_errors += "\n";
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
        println!("ffmpeg encounntered errors: \n{ffmpeg_errors}");
        return Err((video.number, ffmpeg_errors));
    }

    println!(
        "Generated output mp4 sucessfully at `{}`",
        location.to_str().unwrap_or("")
    );
    Ok(())
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
    let mut old_cancellation_token = cancellation_token.lock().await;
    *(old_cancellation_token.deref_mut()) = CancellationToken::new();
    let cancellation_token = old_cancellation_token.clone();
    drop(old_cancellation_token);

    let token = Arc::new(token);
    let folder = Arc::new(folder);
    let app = Arc::new(app);

    let mut set = JoinSet::new();

    let num_videos = videos.len();
    let mut perc_downloaded;
    let mut count_downloaded = 0u32;

    for video in videos {
        println!("Downloading: {}", video.topic);
        let local_token = Arc::clone(&token);
        let app_ref = Arc::clone(&app);
        let folder_ref = Arc::clone(&folder);
        let cancellation_token = cancellation_token.clone();
        set.spawn(async move { 
            tokio::select! {
                _ = cancellation_token.cancelled() => { 
                    println!("Cancelled download of Lecture-{}", video.number);
                    Err((video.number, "Cancelled".to_string()))
                }
                result = async { download_mp4(&video, local_token, folder_ref, app_ref).await } => result,
            }
        });
    }

    while let Some(res) = set.join_next().await {
        if let Err((number, err)) = res.map_err(|e| e.to_string())? {
            let _ = on_error.send(DownloadErrorEvent {
                errors: vec![format!("Failed to download Lecture-{number}: {err}")],
            });
        } else {
            count_downloaded += 1;
        }

        perc_downloaded = (count_downloaded as f32 / num_videos as f32) * 100.0;
        println!("Downloaded {}%", perc_downloaded);

        // Even if there's an error, ignore it, since it's not vital for the download operation
        let _ = on_progress.send(DownloadProgressEvent {
            percent: perc_downloaded,
        });
    }

    drop(set);

    Ok(())
}

#[tauri::command]
pub async fn cancel_download(cancellation_token: State<'_, Mutex<CancellationToken>>) -> Result<(), ()> {
    println!("Attempting to cancel all download tasks...");
    let token = cancellation_token.lock().await;
    token.cancel();
    println!("Cancelled all download tasks sucessfully");
    Ok(())
}
