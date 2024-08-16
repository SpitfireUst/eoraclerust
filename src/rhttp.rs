use reqwest::{Client as ReqwestClient, Error as ReqwestError, RequestBuilder, Response};
use std::time::Duration;
use tokio::sync::Mutex;

pub struct Client {
    inner: ReqwestClient,
    max_retries: u32,
    rate_limiter: Option<Mutex<tokio::time::Interval>>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            inner: ReqwestClient::new(),
            max_retries: 0,
            rate_limiter: None,
        }
    }

    pub fn with_max_requests_per_second(mut self, limit: f64) -> Self {
        if limit > 0.0 {
            let duration = Duration::from_secs_f64(1.0 / limit);
            self.rate_limiter = Some(Mutex::new(tokio::time::interval(duration)));
        }
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn request(&self, method: reqwest::Method, url: reqwest::Url) -> RequestBuilder {
        self.inner.request(method, url)
    }

    pub async fn send(&self, req: RequestBuilder) -> Result<Response, ReqwestError> {
        let mut attempts = 0;
        loop {
            if let Some(limiter) = &self.rate_limiter {
                let mut interval = limiter.lock().await;
                interval.tick().await;
            }

            match req
                .try_clone()
                .expect("Request must be clonable")
                .send()
                .await
            {
                Ok(response) => return Ok(response),
                Err(_) if attempts < self.max_retries => {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_secs(2u64.pow(attempts))).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
