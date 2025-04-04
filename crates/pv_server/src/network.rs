use quinn::Incoming;
use crate::prelude::*;


pub async fn handle_connection(
    connection: Incoming,
) -> Result<(), ServerError> { 
    let connection = connection.await?;
    info!("New connection: {:?}", connection.remote_address());
    loop {
        let stream = connection.accept_bi().await;
        let stream = match stream {
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                info!("connection closed");
                return Ok(());
            }
            Err(e) => {
                return Err(ServerError::ConnectionError(e));
            }
            Ok(s) => s,
        };
        // Print the stream ID
        let stream_id = stream.1.id();
        info!("Stream ID: {:?}", stream_id);
    }

    Ok(())
}
