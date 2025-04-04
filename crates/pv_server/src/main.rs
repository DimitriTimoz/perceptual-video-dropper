use std::net::SocketAddr;

use pv_server::error::ServerError;
use pv_server::server::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ServerError> {
    env_logger::init();
    let addr = SocketAddr::from(([127, 0, 0, 1], 4242));
    let server = Server::new(addr).await?;

    server.run().await?;

    Ok(())
}
