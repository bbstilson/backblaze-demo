use std::{
    fmt::Debug,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use reqwest::header::{HeaderMap, HeaderValue};
use sha1::{Digest, Sha1};

use crate::{consts::*, models::*};

pub struct BackblazeApi {
    client: reqwest::Client,
    versionless_api_url: String,
    api_url: String,
    bucket_id: String,
    bucket: String,
    prefix: String,
}

impl Debug for BackblazeApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "api_url = {}\nbucket_id = {}\nbucket = {}\nprefix = {}",
            self.api_url, self.bucket_id, self.bucket, self.prefix
        )
    }
}

impl BackblazeApi {
    pub async fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::new();

        // https://www.backblaze.com/apidocs/b2-authorize-account
        let r = client
            .get(format!("{BASE_URL}/{API_VERSION}/b2_authorize_account"))
            .basic_auth(KEY_ID, Some(KEY))
            .send()
            .await?
            .json::<AuthorizeAccount>()
            .await?;

        let storage_api = r.api_info.storage_api;

        let mut headers = HeaderMap::new();
        let mut auth_value: HeaderValue = r.authorization_token.parse().unwrap();
        auth_value.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, auth_value);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            versionless_api_url: storage_api.api_url.clone(),
            api_url: format!("{}/{API_VERSION}", storage_api.api_url),
            bucket_id: storage_api.bucket_id,
            bucket: storage_api.bucket_name,
            prefix: storage_api.name_prefix,
        })
    }

    pub async fn get_upload_info(&self) -> anyhow::Result<GetUploadUrl> {
        // https://www.backblaze.com/apidocs/b2-get-upload-url
        let r = self
            .client
            .get(format!("{}/b2_get_upload_url", self.api_url))
            .query(&[("bucketId", &self.bucket_id)])
            .send()
            .await?
            .json::<GetUploadUrl>()
            .await?;

        Ok(r)
    }

    pub async fn upload_file(
        &self,
        upload_info: &GetUploadUrl,
        path: PathBuf,
        file_name: String,
    ) -> anyhow::Result<()> {
        // https://www.backblaze.com/apidocs/b2-upload-file
        let mut f = File::open(&path)?;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;

        let mut hasher = Sha1::new();
        hasher.update(&buf);
        let file_hash = format!("{:X}", hasher.finalize());

        let mut headers = HeaderMap::new();
        let mut auth_value: HeaderValue = upload_info.authorization_token.parse().unwrap();
        auth_value.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, auth_value);
        headers.insert(reqwest::header::CONTENT_TYPE, "b2/x-auto".parse().unwrap());
        headers.insert(
            "X-Bz-File-Name",
            format!("{}/{}", self.prefix, file_name).parse().unwrap(),
        );
        headers.insert("X-Bz-Content-Sha1", file_hash.parse().unwrap());

        self.client
            .post(&upload_info.upload_url)
            .headers(headers)
            .body(buf)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(())
    }

    pub async fn list_files(&self) -> anyhow::Result<Vec<String>> {
        // https://www.backblaze.com/apidocs/b2-list-file-names
        let req = serde_json::json!({
            "bucketId": self.bucket_id,
            "prefix": self.prefix,
            "maxFileCount": 10_000,
        });
        let req = serde_json::to_string(&req)?;
        let r = self
            .client
            .post(format!("{}/b2_list_file_names", self.api_url))
            .body(req)
            .send()
            .await?
            .json::<ListFileNames>()
            .await?;

        Ok(r.files.into_iter().map(|f| f.file_name).collect())
    }

    pub async fn download_file_by_name(
        &self,
        file_name: &str,
        path: PathBuf,
    ) -> anyhow::Result<Duration> {
        // https://www.backblaze.com/apidocs/b2-download-file-by-name
        let url = format!(
            "{}/file/{}/{file_name}",
            self.versionless_api_url, self.bucket
        );
        let start = Instant::now();
        let r = self.client.get(url).send().await?;

        let mut f = File::create(path)?;
        let mut bytes = r.bytes().await?;
        f.write_all(&mut bytes)?;
        Ok(start.elapsed())
    }
}
