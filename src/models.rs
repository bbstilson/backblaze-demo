use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct AuthorizeAccount {
    pub authorization_token: String,
    pub application_key_expiration_timestamp: Option<u64>,
    pub api_info: ApiInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct ApiInfo {
    pub storage_api: StorageApi,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct StorageApi {
    pub bucket_id: String,
    pub bucket_name: String,
    pub api_url: String,
    pub s3_api_url: String,
    pub download_url: String,
    pub name_prefix: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct GetUploadUrl {
    pub authorization_token: String,
    pub upload_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct ListFileNames {
    pub files: Vec<B2File>,
    pub next_file_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct B2File {
    pub file_name: String,
}
