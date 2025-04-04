
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
        if self.drop_interval == 0 {
            return false;
        }
        // Drop frames based on the drop interval
        (self.frame_count % self.drop_interval != 0)
    }

}

#[derive(Debug, Default)]
struct FrameBufferPair {
    previous: Option<gst::Buffer>,
    current: Option<gst::Buffer>,
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
        "filesrc location=\"{}\" ! qtdemux name=demux \
         demux.video_0 ! queue ! h264parse config-interval=1 ! identity name=frame_filter ! rtph264pay pt=96 name=pay0 \
         demux.audio_0 ! queue leaky=no  ! aacparse ! rtpmp4gpay pt=97 name=pay1",
        video_path
    );
        
    // Set the pipeline description
    factory.set_launch(&pipeline_desc);
    factory.set_transport_mode(gstreamer_rtsp_server::RTSPTransportMode::PLAY);
    factory.set_shared(true);
    factory.set_latency(0);

    let dropper_logic = Arc::new(Mutex::new(FrameDropperLogic::new(0)));
    factory.connect_media_configure(move |_, media| {
        let bin = media.element().downcast::<gst::Bin>().unwrap();
        media.set_clock(Some(&gst::SystemClock::obtain()));

        let identity_elem = match bin.by_name("frame_filter") {
            Some(elem) => elem,
            None => {
                eprintln!("Failed to find element 'frame_filter' in the pipeline");
                return;
            }
        };

        let sink_pad = match identity_elem.static_pad("sink") {
             Some(pad) => pad,
             None => {
                 eprintln!("Failed to get sink pad from 'frame_filter'");
                 return;
             }
        };

        let frame_buffer_pair = Arc::new(Mutex::new(FrameBufferPair::default()));
        let logic_clone = Arc::clone(&dropper_logic);
        let buffer_pair_clone = Arc::clone(&frame_buffer_pair);

        sink_pad.add_probe(gst::PadProbeType::BUFFER, move |_, probe_info| {
            if let Some(buffer) = probe_info.buffer() {
                let mut buffer_pair = buffer_pair_clone.lock().unwrap();
        
                // Met à jour les deux dernières frames
                buffer_pair.previous = buffer_pair.current.take();
                buffer_pair.current = Some(buffer.copy());
        
                // ➕ Exemple d'accès aux frames n et n-1
                if let (Some(ref prev), Some(ref curr)) = (&buffer_pair.previous, &buffer_pair.current) {
                    let pts_prev = prev.pts().map(|p| p.mseconds()).unwrap_or(0);
                    let pts_curr = curr.pts().map(|p| p.mseconds()).unwrap_or(0);
                    println!("Frame n-1 PTS: {}, Frame n PTS: {}", pts_prev, pts_curr);
        
                    // Tu peux aussi accéder aux données : curr.map_readable().unwrap().as_slice()
                }
        
                if !buffer.flags().contains(gst::BufferFlags::DELTA_UNIT) {
                    // C’est une keyframe → on garde
                    return gst::PadProbeReturn::Ok;
                } else {
                    // C’est une P ou B frame → on droppe
                    return gst::PadProbeReturn::Drop;
                }
            }
        
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
