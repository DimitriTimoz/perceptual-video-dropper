use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum Request {
    /// Request a ping
    Ping(u64),
    /// Request to play a video stream at the given time offset (ms)
    VideoStream(u64),
}

#[derive(Encode, Decode, Debug)]
pub enum Response {
    /// Response to a ping request
    Ping(u64),
    /// Response Frame
    Frame {
        /// Frame data
        data: Vec<u8>,
    }
}
