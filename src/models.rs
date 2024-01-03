use sqlx::types::chrono::{ DateTime, FixedOffset };
use serde::Deserialize;
use crate::utils::deserialize_datetime;

#[derive(Debug, Deserialize, Clone)]
pub struct JGProxy {
    pub city: String,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub expire_time: DateTime<FixedOffset>,
    pub ip: String,
    pub isp: Option<String>,
    pub port: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct APIResponse {
    pub code: u32,
    #[serde(rename = "data")]
    pub proxies: Vec<JGProxy>,
    pub msg: String,
    pub success: bool,
}