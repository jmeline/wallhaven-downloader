use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperData {
    pub data: Data,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperSearchResults {
    pub data: Vec<Data>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Data {
    pub id: String,
    pub url: String,
    #[serde(rename = "short_url")]
    pub short_url: String,
    pub uploader: Uploader,
    pub views: i64,
    pub favorites: i64,
    pub source: String,
    pub purity: String,
    pub category: String,
    #[serde(rename = "dimension_x")]
    pub dimension_x: i64,
    #[serde(rename = "dimension_y")]
    pub dimension_y: i64,
    pub resolution: String,
    pub ratio: String,
    #[serde(rename = "file_size")]
    pub file_size: i64,
    #[serde(rename = "file_type")]
    pub file_type: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub colors: Vec<String>,
    pub path: String,
    pub thumbs: Thumbs,
    pub tags: Vec<Tag>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Uploader {
    pub username: String,
    pub group: String,
    pub avatar: Avatar,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Avatar {
    #[serde(rename = "200px")]
    pub n200px: String,
    #[serde(rename = "128px")]
    pub n128px: String,
    #[serde(rename = "32px")]
    pub n32px: String,
    #[serde(rename = "20px")]
    pub n20px: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbs {
    pub large: String,
    pub original: String,
    pub small: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub alias: String,
    #[serde(rename = "category_id")]
    pub category_id: i64,
    pub category: String,
    pub purity: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
}
