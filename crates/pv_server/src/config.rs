use std::{net::SocketAddr, str::FromStr};

use serde::Deserialize;

use crate::error::ServerError;

#[derive(Deserialize, Debug)]
pub struct VideoServerConfig {
    pub listen_address: String, 
    pub listen_port: u16,
    pub cert_path: String, 
    pub key_path: String, 
    pub video_path: String,
}

impl VideoServerConfig {
    pub fn new(
        listen_address: String,
        listen_port: u16,
        cert_path: String,
        key_path: String,
        video_path: String,
    ) -> Self {
        Self {
            listen_address,
            listen_port,
            cert_path,
            key_path,
            video_path,
        }
    }

    pub fn listen_address(&self) -> Result<SocketAddr, ServerError> {
        let addr = SocketAddr::from_str(&self.listen_address)?; 
        Ok(addr)
    }

    pub fn listen_port(&self) -> u16 {
        self.listen_port
    }

    pub fn cert_path(&self) -> &str {
        &self.cert_path
    }

    pub fn key_path(&self) -> &str {
        &self.key_path
    }

    pub fn video_path(&self) -> &str {
        &self.video_path
    }
}
