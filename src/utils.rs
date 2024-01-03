use std::error::Error;

use chrono::{ DateTime, FixedOffset, };
use reqwest::Proxy;
use serde::Deserialize;

use crate::models::JGProxy;

pub fn deserialize_datetime<'de, D>(
    deserializer: D
) -> Result<DateTime<FixedOffset>, D::Error>
where 
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let s = format!("{}{}", s, " +0800");
    
    let local_datetime = DateTime::
        parse_from_str(&s, "%Y-%m-%d %H:%M:%S %z")
        .unwrap();

    Ok(local_datetime)
}

pub fn convert_to_reqwest_proxy(jg_proxy: &JGProxy) -> Result<Proxy, Box<dyn Error>> {
    let proxy = format!("socks5://{}:{}", jg_proxy.ip, jg_proxy.port);
    let proxy = proxy.as_str();
    let proxy = Proxy::all(proxy)?;
    Ok(proxy)
}