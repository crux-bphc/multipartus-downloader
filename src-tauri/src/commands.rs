use aes::{
    cipher::{generic_array::GenericArray, BlockDecrypt, KeyInit},
    Aes128Dec,
};
use tauri_plugin_http::reqwest;

const BASE_URL: &str = "https://lex.crux-bphc.com/api/impartus";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Video {
    ttid: i32,
    topic: String,
    number: i32
}

async fn get_m3u8(client: &reqwest::Client, ttid: i32) -> Result<String, String> {
    let master_playlist = client
        .get(format!("{BASE_URL}/ttid/{ttid}/m3u8"))
        .send()
        .await
        .map_err(|_| String::from("Failed to fetch m3u8"))?
        .text()
        .await
        .map_err(|_| String::from("Failed to read m3u8"))?;

    println!("{master_playlist}");
    let high_res_m3u8 = master_playlist.lines().nth(2).unwrap();
    println!("{high_res_m3u8}");

    let chunk = client
        .get(high_res_m3u8)
        .send()
        .await
        .map_err(|_| String::from("Failed to fetch chunks"))?
        .text()
        .await
        .map_err(|_| String::from("Failed to read chunks"))?;

    return Ok(chunk);
}

// fn decrypt(key: bytes::Bytes, block: bytes::Bytes) -> Vec<u8> {
//     let cipher = Aes128Dec::new_from_slice(key.iter().as_slice()).expect("Invalid Length");
//     let mut block = GenericArray::clone_from_slice(block.iter().as_slice());
//     cipher.decrypt_block(&mut block);
//     return block.to_vec();
// }



#[tauri::command]
pub async fn download(token: String, folder: String, videos: Vec<Video>) -> Result<(), String> {
    // Temp files: {{folder}}/tmp/...
    // Lecture videos: {{folder}}/Lecture {number} - {topic}.mp4

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
        let _decryption_key = client
            .get(format!("{BASE_URL}/ttid/{ttid}/key", ttid = video.ttid))
            .send()
            .await
            .map_err(|_| String::from("Failed to fetch decryption key"))?
            .bytes()
            .await
            .map_err(|_| String::from("Failed to read decryption key"))?;


        let chunk = get_m3u8(&client, video.ttid).await?;




        println!("{chunk}");
        // decrypt(decryption_key);
    }
    Ok(())
}
