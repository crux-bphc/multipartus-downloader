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
            let video_file = &format!("{}-{}", video.topic, video.number);
            println!("Attempting to download `{video_file}`...");
            if let Err(error) =
                download_playlist(&local_token, video.ttid as usize, &video_file).await
            {
                return Err((video.number, error));
            }
            println!("Finished download of video file");
            Ok(())
        });
    }

    while let Some(res) = set.join_next().await {
        count_downloaded += 1;
        match res {
            Err(error) => {
                return Err(error.to_string());
            }
            Ok(Err((number, error))) => {
                count_downloaded -= 1;
                let error = format!("Failed to download Lecture-{number}: {error}");
                on_error
                    .send(DownloadErrorEvent {
                        errors: vec![error],
                    })
                    .unwrap_or(());
            }
            Ok(Ok(())) => (),
        }

        perc_downloaded = (count_downloaded as f32 / num_videos as f32) * 100.0;
        println!("Downloaded {}%", perc_downloaded);

        // Even if there's an error, ignore it, since it's not vital for the download operation
        on_progress
            .send(DownloadProgressEvent {
                percent: perc_downloaded,
            })
            .unwrap_or(());
    }

    Ok(())
}
