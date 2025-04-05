use std::{error::Error, fs, net::SocketAddr, sync::Arc};

use quinn::{rustls::{self, pki_types::CertificateDer}, ClientConfig, Endpoint};

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
