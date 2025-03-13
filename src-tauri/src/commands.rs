pub mod downloader;

use std::sync::Arc;
use downloader::download_playlist;
use tokio::task::JoinSet;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Video {
    pub ttid: i32,
    pub topic: String,
    pub number: i32,
}

#[tauri::command]
pub async fn download(token: String, folder: String, videos: Vec<Video>) -> Result<(), String> {
    let token = Arc::new(token);
    let mut set = JoinSet::new();

    for video in videos {
        println!("Downloading: {}", video.topic);
        let local_token: Arc<String> = Arc::clone(&token);

        set.spawn(async move {
            let video_file = &format!("{}-{}", video.topic, video.number);
            println!("Attempting to download `{video_file}`...");
            download_playlist(&local_token, video.ttid as usize, &video_file)
                .await
                .unwrap();
            println!("Finished download of video file");
        });
        
    }

    while let Some(res) = set.join_next().await {
        // TODO: Add proper error handling
        let _ = res.unwrap();
    }
    Ok(())
}
