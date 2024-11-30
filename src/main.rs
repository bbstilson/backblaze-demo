pub mod backblaze_api;
pub mod consts;
pub mod models;

use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use backblaze_api::BackblazeApi;
use consts::BUCKET;
use rand::seq::SliceRandom;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // pick one
    // native_api_example().await?;
    // s3_sdk_example().await?;

    Ok(())
}

async fn s3_sdk_example() -> anyhow::Result<()> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);

    let r = client
        .list_objects_v2()
        .bucket(BUCKET)
        .max_keys(1000)
        .prefix("audio")
        .send()
        .await?;
    let keys = r
        .contents()
        .into_iter()
        .filter_map(|c| c.key())
        .collect::<Vec<_>>();

    let downloads = 100;
    let mut durations = vec![Duration::default(); downloads];
    for idx in 0..downloads {
        let key = keys.choose(&mut rand::thread_rng()).unwrap();
        let path = PathBuf::from(key);

        let start = Instant::now();
        let r = client.get_object().bucket(BUCKET).key(*key).send().await?;

        let mut f = File::create(path)?;
        let mut data = r.body;
        while let Some(bytes) = data.try_next().await? {
            f.write_all(&bytes)?;
        }
        durations[idx] = start.elapsed();
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

async fn native_api_example() -> anyhow::Result<()> {
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
