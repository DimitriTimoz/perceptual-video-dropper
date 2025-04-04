use std::net::SocketAddr;

use pv_server::error::ServerError;
use pv_server::server::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ServerError> {
    env_logger::init();
    let addr = SocketAddr::from(([127, 0, 0, 1], 4242));
    let config = pv_server::config::VideoServerConfig::new(
        addr.to_string(),
        4242,
        "./assets/rick.mp4".to_string(),
    );
    let server = Server::new(config).await?;

    server.run().await?;

    Ok(())
}
