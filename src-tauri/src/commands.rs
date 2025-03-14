pub mod downloader;

use downloader::download_playlist;
use std::sync::Arc;
use tauri::ipc::Channel;
use tokio::task::JoinSet;

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

#[tauri::command]
pub async fn download(
    token: String,
    folder: String,
    videos: Vec<Video>,
    on_progress: Channel<DownloadProgressEvent>,
    on_error: Channel<DownloadErrorEvent>,
) -> Result<(), String> {
    let token = Arc::new(token);
    let mut set = JoinSet::new();

    let num_videos = videos.len();
    let mut perc_downloaded;
    let mut count_downloaded = 0u32;

    for video in videos {
        println!("Downloading: {}", video.topic);
        let local_token: Arc<String> = Arc::clone(&token);

        set.spawn(async move {
            let video_file = &format!("{}-{}", remove_special(&video.topic), video.number);
            println!("Attempting to download `{video_file}`...");
            download_playlist(&local_token, video.ttid as usize, video_file)
                .await
                .map_err(|e| (video.number, e))?;
            println!("Finished download of video file");
            Ok(())
        });
    }

    while let Some(res) = set.join_next().await {
        count_downloaded += 1;
        if let Some((number, err)) = res.map_err(|e| e.to_string())?.err() {
            count_downloaded -= 1;
            let _ = on_error.send(DownloadErrorEvent {
                errors: vec![format!("failed to download Lecture-{number}: {err}")],
            });
        }

        perc_downloaded = (count_downloaded as f32 / num_videos as f32) * 100.0;
        println!("Downloaded {}%", perc_downloaded);

        // Even if there's an error, ignore it, since it's not vital for the download operation
        let _ = on_progress.send(DownloadProgressEvent {
            percent: perc_downloaded,
        });
    }

    Ok(())
}
