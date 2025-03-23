use std::{fmt::Display, io::Write};

use anyhow::{Context, Result};

use log::info;
use tauri_plugin_http::reqwest::{self, Client};
use tokio::io::AsyncWriteExt;

use std::sync::LazyLock;

// A static instance of a client, so that just one client is used for all requests
static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
const BASE: &str = dotenvy_macro::dotenv!("BASE");

/// References static client to perform a GET request with the token auth header
async fn get(url: &str, id_token: &str) -> Result<reqwest::Response> {
    CLIENT
        .get(url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {id_token}"))
        .send()
        .await
        .context(format!("Failed to GET data from url \"{url}\"!"))
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum Resolution {
    /// 480p
    LowRes,
    /// 720p
    HighRes,
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if let Self::HighRes = self {
                "high_res"
            } else {
                "low_res"
            }
        )
    }
}

// TODOS: Not in order of importance:
// 1. Select resoultion of download
// 2. Improve error messages
// 4. Remove / reduce debug logging

/// Creates an m3u8 file referencing local unencrypted .ts files
pub async fn download_playlist(
    resolution: Resolution,
    tx: tokio::sync::watch::Sender<f32>,
    id_token: &str,
    ttid: usize,
    filename: &str,
) -> Result<(String, Option<String>)> {
    // {temp}/multipartus-downloader/Lecture-<lecture-ttid>
    let temp_location = std::env::temp_dir()
        .join("multipartus-downloader")
        .join(format!("Lecture-{ttid}"));

    let temp = temp_location.as_path().to_str().unwrap_or("./tmp");

    info!("Creating temp directory at {temp}");

    // Create this temp location if it doesn't exist
    std::fs::create_dir_all(temp)
        .context(format!("Failed to create temporary directory {}!", temp))?;

    info!("Created temp directory at {temp}");

    // URLs to get data from
    let m3u8_url = format!("{BASE}/impartus/ttid/{ttid}/m3u8");
    let key_url = format!("{BASE}/impartus/ttid/{ttid}/key");

    // Temp locations to store the files used for generating outputs
    let m3u8_side1_file_path = format!("{temp}/{filename}_side_1.m3u8");
    let m3u8_side2_file_path = format!("{temp}/{filename}_side_2.m3u8");
    let key_file_path = format!("{temp}/{filename}.key.key");

    info!("Fetching index playlist file for {ttid}");

    // I hope you love these beautiful waterfalls @TheComputerM :)
    // Get impartus .m3u8 file
    let m3u8_index_text = get(&m3u8_url, id_token)
        .await
        .context("Failed to fetch index playlist file!")?
        .text()
        .await
        .context("Failed to read contents of index playlist file!")?;

    info!("Fetched index playlist file for {ttid}");

    // Get both resoultions
    let m3u8_1 = m3u8_index_text.lines().nth(2).context(format!(
        "Failed to get playlist file data! {m3u8_index_text}"
    ))?;

    let m3u8_2 = m3u8_index_text.lines().nth(4).context(format!(
        "Failed to get playlist file data! {m3u8_index_text}"
    ))?;

    // Find the correct resolution
    let (high_res_m3u8, low_res_m3u8) = if m3u8_1.contains("F1280x720") {
        (m3u8_1, m3u8_2)
    } else {
        (m3u8_2, m3u8_1)
    };

    // Select the correct resolution
    let selected_m3u8 = if let Resolution::HighRes = resolution {
        high_res_m3u8
    } else {
        low_res_m3u8
    };

    info!("Fetching main playlist file for {ttid}");

    // Get .m3u8 file that contains the video chunks
    let m3u8_in_text = get(selected_m3u8, id_token)
        .await
        .context("Failed to fetch playlist file!")?
        .text()
        .await
        .context("Failed to read contents of playlist file!")?;

    info!("Fetched main playlist file. Fetching key file for {ttid}");

    // get impartus key
    let key = get(&key_url, id_token)
        .await
        .context("Failed to fetch key!")?
        .bytes()
        .await
        .context("Failed to read key!")?
        .to_vec();

    info!("Fetched key file. Opening key file for {ttid}");

    // write it to .key file for ffmpeg to deal with it later
    let mut key_out = std::fs::File::create(&key_file_path)
        .context(format!("Failed to create `.key` file at {key_file_path}!"))?;

    key_out
        .write(&key)
        .context("Failed to write key contents to .key file!")?;

    key_out.flush().context("Failed to flush .key file!")?;

    drop(key_out);

    info!("Created key file for {ttid}");

    let mut m3u8_lines = m3u8_in_text.lines();

    let mut i = 0u32;

    let mut out_1 = String::with_capacity(m3u8_in_text.len() / 2);
    let mut out_2 = String::with_capacity(m3u8_in_text.len() / 2);

    // Side = 0 -> Parse first headers, side = 1 / 2: Different views
    let mut side = 0u8;

    let number_of_ts_files = (m3u8_lines.clone().count() - 8) / 2;
    let mut perc_downloaded;

    // Get the folder to store the .ts files
    let ts_store_location = std::path::Path::new(&temp).join("ts_store");

    // Create the folder if it does not exist
    std::fs::create_dir_all(&ts_store_location)
        .context("Failed to create `ts_store` directory!")?;

    let mut side2_file_path = None;

    // Process each .ts file and create a local copy of it and add it to the out string
    loop {
        // Assuming the .m3u8 file matches the spec, it will always follow #header\nuri\n
        let mut header = m3u8_lines
            .next()
            .context("Failed to read input playlist!")?;

        if side == 0 && header.starts_with("#EXTINF") {
            info!("Parsed headers of playlist. Switching to side 1");
            // Copy headers to side 2
            out_2 = out_1.clone();
            side = 1;
        }

        // Other view of the lecture
        if header.starts_with("#EXT-X-DISCONTINUITY") {
            info!("Finished parsing side 1, starting side 2");
            header = m3u8_lines.next().unwrap();
            side = 2;
            side2_file_path = Some(m3u8_side2_file_path.clone());
        }

        // Stop if the playlist has ended
        if header.starts_with("#EXT-X-ENDLIST") {
            info!("Finished parsing playlist");
            break;
        }

        // "Parse" first headers
        if side == 0 {
            info!("Parsing header {header}");
            if header.starts_with("#EXT-X-KEY:METHOD=") {
                // [#EXT-X-KEY:METHOD=AES-128],[URI="XXXX"]
                let key_method = header
                    .split(",")
                    .next()
                    .context("Failed to parse key method of recieved playlist file!")?;

                let ext_header = format!("{key_method},URI={key_file_path:?}\n");

                out_1 += &ext_header;
            } else {
                out_1 += header;
                out_1.push('\n');
            }
            continue;
        }

        // The url to send a get request to
        let ts_url = m3u8_lines.next().unwrap();

        // Get the file-name of the .ts file
        let ts_store_location = ts_store_location.join(format!(
            "tmp_ttid_{ttid}_{filename}_side_{side}_{i}_{resolution}.ts"
        ));

        // Failable?
        let ts_store_path = ts_store_location.to_str().unwrap();

        let out = if side == 1 { &mut out_1 } else { &mut out_2 };

        // Attach original header and path to file that will be created next
        *out += header;
        out.push('\n');
        *out += ts_store_path;
        out.push('\n');

        i += 1;

        // Re-downloads if io-error
        if let Ok(true) = tokio::fs::try_exists(&ts_store_location).await {
            info!("The file at `{ts_store_path}` already exists. It likely has been downloaded previously. Skipping to next file");
            continue;
        }

        download_ts_file(ts_store_path, id_token, ts_url).await?;

        perc_downloaded = ((i as f32) / (number_of_ts_files as f32)) * 100.0f32;

        // There's no need to have an error occur if the progress cannot be reported
        tx.send(perc_downloaded).unwrap_or(());
    }

    // End playlist
    out_1 += "#EXT-X-ENDLIST";
    out_2 += "#EXT-X-ENDLIST";

    write_m3u8(&m3u8_side1_file_path, out_1).await?;
    write_m3u8(&m3u8_side2_file_path, out_2).await?;

    // TODO: Remove
    info!("Output .m3u8 plalists created at `{m3u8_side1_file_path}` (side 1), `{m3u8_side2_file_path}` (side 2)");

    Ok((m3u8_side1_file_path, side2_file_path))
}

async fn write_m3u8(filepath: &String, out: String) -> Result<()> {
    let mut m3u8_out = tokio::fs::File::create(&filepath)
        .await
        .context(format!("Failed to create playlist file at {filepath}!"))?;

    m3u8_out
        .write(out.as_bytes())
        .await
        .context("Failed to write to temporary playlist file!")?;

    m3u8_out
        .flush()
        .await
        .context("Failed to flush temporary playlist file!")?;

    drop(m3u8_out);

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
    let mut ts_store = tokio::fs::File::create(&file_path)
        .await
        .context(format!("Failed to create `.ts` file at {file_path}!"))?;

    // Populate the .ts file
    ts_store
        .write(&ts_data)
        .await
        .context("Failed to write video chunk!")?;

    ts_store
        .flush()
        .await
        .context("Failed to flush video chunk!")?;

    Ok(())
}
