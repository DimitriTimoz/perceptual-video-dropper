use std::net::SocketAddr;

use pv_server::server::make_client_endpoint;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let endpoint=  make_client_endpoint("0.0.0.0:0".parse().unwrap(), "pub_key.pem")?;

    let server_addr = SocketAddr::from(([127, 0, 0, 1], 4242));
    let connection = endpoint
        .connect(server_addr, "localhost")
        .unwrap()
        .await
        .unwrap();

    println!("[client] connected: addr={}", connection.remote_address());

    
    // Waiting for a stream will complete with an error when the server closes the connection
    let _ = connection.accept_uni().await;
    
    Ok(())
}
