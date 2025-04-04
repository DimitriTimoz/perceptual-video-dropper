use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub listen_address: String, 
    pub listen_port: u16,
    pub cert_path: String, 
    pub key_path: String, 
}
