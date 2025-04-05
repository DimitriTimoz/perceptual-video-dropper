
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VideoFrame {
    timestamp: u64,
    data: Vec<u8>,
}

impl VideoFrame {
    pub fn new(timestamp: u64, data: Vec<u8>) -> Self {
        Self { timestamp, data }
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl std::fmt::Debug for VideoFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoFrame")
            .field("timestamp", &self.timestamp)
            .field("data_length", &self.data.len())
            .finish()
    }
}
