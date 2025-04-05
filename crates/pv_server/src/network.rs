use quinn::Incoming;
use crate::prelude::*;
use pv_core::network::{recv_packet, send_packet, Request, Response};

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
            // Spawn a task to send the video stream
            tokio::spawn(async move {
                for i in 0..10000 {
                    let response = Response::Frame{
                        data: vec![i as u32; 640 * 480],
                        width: 640,
                        height: 480,
                    };
                    if let Err(e) = send_packet(&mut send, response).await {
                        error!("Failed to send video frame: {:?}", e);
                        break;
                    } else {
                        info!("Sent video frame: {:?}", i);
                    }
                }
            });
        },
        Request::Ping(ping_id) => {
            info!("Received ping request: {:?}", ping_id);
            let response = Response::Pong(ping_id);
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
}
