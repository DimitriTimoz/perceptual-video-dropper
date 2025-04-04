use quinn::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::{error::Error, io::Write};

use crate::{config::VideoServerConfig, network::handle_connection, prelude::*};
use quinn::{ClientConfig, Endpoint};
use std::fs;

fn configure_client(cert_path: &str) -> Result<ClientConfig, Box<dyn Error + Send + Sync + 'static>> {
    // Load the server certificate from the file
    let cert_data = fs::read(cert_path)?;
    let server_certs = vec![CertificateDer::from(cert_data)];

    let mut certs = rustls::RootCertStore::empty();
    for cert in server_certs {
        certs.add(cert)?;
    }

    Ok(ClientConfig::with_root_certificates(Arc::new(certs))?)
}

pub fn make_client_endpoint(
    bind_addr: SocketAddr,
    cert_path: &str, 
) -> Result<Endpoint, Box<dyn Error + Send + Sync + 'static>> {
    let client_config = configure_client(cert_path)?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

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
