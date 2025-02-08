use std::{io::Write, str::Bytes};

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use tauri_plugin_http::reqwest;

#[derive(Debug)]
pub struct Error {
    error_string: String,
}

impl Into<Error> for std::io::Error {
    fn into(self) -> Error {
        Error {
            error_string: self.to_string(),
        }
    }
}

impl Into<Error> for reqwest::Error {
    fn into(self) -> Error {
        Error {
            error_string: self.to_string(),
        }
    }
}

/// Creates an m3u8 file referencing local unencrypted .ts files
///
/// The `.m3u8` file is located at `<location>/tmp_playlist_<id>.m3u8`
///
/// Each `.ts` file is located at `<location>/ts_store/tmp_ts_<id>_<ts_number: 0..n>.ts`
pub async fn generate_m3u8_tmp_file(
    key: &[u8],
    url: &str,
    location: &str,
    id: usize,
) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let file_path = format!("{location}/tmp_playlist_{id}.m3u8");

    let mut file = match std::fs::File::create(&file_path) {
        Err(error) => {
            println!("Failed to create `.m3u8` file at {file_path}! Error: {error}");
            return Err(error.into());
        }

        Ok(file) => file,
    };

    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(error) => return Err(error.into()),
    };

    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => return Err(error.into()),
    };

    // First 6 lines are headers
    let mut m3u8_lines = text.lines().skip(6);
    let mut i = 0;

    let mut out = String::with_capacity(text.len());

    // Write headers to file
    out.push_str(
        r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-MEDIA-SEQUENCE:0
#EXT-X-ALLOW-CACHE:YES
#EXT-X-TARGETDURATION:11
#EXT-X-KEY:METHOD=NONE"#,
    );

    // Process each .ts file and create a local, unencrypted copy of it and add it to the out string
    loop {
        // Assuming the .m3u8 file matches the spec, it will always follow #header\nuri\n
        let header = m3u8_lines.next().unwrap();

        // Stop if the playlist has ended
        if header.starts_with("#EXT-X-ENDLIST") {
            break;
        }

        // The url to send a get request to
        let ts_url = m3u8_lines.next().unwrap();

        // Get the .ts file
        let response = match client.get(ts_url).send().await {
            Ok(response) => response,
            Err(error) => return Err(error.into()),
        };

        let bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(error) => return Err(error.into()),
        };

        let mut byte_array = bytes.to_vec();

        // Decrypt the bytes in the .ts file
        type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
        let decryptor = Aes128CbcDec::new(key.into(), (&[0; 16]).into());

        let decrypted_bytes = match decryptor.decrypt_padded_mut::<Pkcs7>(&mut byte_array) {
            Ok(bytes) => bytes,
            Err(error) =>
                return Err(Error {
                    error_string: error.to_string(),
                }),
        };

        let ts_store_location = format!("{location}/ts_store/tmp_ts_{id}_{i}.ts");

        // Create a local unencrypted copy of the .ts file
        let mut ts_store = match std::fs::File::create(&ts_store_location) {
            Err(error) => {
                println!("Failed to create `.ts` file at {ts_store_location}! Error: {error}");
                return Err(error.into());
            }

            Ok(file) => file,
        };

        if let Err(error) = ts_store.write(decrypted_bytes) {
            return Err(error.into());
        };

        ts_store.flush().expect("Failed to flush .ts file!");

        // Attach original header and path to newly created file
        out += header;
        out += &ts_store_location;

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
