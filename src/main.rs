pub mod backblaze_api;
pub mod consts;
pub mod models;

use std::{path::PathBuf, sync::Arc, time::Duration};

use backblaze_api::BackblazeApi;
use rand::seq::SliceRandom;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api = Arc::new(BackblazeApi::new().await?);
    // upload_files(api, 100).await?;
    let files = api.list_files().await?;
    let downloads = 100;
    let mut durations = vec![Duration::default(); downloads];
    for idx in 0..downloads {
        let file_name = files.choose(&mut rand::thread_rng()).unwrap();
        let path = PathBuf::from(file_name);
        durations[idx] = api.download_file_by_name(file_name, path).await?;
    }

    println!("{durations:?}");

    let total = durations
        .into_iter()
        .map(|d| d.as_millis() as f32)
        .sum::<f32>();
    let avg = total / (downloads as f32);
    println!("average duration: {:?}", avg);

    Ok(())
}

async fn upload_files(api: Arc<BackblazeApi>, n: usize) -> anyhow::Result<()> {
    for _ in 0..n {
        let api = api.clone();
        let audio = PathBuf::from("audio.ogg");
        let key = uuid::Uuid::new_v4().to_string();
        println!("uploading {key}");
        let upload_info = api.get_upload_info().await.unwrap();
        api.upload_file(&upload_info, audio, key)
            .await
            .expect("upload failed");
    }
    Ok(())
}
