use std::io::Write;

use aes::{cipher::{block_padding::ZeroPadding, BlockDecryptMut, KeyInit}, Aes128};
use tauri_plugin_http::{reqwest, reqwest::Response};


// TODO: Extract this into an .env variable
const BASE: &str = "https://lex.crux-bphc.com/api";

// TODO: Change this?
const M3U8_LOCATION: &str = "./";

// TODO: Extract this into an .env variable
const ID_TOKEN: &str = "Put your id here for now";

// Debug
const LOGS: bool = true;

#[derive(Debug)]
pub struct Error {
    error_string: String,
}

impl From<std::io::Error> for Error {
    fn from(val: std::io::Error) -> Self {
        Error {
            error_string: val.to_string(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(val: reqwest::Error) -> Self {
        Error {
            error_string: val.to_string(),
        }
    }
}

/// Converts 16 bytes to 14, so this doesn't work here. Idk how it worked in Go
/// 
/// https://github.com/pnicto/impartus-video-downloader/blob/main/impartus.go#L241
fn get_decryption_key(mut encryption_key: Vec<u8>) -> Vec<u8> {
	encryption_key.drain(0..2);
    let (mut i, mut j) = (0, encryption_key.len() - 1);
	loop  {
        encryption_key.swap(i, j);
        i += 1;
        j -= 1;
        if i >= j { break; }
	}

	encryption_key
}

async fn authenticated_request(client: &reqwest::Client, url: &str, token: &str) -> Result<Response, Error>{
    let request = 
    client
        .get(url)
        .header("authorization", format!("Bearer {token}"))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0");
    match request.send().await {
        Ok(response) => Ok(response),
        Err(error) => Err(error.into()),
    }
}

async fn authenticated_result_bytes(client: &reqwest::Client, url: &str, token: &str) -> Result<Vec<u8>, Error> {
    let response = authenticated_request(client, url, token).await?;
    let status = response.status();

    match response.bytes().await {
        Ok(bytes) => {
            if status == 200 { Ok(bytes.to_vec()) }
            else { Err(Error { error_string: format!("Failed to fetch data! Status code: {status}") }) }
        },
        Err(error) => Err(error.into())
    }
}

async fn authenticated_result_text(client: &reqwest::Client, url: &str, token: &str) -> Result<String, Error> {
    let response = authenticated_request(client, url, token).await?;
    let status = response.status();

    match response.text().await {
        Ok(text) => {
            if status == 200 { Ok(text) }
            else { Err(Error { error_string: text }) }
        },
        Err(error) => Err(error.into())
    }
}

/// Creates an m3u8 file referencing local unencrypted .ts files
///
/// The `.m3u8` file is located at `<LOCATION>/<filename>.m3u8`
///
/// Each `.ts` file is located at `<LOCATION>/ts_store/tmp_<filename>_<ts_number: 0..n>.ts`
pub async fn generate_m3u8_tmp_file(
    ttid: usize,
    filename: &str,
) -> Result<(), Error> {

    // URLs to get data from
    let url = format!("{BASE}/impartus/ttid/{ttid}/m3u8");
    let key_url = format!("{BASE}/impartus/ttid/{ttid}/key");

    let client = reqwest::Client::new();

    let file_path = format!("{M3U8_LOCATION}/{filename}.m3u8");

    let mut file = match std::fs::File::create(&file_path) {
        Err(error) => {
            println!("Failed to create `.m3u8` file at {file_path}! Error: {error}");
            return Err(error.into());
        }

        Ok(file) => file,
    };

    // Get impartus .m3u8 file
    let text = authenticated_result_text(&client, &url, ID_TOKEN).await?;
    
    let high_res_m3u8 = text.lines().nth(2).unwrap();

    if LOGS { println!("High res m3u8 is {high_res_m3u8}"); }

    let text = authenticated_result_text(&client, high_res_m3u8, ID_TOKEN).await?;

    // get impartus key
    let key= authenticated_result_bytes(&client, &key_url, ID_TOKEN).await?;

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
        if LOGS { println!("{line}"); }
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

        if LOGS { println!("header {header}"); }

        // The url to send a get request to
        let ts_url = m3u8_lines.next().unwrap();
    
        if LOGS { println!("ts_url {ts_url}"); }

        let mut ts_store_location = 
            std::path::Path::new(M3U8_LOCATION)
                .join("ts_store");

        std::fs::create_dir_all(&ts_store_location).unwrap();

        ts_store_location.push(format!("tmp_ttid-{ttid}_{filename}_side-{side}_{i}.ts"));

        let ts_store_name = ts_store_location.to_str().unwrap();

        if let Ok(true) = std::fs::exists(&ts_store_location) {
            if LOGS { println!("Already downloaded. Skipping to next..."); }
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

        let mut ts_data = authenticated_result_bytes(&client, ts_url, ID_TOKEN).await?;
        
        // Decrypt the file
        decrypt(&mut ts_data, key.as_slice())?;    

        if let Err(error) = ts_store.write(&ts_data) {
            return Err(error.into());
        };

        ts_store.flush().expect("Failed to flush .ts file!");

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

    if let Err(error) = file.write(out.as_bytes()) {
        return Err(error.into());
    };

    file.flush().expect("Failed to flush file!");
    drop(file);

    Ok(())
}


fn decrypt(bytes: &mut Vec<u8>, key: &[u8]) -> Result<(), Error> {
    let extra_length = 16 - (bytes.len() % 16);
    let mut extra_bytes = [extra_length as u8].repeat(extra_length);

    bytes.append(&mut extra_bytes);

    let decryptor = match Aes128::new_from_slice(key) {
        Ok(value) => value,
        Err(error) => return Err(Error{ error_string: error.to_string() })
    };

    match decryptor.decrypt_padded_mut::<ZeroPadding>(bytes) {
        Ok(_) => (),
        Err(error) => return Err(Error{ error_string: error.to_string() }),
    };
    Ok(())
}