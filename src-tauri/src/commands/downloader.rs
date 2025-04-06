use std::{
    collections::HashMap, fmt::Display, future::Future, io::Write, sync::Arc, time::Duration,
};

use crate::prelude::*;

use tauri_plugin_http::reqwest::{self, Client};
use tokio::{io::AsyncWriteExt, task::JoinSet};
use tracing::info;

use std::sync::LazyLock;

use crate::commands::get_temp;

// A static instance of a client, so that just one client is used for all requests
static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
const BASE: &str = dotenvy_macro::dotenv!("BASE");
static REMOTES: LazyLock<Vec<&str>> =
    LazyLock::new(|| serde_json::from_str(dotenvy_macro::dotenv!("VITE_REMOTES")).unwrap());
static MAX_RETRY_COUNT: LazyLock<usize> =
    LazyLock::new(|| dotenvy_macro::dotenv!("MAX_RETRY_COUNT").parse().unwrap());

/// References static client to perform a GET request with the token auth header
async fn get(url: &str, id_token: &str) -> Result<reqwest::Response> {
    CLIENT
        .get(url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {id_token}"))
        .send()
        .await
        .context(format!("Failed to GET data from url \"{url}\"!"))
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Views {
    left: bool,
    right: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TrackInfo {
    tracks: HashMap<String, Vec<String>>,
    views: Views,
}

async fn check_available(url: &str) -> bool {
    // Check if the host returns anything - ie. it's available to download from
    // If this does not recieve a response, it's considered unavailable
    if let Ok(res) = CLIENT.head(url).timeout(Duration::new(5, 0)).send().await {
        if res.status().is_success() {
            return true;
        }
    }
    false
}

/// Call a function <MAX_RETRY_COUNT> times or until it succeeds, whichever is lower
async fn retry<T, O: Future<Output = Result<T>>, F: Fn() -> O>(
    function: F,
    name: &str,
) -> Result<T> {
    let mut error = None;

    for i in 1..=*MAX_RETRY_COUNT {
        match function().await {
            Err(err) => {
                info!(
                    "Task `{name}` failed {} time(s). Max retry count is {}. {}etrying again.",
                    i,
                    *MAX_RETRY_COUNT,
                    if i == *MAX_RETRY_COUNT { "Not r" } else { "R" }
                );
                error = Some(err)
            }
            Ok(v) => return Ok(v),
        };
    }

    Err(error.expect(
        "Failed to run retry function, and an error was not registered. This should be impossible",
    ))
}

#[instrument(fields(ttid))]
async fn select_base<'a>(ttid: usize) -> Result<&'a str> {
    info!("Parsing remote url json for {ttid}");

    info!("Finding fastest remote url for {ttid}");
    // Pick the fastest server to download from
    let mut set_base = "";
    // If the client failed to connect to any of the available hosts
    let mut failed = true;

    let mut set = JoinSet::new();
    for base in REMOTES.iter() {
        set.spawn(async move { (check_available(base).await, base) });
    }

    // Run all the ping-functions at the same time, and wait for the first successful response
    while let Some(res) = set.join_next().await {
        match res {
            Ok((is_available, base)) => {
                failed = !is_available;
                set_base = base;
                break;
            }

            Err(_) => failed = true,
        }
    }

    // Is 5s a good enough amount of time to decide if a server is unavailable?
    // Or should it try connecting to the default server again - waiting as long as it takes?
    if failed {
        return Err(anyhow::Error::msg(
            "Failed to connect to any hosts! Check your connection and try again.",
        ));
    }

    Ok(set_base)
}

// TODOS: Not in order of importance:
// 1. Improve error messages

/// Creates an m3u8 file referencing local unencrypted .ts files
pub async fn download_playlist(
    resolution: Resolution,
    base: Arc<Option<String>>,
    tx: tokio::sync::watch::Sender<f32>,
    id_token: &str,
    ttid: usize,
    filename: &str,
) -> Result<(String, Option<String>)> {
    // If a base has been dictated by settings
    let download_base = if let Some(base) = base.as_ref() {
        info!("Using {base} as per download settings to download from");
        base.as_str()
    } else {
        retry(async || select_base(ttid).await, "select_base").await?
    };

    info!("Selected remote: {download_base} for {ttid}");

    // {temp}/multipartus-downloader/Lecture_<lecture-ttid>
    let temp_location = get_temp().join(format!("Lecture_{ttid}"));

    let temp = temp_location.as_path().to_str().unwrap_or("./tmp");

    info!("Creating temp directory at {temp}");

    // Create this temp location if it doesn't exist
    std::fs::create_dir_all(temp)
        .context(format!("Failed to create temporary directory {}!", temp))?;

    info!("Created temp directory at {temp}");

    // URLs to get data from
    let m3u8_info = format!("{BASE}/impartus/ttid/{ttid}/m3u8/info");
    let key_url = format!("{BASE}/impartus/ttid/{ttid}/key");

    // Temp locations to store the files used for generating outputs
    let m3u8_side1_file_path = format!("{temp}/{filename}_side_1.m3u8");
    let m3u8_side2_file_path = format!("{temp}/{filename}_side_2.m3u8");
    let key_file_path = format!("{temp}/{filename}.key.key");

    info!("Fetching index playlist file for {ttid}");

    // I hope you love these beautiful waterfalls @TheComputerM :)
    // Get impartus .m3u8 file
    let m3u8_index_bytes = retry(
        async || {
            get(&m3u8_info, id_token)
                .await
                .context("Failed to fetch index playlist file!")?
                .bytes()
                .await
                .context("Failed to read contents of playlist info file!")
        },
        "Get m3u8 index bytes",
    )
    .await?;

    info!("Fetched playlist json, now parsing it for {ttid}");

    let m3u8_tracks = serde_json::from_slice::<TrackInfo>(&m3u8_index_bytes)
        .context("Failed to parse track json!")?;

    info!("Finished parsing playlist json file for {ttid}");

    // TODO: Use second url in conjuction with local / online bases depending on connectivity
    // Select the correct resolution
    let selected_m3u8 = {
        let address = if let Resolution::HighRes = resolution {
            m3u8_tracks
                .tracks
                .get("1280x720")
                .context("Failed to get 1280x720p video playlist")?
                .last()
                .context("Failed to get first link in 1280x720p video playlist")?
        } else {
            m3u8_tracks
                .tracks
                .get("854x480")
                .context("Failed to get 854x480 video playlist")?
                .last()
                .context("Failed to get first link in 854x480 video playlist")?
        }
        .clone();
        download_base.to_string() + "/api/fetchvideo?tag=LC&inm3u8=" + &address
    };

    info!("Selected playlist file url: {selected_m3u8} for {ttid}");

    info!("Fetching main playlist file for {ttid}");

    // Get .m3u8 file that contains the video chunks
    let m3u8_in_text = retry(
        async || {
            get(&selected_m3u8, id_token)
                .await
                .context("Failed to fetch playlist file!")?
                .text()
                .await
                .context("Failed to read contents of playlist file!")
        },
        "Get m3u8 playlist file",
    )
    .await?;

    info!("Fetched main playlist file. Fetching key file for {ttid}");

    // get impartus key
    let key = retry(
        async || {
            get(&key_url, id_token)
                .await
                .context("Failed to fetch key!")?
                .bytes()
                .await
                .context("Failed to read key!")
        },
        "Get key file for decrypting incoming chunks",
    )
    .await?
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

    let m3u8_line_count = m3u8_lines.clone().count();

    let number_of_ts_files = (if m3u8_line_count > 8 {
        m3u8_line_count
    } else {
        8
    } - 8)
        / 2;
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
            header = m3u8_lines.next().context("Expected a new line in the playlist file, but found nothing! Maybe the video is corrupted?")?;
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

                // TODO: Check if this has any problems
                let ext_header = format!(
                    "{key_method},URI=\"{}\"\n",
                    key_file_path.replace("\\", "\\\\")
                );

                out_1 += &ext_header;
            } else {
                out_1 += header;
                out_1.push('\n');
            }
            continue;
        }

        // The url to send a get request to
        let ts_url = m3u8_lines.next().context("Expected a new line in the playlist file, but found nothing! Maybe the video is corrupted?")?;

        // Get the file-name of the .ts file
        let ts_store_location = ts_store_location.join(format!(
            "tmp_ttid_{ttid}_{filename}_side_{side}_{i}_{resolution}.ts"
        ));

        let ts_store_path = ts_store_location
            .to_str()
            .context("Failed to find download location for temp media file!")?;

        let out = if side == 1 { &mut out_1 } else { &mut out_2 };

        // Attach original header and path to file that will be created next
        *out += header;
        out.push('\n');
        *out += ts_store_path;
        out.push('\n');

        i += 1;

        perc_downloaded = ((i as f32) / (number_of_ts_files as f32)) * 100.0f32;

        // Re-downloads if io-error
        if let Ok(true) = tokio::fs::try_exists(&ts_store_location).await {
            // There's no need to have an error occur if the progress cannot be reported
            tx.send(perc_downloaded).unwrap_or(());
            info!("The file at `{ts_store_path}` already exists. It likely has been downloaded previously. Skipping to next file");
            continue;
        }

        download_ts_file(ts_store_path, id_token, ts_url).await?;

        tx.send(perc_downloaded).unwrap_or(());
    }

    // End playlist
    out_1 += "#EXT-X-ENDLIST";
    out_2 += "#EXT-X-ENDLIST";

    // Could also check against the existence of the side 2 file path
    if m3u8_tracks.views.left {
        info!("Output .m3u8 playlist created at `{m3u8_side1_file_path}` (side 1) for {ttid}");
        write_m3u8(&m3u8_side1_file_path, out_1).await?;
    }

    if m3u8_tracks.views.right {
        info!("Output .m3u8 playlist created at `{m3u8_side2_file_path}` (side 2) for {ttid}");
        write_m3u8(&m3u8_side2_file_path, out_2).await?;
    }

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
    let ts_data = retry(
        async || {
            get(url, id_token)
                .await
                .context("Failed to fetch video chunk!")?
                .bytes()
                .await
                .context("Failed to read video chunk!")
        },
        "Get chunk data",
    )
    .await?
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
