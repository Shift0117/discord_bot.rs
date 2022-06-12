use std::{
    fs::{self, read_dir, remove_dir_all, File},
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

use reqwest;
use serenity::{
    client::Context,
    model::{channel::Message},
};
use tempfile::{self, NamedTempFile};



const BASE_URL: &str = "http://127.0.0.1:50031";
pub async fn play_voice(ctx: &Context, msg: Message) {


    let mut temp_file = tempfile::Builder::new().suffix(".wav").rand_bytes(5).tempfile().unwrap();
    create_voice(msg.content.clone(), &mut temp_file).await;
    dbg!(&msg.content);
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let (_, path) = temp_file.keep().unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let mut source = songbird::ffmpeg(&path).await.unwrap();
        source.metadata.source_url = Some(path.to_string_lossy().to_string());
        handler.enqueue_source(source.into());

    } else {
    }
}

async fn create_voice(text: String, temp_file: &mut NamedTempFile) {
    let params = [("text", text), ("speaker", 1.to_string())];
    let client = reqwest::Client::new();
    let voice_query_url = format!("{}/audio_query", BASE_URL);
    dbg!(&voice_query_url);
    let res = client
        .post(voice_query_url)
        .query(&params)
        .send()
        .await
        .expect("Panic in audio query");
    println!("{}", res.status());
    let synthesis_body = res.text().await.expect("Panic in get body");
    let synthesis_arg = [("speaker", 1i16)];
    let synthesis_url = format!("{}/synthesis", BASE_URL);
    let synthesis_res = client
        .post(synthesis_url)
        .body(synthesis_body)
        .query(&synthesis_arg)
        .send()
        .await
        .expect("Panic in synthesis query");
    dbg!(&synthesis_res.status());
    temp_file
        .write(&synthesis_res.bytes().await.unwrap())
        .unwrap();
}
