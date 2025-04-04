use std::net::SocketAddr;

use pv_core::network::Request;
use pv_server::server::make_client_endpoint;

#[tokio::test(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let endpoint=  make_client_endpoint("0.0.0.0:0".parse().unwrap(), "./pub_key.pem")?;

    let server_addr = SocketAddr::from(([127, 0, 0, 1], 4242));
    let connection = endpoint
        .connect(server_addr, "localhost")
        .unwrap()
        .await
        .unwrap();

    println!("[client] connected: addr={}", connection.remote_address());
    // Ping/pong
    let (mut send, mut recv) = connection.open_bi().await?;
    pv_server::network::send_packet(&mut send, Request::Ping(12313897890)).await?;
    println!("[client] sent request");
    let response = pv_server::network::recv_packet::<Request>(&mut recv).await?;
    assert!(matches!(response, Request::Ping(_)));
    println!("[client] received response: {:?}", response);

    // Waiting for a stream will complete with an error when the server closes the connection
    let (mut send, mut recv) = connection.open_bi().await?;
    pv_server::network::send_packet(&mut send, Request::VideoStream(0)).await?;
    println!("[client] sent request");
    let response = pv_server::network::recv_packet::<Request>(&mut recv).await?;
    assert!(matches!(response, Request::VideoStream { .. }));
    println!("[client] received response: {:?}", response);
    endpoint.close(0u32.into(), b"done");
    Ok(())
}
