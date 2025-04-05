use quinn::{rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer}, ServerConfig};
use std::{error::Error, io::Write};

use crate::{config::VideoServerConfig, network::handle_connection, prelude::*};

fn configure_server()
-> Result<(ServerConfig, CertificateDer<'static>), Box<dyn Error + Send + Sync + 'static>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

    let mut server_config =
        ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into())?;
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    Ok((server_config, cert_der))
}

fn make_server_endpoint(
    bind_addr: SocketAddr,
) -> Result<(Endpoint, CertificateDer<'static>), Box<dyn Error + Send + Sync + 'static>> {
    let (server_config, server_cert) = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}

pub struct Server {
    endpoint: Endpoint,
    config: VideoServerConfig,
}

impl Server {
    pub async fn new(
        config: VideoServerConfig,
    ) -> Result<Self, ServerError> {
        let (endpoint, server_cert) = make_server_endpoint(config.listen_address()?)?;
        // Save the server certificate to a file
        let cert_path = "pub_key.pem";
        let mut file = std::fs::File::create(cert_path)?;
        file.write_all(&server_cert)?;
        info!("Server certificate saved to {}", cert_path);
        Ok(Server {
            endpoint,
            config,
        })
    }

    pub async fn run(&self) -> Result<(), ServerError> {
        let bind_addr = self.endpoint.local_addr()?;
        info!("Server running on {}", bind_addr);
        while let Some(conn) = self.endpoint.accept().await {
            info!("New connection: {:?}", conn.remote_address());
            let fut = handle_connection(conn);
            tokio::spawn(async move {
                if let Err(e) = fut.await {
                    error!("Error handling connection: {:?}", e);
                }
            });
        }
        Ok(())
    }
}
