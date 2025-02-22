pub mod downloader;

use std::{
    fs,
    path::{Path, PathBuf},
    vec,
};

use tauri_plugin_http::reqwest;

const BASE_URL: &str = "http://localhost:3000/impartus";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Video {
    pub ttid: i32,
    pub topic: String,
    pub number: i32,
}

#[derive(serde::Deserialize)]
struct MasterChunkInfo {
    tracks: Vec<Vec<String>>,
}

async fn get_m3u8(client: &reqwest::Client, ttid: i32) -> Result<String, String> {
    let master_m3u8 = client
        .get(format!("{BASE_URL}/ttid/{ttid}/m3u8/info"))
        .send()
        .await
        .map_err(|_| String::from("Failed to fetch m3u8"))?
        .text()
        .await
        .map_err(|_| String::from("Failed to read m3u8"))?;

    let m3u8_info: MasterChunkInfo = serde_json::from_str(&master_m3u8).unwrap();
    let url = m3u8_info.tracks[0][1].as_str();

    dbg!(format!("got m3u8 url: {url}"));

    let chunk = client
        .get(format!("{BASE_URL}/chunk/m3u8/left"))
        .query(&[("m3u8", url)])
        .send()
        .await
        .map_err(|_| String::from("Failed to fetch chunks"))?
        .text()
        .await
        .map_err(|_| String::from("Failed to read chunks"))?;

    return Ok(chunk);
}

fn process_m3u8(chunk: String) -> (String, Vec<String>) {
    let mut chunks: Vec<String> = vec![];
    let m3u8 = chunk
        .lines()
        .map(|line| {
            if line.starts_with("#EXT-X-KEY") {
                return format!("#EXT-X-KEY:METHOD=AES-128,URI=\"./key\"");
            }

            if line.ends_with(".ts") {
                chunks.push(String::from(line));
                return format!("./{}.ts", chunks.len() - 1);
            }

            return String::from(line);
        })
        .collect::<Vec<_>>()
        .join("\n");
    return (m3u8, chunks);
}

async fn download_chunk(
    client: &reqwest::Client,
    url: &String,
    path: PathBuf,
) -> Result<(), String> {
    let chunk = client
        .get(url)
        .send()
        .await
        .map_err(|_| String::from("Failed to fetch chunk"))?
        .bytes()
        .await
        .map_err(|_| String::from("Failed to read chunk"))?
        .to_vec();

    fs::write(path, chunk).expect("Failed to write chunk");
    Ok(())
}

#[tauri::command]
pub async fn download(token: String, folder: String, videos: Vec<Video>) -> Result<(), String> {
    let base_dir = Path::new(&folder);
    let temp_dir = base_dir.join("tmp");

    // Temp files: {{folder}}/tmp/...
    // Lecture videos: {{folder}}/Lecture {number} - {topic}.mp4

    fs::create_dir_all(base_dir).expect("Failed to create folder");
    fs::create_dir_all(&temp_dir).expect("Failed to create folder");

    let client = reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {token}").parse().unwrap(),
            );
            headers
        })
        .build()
        .map_err(|_| String::from("Failed to create client"))?;

    for video in videos {
        let ttid_dir = temp_dir.join(video.ttid.to_string());

        fs::create_dir(&ttid_dir).expect("Failed to create folder");

        let decryption_key = client
            .get(format!("{BASE_URL}/ttid/{ttid}/key", ttid = video.ttid))
            .send()
            .await
            .map_err(|_| String::from("Failed to fetch decryption key"))?
            .bytes()
            .await
            .map_err(|_| String::from("Failed to read decryption key"))?
            .to_vec();

        fs::write(ttid_dir.join("key"), decryption_key).expect("Failed to write key");

        let m3u8 = get_m3u8(&client, video.ttid).await?;

        let (m3u8, chunks) = process_m3u8(m3u8);

        fs::write(ttid_dir.join("index.m3u8"), m3u8).expect("Failed to write m3u8");

        for (i, chunk) in chunks.iter().enumerate() {
            download_chunk(&client, chunk, ttid_dir.join(format!("{i}.ts"))).await?;
        }
    }
    Ok(())
}
