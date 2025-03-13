use std::{fmt::Display, io::Write};

use anyhow::{Context, Result};

use tauri_plugin_http::reqwest::{self, Client};

use std::sync::LazyLock;

// A static instance of a client, so that just one client is used for all requests
static CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

#[derive(Debug)]
pub struct DownloadError {
    error_string: String,
}

impl DownloadError {
    fn new(message: String) -> Self {
        Self {
            error_string: message,
        }
    }
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_string)
    }
}

impl std::error::Error for DownloadError {}

/// References static client to perform a GET request with the token auth header
async fn get(url: &str, id_token: &str) -> Result<reqwest::Response> {
    Ok(CLIENT
        .get(url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {id_token}"))
        .send()
        .await
        .context(format!("Failed to GET data from url \"{url}\"!"))?)
}

/// Creates an m3u8 file referencing local unencrypted .ts files
pub async fn download_playlist(id_token: &str, ttid: usize, filename: &str) -> Result<()> {
    // Get env variables

    // {temp}/multipartus-downloader/Lecture-<lecture-ttid>
    let temp_location = std::env::temp_dir()
        .join("multipartus-downloader")
        .join(format!("Lecture-{ttid}"));

    let temp = temp_location.as_path().to_str().unwrap_or("./tmp");

    // Create this temp location if it doesn't exist
    std::fs::create_dir_all(&temp)
        .context(format!("Failed to create temporary directory {}!", temp))?;

    let base = std::env::var("BASE").context("Failed to fetch environment variable `BASE`!")?;

    // URLs to get data from
    let m3u8_url = format!("{base}/impartus/ttid/{ttid}/m3u8");
    let key_url = format!("{base}/impartus/ttid/{ttid}/key");

    // Temp locations to store the files used for generating outputs
    let m3u8_file_path = format!("{temp}/{filename}.m3u8");
    let key_file_path = format!("{temp}/{filename}.key.key");

    // I hope you love these beautiful waterfalls @TheComputerM :)
    // Get impartus .m3u8 file
    let m3u8_index_text = get(&m3u8_url, id_token)
        .await
        .context("Failed to fetch index playlist file!")?
        .text()
        .await
        .context("Failed to read contents of index playlist file!")?;

    // TODO: Pick resolution
    let high_res_m3u8 = m3u8_index_text.lines().nth(2).context(format!(
        "Failed to get playlist file data! {m3u8_index_text}"
    ))?;

    // Get .m3u8 file that contains the video chunks
    let m3u8_in_text = get(&high_res_m3u8, id_token)
        .await
        .context("Failed to fetch playlist file!")?
        .text()
        .await
        .context("Failed to read contents of playlist file!")?;

    // get impartus key
    let key = get(&key_url, id_token)
        .await
        .context("Failed to fetch key!")?
        .bytes()
        .await
        .context("Failed to read key!")?
        .to_vec();

    // write it to .key file for ffmpeg to deal with it later
    let mut key_out = std::fs::File::create(&key_file_path)
        .context(format!("Failed to create `.key` file at {key_file_path}!"))?;

    key_out
        .write(&key)
        .context("Failed to write key contents to .key file!")?;

    key_out.flush().context("Failed to flush .key file!")?;

    drop(key_out);

    let mut m3u8_lines = m3u8_in_text.lines();

    let mut i = 0u32;

    let mut out = String::with_capacity(m3u8_in_text.len());

    // Side = 0 -> Parse first headers, side = 1 / 2: Different views
    let mut side = 0u8;

    // For later
    let number_of_ts_files = (m3u8_lines.clone().count() - 8) / 2;

    let mut perc_downloaded: f32 = 0f32;

    // Get the folder to store the .ts files
    let mut ts_store_location = std::path::Path::new(&temp).join("ts_store");

    // Create the folder if it does not exist
    std::fs::create_dir_all(&ts_store_location)
        .context(format!("Failed to create `ts_store` directory!"))?;

    // Process each .ts file and create a local copy of it and add it to the out string
    loop {
        // Assuming the .m3u8 file matches the spec, it will always follow #header\nuri\n
        let mut header = m3u8_lines
            .next()
            .context("Failed to read input playlist!")?;

        if header.starts_with("#EXTINF") && side != 2 {
            side = 1;
        }

        // Other view of the lecture
        if header.starts_with("#EXT-X-DISCONTINUITY") {
            out += header;
            out.push('\n');

            header = m3u8_lines.next().unwrap();
            side = 2;
        }

        // Stop if the playlist has ended
        if header.starts_with("#EXT-X-ENDLIST") {
            break;
        }

        // "Parse" first headers
        if side == 0 {
            if header.starts_with("#EXT-X-KEY:METHOD=") {
                // [#EXT-X-KEY:METHOD=AES-128],[URI="XXXX"]
                let key_method = header
                    .split(",")
                    .next()
                    .context("Failed to parse key method of recieved playlist file!")?;

                let ext_header = format!("{key_method},URI=\"{key_file_path}\"\n");

                out += &ext_header;
            } else {
                out += header;
                out.push('\n');
            }
            continue;
        }

        // The url to send a get request to
        let ts_url = m3u8_lines.next().unwrap();

        // Get the file-name of the .ts file
        let ts_store_location =
            ts_store_location.join(format!("tmp_ttid-{ttid}_{filename}_side-{side}_{i}.ts"));

        // Failable?
        let ts_store_path = ts_store_location.to_str().unwrap();

        // Attach original header and path to file that will be created next
        out += header;
        out.push('\n');
        out += ts_store_path;
        out.push('\n');

        i += 1;

        if let Ok(true) = std::fs::exists(&ts_store_location) {
            // TODO: Remove
            println!("Already downloaded `{ts_store_path}`. Skipping to next...");
            continue;
        }

        download_ts_file(ts_store_path, id_token, ts_url).await?;

        perc_downloaded = ((i as f32) / (number_of_ts_files as f32)) * 100.0f32;

        // TODO: Remove
        println!("[{ttid}] Now at {perc_downloaded}% to completion.");
    }

    // End playlist
    out += "#EXT-X-ENDLIST";

    let mut m3u8_out = std::fs::File::create(&m3u8_file_path).context(format!(
        "Failed to create playlist file at {m3u8_file_path}!"
    ))?;

    m3u8_out
        .write(out.as_bytes())
        .context("Failed to write to temporary playlist file!")?;

    m3u8_out
        .flush()
        .context("Failed to flush temporary playlist file!")?;

    drop(m3u8_out);

    // TODO: Remove
    println!("Output .m3u8 created at: `{m3u8_file_path}`");

    Ok(())
}

async fn download_ts_file(file_path: &str, id_token: &str, url: &str) -> Result<()> {
    let ts_data = get(url, id_token)
        .await
        .context("Failed to fetch video chunk!")?
        .bytes()
        .await
        .context("Failed to read video chunk!")?
        .to_vec();

    // Create a local copy of the .ts file
    let mut ts_store = std::fs::File::create(&file_path)
        .context(format!("Failed to create `.ts` file at {file_path}!"))?;

    // Populate the .ts file
    ts_store
        .write(&ts_data)
        .context("Failed to write video chunk!")?;

    ts_store.flush().context("Failed to flush video chunk!")?;

    drop(ts_store);

    Ok(())
}
