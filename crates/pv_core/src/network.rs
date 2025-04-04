use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum Response {
    /// Response Frame
    Frame {
        /// Frame data
        data: Vec<u8>,
    }
}

#[derive(Encode, Decode, Debug)]
pub enum Request {
    /// Request to play a video stream at the given time offset (ms)
    VideoStream(u64),
}
