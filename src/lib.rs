pub mod errors;
pub mod models;
pub mod utils;
pub mod tests;

use std::{error::Error, sync::Arc};
use errors::CurrentIPNotInWhitelistError;
use models::APIResponse;
use rand::seq::SliceRandom;
use reqwest::Proxy;
use sqlx::types::chrono::{ Utc, FixedOffset, };
use tokio::sync::Mutex;

use crate::models::JGProxy;


#[derive(Debug)]
pub struct JGProxiesPool {
    proxies: Arc<Mutex<Vec<JGProxy>>>,
    size: usize,
    min_proxies: usize,
    expire_threshold: usize, // seconds of expiration
}

impl JGProxiesPool {
    pub async fn new(
        size: usize, 
        min_proxies: usize, 
        expire_threshold: usize
    ) -> Result<Self, Box<dyn Error>>{
        let mut proxies_pool = JGProxiesPool {
            proxies: Arc::new(Mutex::new(vec![])),
            size,
            min_proxies,
            expire_threshold,
        };

        proxies_pool.add_new_proxies().await?;

        Ok(proxies_pool)
    }

    pub async fn new_default() -> Result<Self, Box<dyn Error>> {
        let proxies_pool = Self::new(100, 30, 30).await?;
        Ok(proxies_pool)
    }

    pub async fn get_proxy(&mut self) -> Result<Proxy, Box<dyn Error>> {
        let proxies = self.proxies.clone();
        let mut proxies = proxies.lock().await;
        
        let current_time = Utc::now()
            .with_timezone(&FixedOffset::east_opt(8*3600).unwrap());

        proxies.retain(|proxy| {
            let time_span = proxy.expire_time - current_time;
            time_span.num_seconds() > self.expire_threshold as i64
        });

        if proxies.len() < self.min_proxies {
            self.add_new_proxies().await?;
        }

        let jg_proxy = proxies
            .choose(&mut rand::thread_rng())
            .unwrap();

        let proxy = utils::convert_to_reqwest_proxy(&jg_proxy);
        proxy
    }

    async fn add_new_proxies(&mut self) -> Result<(), Box<dyn Error>> {
        let proxies = self.proxies.clone();
        let mut proxies = proxies.lock().await;

        let api = format!(
            "http://sd.jghttp.alicloudecs.com/get_ip?num={}&type=2&pro=&city=0&yys=0&port=2&time=8&ts=1&ys=1&cs=1&lb=1&sb=0&pb=4&mr=1&regions=",
            &self.size
        );

        let response = reqwest::get(api)
            .await?
            .text()
            .await?;

        let api_response: APIResponse = serde_json::from_str(&response)?;

        match api_response.code {
            0 => {
                let new_proxies = api_response.proxies;
                proxies.extend(new_proxies);
            },
            113 => return Err(Box::new(CurrentIPNotInWhitelistError)),
            _ => panic!("Unknown response from {} JG Proxies: {:#?}", api_response.code, api_response) 
        }

        println!("New proxies from sd.jghttp.alicloudecs.com added!");

        Ok(())
    }
}

#[cfg(test)]
mod test{
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use futures;

    use super::*;

    #[tokio::test]
    async fn test_get_proxy_sigle_thread() {
        let mut proxies_pool = JGProxiesPool::new(20, 5, 5).await.unwrap();
        for i in 0..100 {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            let proxy = proxies_pool.get_proxy().await.unwrap();
            println!("{i}, {:#?}", proxy);
        }
    }

    #[tokio::test]
    async fn test_get_proxy_multiple_threads() {
        let test_threads = 1000;
        let rep = 1000;

        let proxies_pool = Arc::new(Mutex::new(
            JGProxiesPool::new_default().await.unwrap()
        ));

        let mut tasks = vec![];

        for _ in 0..test_threads {
            let task = async {
                let proxies_pool_cloned = proxies_pool.clone();
                for _ in 0..rep {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    let mut proxies_pool_cloned = proxies_pool_cloned.lock().await;
                    let _proxy = proxies_pool_cloned.get_proxy().await.unwrap();
                }
            };
            tasks.push(task);
        }

        let _res = futures::future::join_all(tasks).await;
    }

    #[tokio::test]
    async fn test_add_new_proxies() {
        let mut proxies_pool = JGProxiesPool::new_default().await.unwrap();
        let res = proxies_pool.add_new_proxies().await;
        if let Ok(_) = res {
            assert_eq!(true, true);
        }
    }
}

