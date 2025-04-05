
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

struct FrameDropperLogic {
    frame_count: usize,
    drop_interval: usize,
}

impl FrameDropperLogic {
    fn new(drop_interval: usize) -> Self {
        Self { frame_count: 0, drop_interval }
    }

    fn should_drop(&mut self) -> bool {
        self.frame_count += 1;
        if self.drop_interval == 0 {
            return false;
        }
        // Drop frames based on the drop interval
        (self.frame_count % self.drop_interval != 0)
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
