use std::{net::SocketAddr, sync::{Arc, Mutex}};
pub mod client;
use client::make_client_endpoint;
use log::{debug, error, info, warn};
use pv_core::network::{recv_packet, send_packet, Request, Response}; // Suppose que Request est sérializable
use minifb::{Window, WindowOptions};
use quinn::Connection;

// Structure pour partager la frame décodée
struct PlayerState {
    frame_buffer: Vec<u32>,
    width: usize,
    height: usize,
    new_frame: bool, 
}

impl PlayerState {
    fn new() -> Self {
        PlayerState {
            frame_buffer: vec![128; 640 * 480],
            width: 640,
            height: 480,
            new_frame: false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let endpoint = make_client_endpoint("0.0.0.0:0".parse().unwrap(), "../pv_server/pub_key.pem")?;

    let server_addr = SocketAddr::from(([127, 0, 0, 1], 4242));
    let connection = endpoint
        .connect(server_addr, "localhost")? 
        .await?;
    info!("connected: addr={}", connection.remote_address());

    // Test ping
    let (mut send, mut recv) = connection.open_bi().await?;
    send_packet(&mut send, Request::Ping(12313897890)).await?;
    info!("sent ping request");
    let response = recv_packet::<Response>(&mut recv).await?;
    assert!(matches!(response, Response::Pong(_)));
    info!("received pong response: {:?}", response);
    drop(send);
    drop(recv);


    // Request video stream
    info!("Requesting video stream...");
    let (mut send_req, mut recv_resp) = connection.open_bi().await?;
    send_packet(&mut send_req, Request::VideoStream(0)).await?; 
    info!("sent video stream request");

    // Attendre la confirmation (ou des métadonnées vidéo ?)
    // Le serveur DOIT envoyer une réponse pour que le client sache que le stream va démarrer
    let mut window: Option<Window> = None;

    let response = recv_packet::<Response>(&mut recv_resp).await?;
    let (initial_width, initial_height, buffer) = if let Response::Frame { data, width, height} = &response {
         info!("received video stream confirmation w={}, h={} size={}", width, height, data.len());
         (*width as usize, *height as usize, data.clone())
    } else {
        error!("Unexpected response to VideoStream request: {:?}", response);
        return Err("Bad response from server".into());
    };

    window = Some(
        Window::new(
            "Video Player - Press ESC to exit",
            initial_width,
            initial_height,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| panic!("{}", e)),
    );
    if let Some(w) = &mut window {
        if let Err(e) = w.update_with_buffer(&buffer, initial_width, initial_height) {
            error!("Failed to update window: {}", e);
        }
    } else {
        error!("Failed to create window");
        return Err("Failed to create window".into());
        
    }

    let player_state = Arc::new(Mutex::new(PlayerState::new()));

    let player_state_clone = Arc::clone(&player_state);

    tokio::spawn(async move {
        info!("Network/Decode task started.");
        if let Err(e) = run_network_decode_loop(recv_resp, player_state_clone).await {
             error!("Network/Decode task failed: {}", e);
        }
        info!("Network/Decode task finished.");
    });


    info!("Starting display loop...");
    loop {
        // Initialiser/MàJ la fenêtre si la taille est connue
        if let Some(window) =  &mut window {
            let state = player_state.lock().unwrap();
            window.update_with_buffer(&state.frame_buffer, state.width, state.height).unwrap_or_else(|e| {
                error!("Failed to update window: {}", e);
            });
            if window.is_open() && window.is_key_down(minifb::Key::Escape) {
                info!("Window closed or ESC pressed, exiting...");
                break;
            } 
        } else {
            let state = player_state.lock().unwrap();
            if state.width > 0 && state.height > 0 {
                info!("Initializing window {}x{}", state.width, state.height);
                window = Some(
                    Window::new(
                        "Video Player - Press ESC to exit",
                        state.width,
                        state.height,
                        WindowOptions::default(),
                    )
                    .unwrap_or_else(|e| panic!("{}", e)),
                );
                // Limiter le rafraîchissement pour ne pas surcharger le CPU
                window.as_mut().unwrap().set_target_fps(60);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;

    }

    info!("Closing QUIC connection.");
    connection.close(0u32.into(), b"done");
    endpoint.wait_idle().await;

    info!("Client shutdown complete.");
    Ok(())
}


pub async fn run_network_decode_loop(
    mut recv_stream: quinn::RecvStream,
    player_state: Arc<Mutex<PlayerState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        info!("Waiting for packets...");
        match recv_packet::<Response>(&mut recv_stream).await {
            Ok(response) => {
                match response {
                    Response::Frame { data, width, height } => {
                        let mut state = player_state.lock().unwrap();
                        state.frame_buffer = data;
                        state.width = width as usize;
                        state.height = height as usize;
                        state.new_frame = true;
                        info!("Received frame: {}x{} ", width, height);
                    }
                    _ => {
                        warn!("Unexpected response: {:?}", response);
                    }
                }
            }
            Err(e) => {
                error!("Error receiving packet: {}", e);
                break;
            }
        }
    }
    Ok(())
}