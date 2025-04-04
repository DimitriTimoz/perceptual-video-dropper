
use std::sync::{Arc, Mutex};

use derive_more::{Display, Error};
use log::info;
use serde::{Deserialize, Serialize};
use gstreamer::{self as gst, glib::{object::Cast, MainLoop}};
use gstreamer_rtsp_server::{prelude::*, RTSPServer};

#[derive(Serialize, Deserialize)]
pub struct VideoFrame {
    timestamp: u64,
    data: Vec<u8>,
}

impl VideoFrame {
    pub fn new(timestamp: u64, data: Vec<u8>) -> Self {
        Self { timestamp, data }
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

struct FrameDropperLogic {
    frame_count: usize,
    drop_interval: usize,
}

impl FrameDropperLogic {
    fn new(drop_interval: usize) -> Self {
        Self { frame_count: 0, drop_interval }
    }

    fn should_drop(&mut self) -> bool {
        self.frame_count += 1;
        !(self.frame_count % self.drop_interval == 0)
    }

}


#[derive(Debug, Display, Error)]
#[display("Could not get mount points")]
struct NoMountPoints;

pub async fn pipeline(video_path: &str) -> Result<(), gst::ErrorMessage> {
    gst::init().unwrap();
    let main_loop = MainLoop::new(None, false);
    let server = RTSPServer::new();

    let mounts = server.mount_points().unwrap();

    let factory = gstreamer_rtsp_server::RTSPMediaFactory::new();
    
    // Create a complete pipeline description
    let pipeline_desc = format!(
        "( filesrc location=\"{}\" ! decodebin name=decode ! \
         queue ! videoconvert ! identity name=frame_filter ! queue ! x264enc tune=zerolatency ! rtph264pay name=pay0 pt=96 \
         decode. ! queue ! audioconvert ! avenc_aac ! rtpmp4gpay name=pay1 pt=97 )",
        video_path
    );
    
    // Set the pipeline description
    factory.set_launch(&pipeline_desc);
    factory.set_shared(true);

    let dropper_logic = Arc::new(Mutex::new(FrameDropperLogic::new(4)));
    factory.connect_media_configure(move |_, media| {
        let bin = media.element().downcast::<gst::Bin>().unwrap();

        let identity_elem = match bin.by_name("frame_filter") {
            Some(elem) => elem,
            None => {
                eprintln!("Failed to find element 'frame_filter' in the pipeline");
                return;
            }
        };

        // Obtenir le pad 'sink' (entrée) de l'élément identity
        let sink_pad = match identity_elem.static_pad("sink") {
             Some(pad) => pad,
             None => {
                 eprintln!("Failed to get sink pad from 'frame_filter'");
                 return;
             }
        };

        // Cloner l'Arc pour le déplacer dans le callback de la sonde
        let logic_clone = Arc::clone(&dropper_logic);

        // Ajouter la sonde au pad 'sink'
        sink_pad.add_probe(gst::PadProbeType::BUFFER, move |_, probe_info| {
            if let Some(buffer) = probe_info.buffer() {
                // Obtenir le timestamp (optionnel, mais utile pour une logique plus complexe)
                let _timestamp = buffer.pts().map(|pts| pts.nseconds());

                let mut logic = logic_clone.lock().unwrap();
                if logic.should_drop() {
                    // Si on doit dropper, retourner Drop
                    return gst::PadProbeReturn::Drop;
                }
            }
            // Sinon (ou si ce n'est pas un buffer), laisser passer
            gst::PadProbeReturn::Ok
        });

        info!("Pad probe added to 'frame_filter' sink pad.");
    });


    mounts.add_factory("/test", factory);

    let id = server.attach(None).unwrap();

    info!(
        "Stream ready at rtsp://127.0.0.1:{}/test",
        server.bound_port()
    );

    main_loop.run();
    id.remove();

    Ok(())
}

impl std::fmt::Debug for VideoFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoFrame")
            .field("timestamp", &self.timestamp)
            .field("data_length", &self.data.len())
            .finish()
    }
}
