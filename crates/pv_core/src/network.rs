use bincode::{de, enc, Decode, Encode};
use quinn::SendStream;

use crate::error::CoreError;

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
    Pong(u64),
    /// Response Frame
    Frame {
        /// Frame data
        data: Vec<u32>,
        width: u32,
        height: u32,
    }
}

pub async fn send_packet<E: enc::Encode>(
    stream: &mut SendStream,
    packet: E,
) -> Result<(), CoreError> {
    let encoded = bincode::encode_to_vec(packet, bincode::config::standard())?;
    stream.write_all(&(encoded.len() as u32).to_be_bytes()).await?;
    stream.write_all(&encoded).await?;
    Ok(())
}

pub async fn recv_packet<D: de::Decode<()>>(
    stream: &mut quinn::RecvStream,
) -> Result<D, CoreError> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    let (packet, _) = bincode::decode_from_slice::<D, _>(&buf, bincode::config::standard())?;
    Ok(packet)
}
