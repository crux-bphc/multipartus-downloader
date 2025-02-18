use std::{fmt::Display, io::Write};

use anyhow::{Context, Result};

use aes::{cipher::{block_padding::ZeroPadding, BlockDecryptMut, KeyIvInit}, Aes128};
use tauri_plugin_http::{reqwest, reqwest::header::HeaderMap};

// TODO: Extract this into an .env variable
const BASE: &str = "https://lex.crux-bphc.com/api";

// TODO: Change this?
const M3U8_LOCATION: &str = "./";

// TODO: Extract this into an .env variable
const ID_TOKEN: &str = "";

#[derive(Debug)]
pub struct DownloadProcessError {
    error_string: String,
}

impl DownloadProcessError {
    fn new(message: String) -> Self {
        Self { error_string: message }
    }
}

impl Display for DownloadProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_string)
    }
}

impl std::error::Error for DownloadProcessError {}

/// Creates an m3u8 file referencing local unencrypted .ts files
pub async fn generate_m3u8_tmp_file(
    ttid: usize,
    filename: &str,
) -> Result<()> {

    // URLs to get data from
    let url = format!("{BASE}/impartus/ttid/{ttid}/m3u8");
    let key_url = format!("{BASE}/impartus/ttid/{ttid}/key");

    let mut default_headers = HeaderMap::new();
    default_headers.append("authorization", format!("Bearer {ID_TOKEN}").parse().unwrap());

    let client = 
        reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .map_err(|val| DownloadProcessError::new(format!("Error creating client! {val}")))?;

    let file_path = format!("{M3U8_LOCATION}/{filename}.m3u8");

    let mut file = match std::fs::File::create(&file_path) {
        Err(error) => {
            println!("Failed to create `.m3u8` file at {file_path}! Error: {error}");
            return Err(error.into());
        }

        Ok(file) => file,
    };

    // Get impartus .m3u8 file
    let text = 
        client
            .get(url)
            .send().await?
            .text().await?;
    let high_res_m3u8 = 
        text
            .lines()
            .nth(2)
            .context("Failed to get file data!")?;

    let text = 
        client
            .get(high_res_m3u8)
            .send().await?
            .text().await?;

    // get impartus key
    let key= 
        client 
            .get(key_url)
            .send().await?
            .bytes().await?
            .to_vec();

    // First 6 lines are headers
    let mut m3u8_lines = text.lines();
    let mut i = 0;

    let mut out = String::with_capacity(text.len());

    let mut side = 1u8;

    // "parse" the first 6 lines. These are "headers"?
    // TODO: Add a loop that does not parse just 6 lines?
    for _ in 0..6 {
        // Assuming the file is valid and has at least 6 lines [Dies otherwise :)]
        let line = m3u8_lines.next().unwrap();
        if line.starts_with("#EXT-X-KEY:METHOD=") {
            // Have no encryption enabled
            out.push_str("#EXT-X-KEY:METHOD=NONE\n");
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    // Process each .ts file and create a local, unencrypted copy of it and add it to the out string
    loop {
        // Assuming the .m3u8 file matches the spec, it will always follow #header\nuri\n
        let mut header = m3u8_lines.next().unwrap();

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

        // The url to send a get request to
        let ts_url = m3u8_lines.next().unwrap();
        let mut ts_store_location = 
            std::path::Path::new(M3U8_LOCATION)
                .join("ts_store");

        std::fs::create_dir_all(&ts_store_location).unwrap();

        ts_store_location.push(format!("tmp_ttid-{ttid}_{filename}_side-{side}_{i}.ts"));

        let ts_store_name = ts_store_location.to_str().unwrap();

        if let Ok(true) = std::fs::exists(&ts_store_location) {
            println!("Already downloaded. Skipping to next...");
            i += 1;
            continue;
        }

        // Create a local unencrypted copy of the .ts file
        let mut ts_store = match std::fs::File::create(&ts_store_location) {
            Err(error) => {
                println!("Failed to create `.ts` file at {ts_store_name}! Error: {error}");
                return Err(error.into());
            }

            Ok(file) => file,
        };

        let mut ts_data = 
            client
                .get(ts_url)
                .send().await?
                .bytes().await?
                .to_vec();
        
        // Decrypt the file
        decrypt(&mut ts_data, key.as_slice())?;    

        ts_store.write(&ts_data)?;

        ts_store.flush().context("Failed to flush .ts file!")?;

        // Attach original header and path to newly created file
        out += header;
        out.push('\n');
        out += ts_store_name;
        out.push('\n');

        drop(ts_store);

        i += 1;
    }

    // End playlist
    out += "#EXT-X-ENDLIST";

    file.write(out.as_bytes())?;

    file.flush().context("Failed to flush file!")?;
    drop(file);

    Ok(())
}


fn decrypt(bytes: &mut Vec<u8>, key: &[u8]) -> Result<()> {
    let extra_length = 16 - (bytes.len() % 16);
    let mut extra_bytes = [extra_length as u8].repeat(extra_length);

    bytes.append(&mut extra_bytes);

    type Aes128Cbc = cbc::Decryptor<Aes128>;

    let decryptor = 
        Aes128Cbc::new_from_slices(key, &[0; 16])
            .map_err(|err| DownloadProcessError::new(format!("{err}")))?;

    match decryptor.decrypt_padded_mut::<ZeroPadding>(bytes) {
        Ok(_) => (),
        Err(error) => return Err(DownloadProcessError{ error_string: error.to_string() }.into()),
    };

    Ok(())
}