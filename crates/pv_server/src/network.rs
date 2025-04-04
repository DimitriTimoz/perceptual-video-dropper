use bincode::{de, enc};
use quinn::{Incoming, SendStream};
use crate::prelude::*;
use pv_core::network::{Request, Response};

pub async fn send_packet<E: enc::Encode>(
    stream: &mut SendStream,
    packet: E,
) -> Result<(), ServerError> {
    let encoded = bincode::encode_to_vec(packet, bincode::config::standard())?;
    stream.write_all(&(encoded.len() as u32).to_be_bytes()).await?;
    stream.write_all(&encoded).await?;
    Ok(())
}

pub async fn recv_packet<D: de::Decode<()>>(
    stream: &mut quinn::RecvStream,
) -> Result<D, ServerError> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    let (packet, _) = bincode::decode_from_slice::<D, _>(&buf, bincode::config::standard())?;
    Ok(packet)
}

async fn handle_request(
    send: quinn::SendStream,
    recv: quinn::RecvStream,
) -> Result<(), ServerError> {
    let mut send = send;
    let mut recv = recv;
    info!("Handling request");
    let request = recv_packet::<Request>(&mut recv).await?;
    info!("Received request: {:?}", request);

    match request {
        Request::VideoStream(stream_id) => {
            info!("Received video stream request: {:?}", stream_id);
            let response = Response::Frame{data:vec![]};
            send_packet(&mut send, response).await?;        
        },
        Request::Ping(ping_id) => {
            info!("Received ping request: {:?}", ping_id);
            let response = Response::Ping(ping_id);
            send_packet(&mut send, response).await?;
        }
        _ => {
            unimplemented!("Request: {:?} unimplemented", request);
        }
    }
    Ok(())
}

pub async fn handle_connection(
    connection: Incoming,
) -> Result<(), ServerError> { 
    let connection = connection.await?;
    info!("New connection: {:?}", connection.remote_address());
    loop {
        let stream = connection.accept_bi().await;
        let (send, recv) = match stream {
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                info!("connection closed");
                return Ok(());
            }
            Err(e) => {
                return Err(ServerError::ConnectionError(e));
            }
            Ok(s) => s,
        };
        info!("Accepted stream: {:?}", send.id());
        let fut = handle_request(send, recv);
        tokio::spawn(
            async move {
                if let Err(e) = fut.await {
                    error!("failed: {reason}", reason = e);
                }
            }
        );
    }

    Ok(())
}
