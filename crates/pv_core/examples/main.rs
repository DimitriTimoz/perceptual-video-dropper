use pv_core::video::pipeline;


#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();
    let url = concat!(
        "( ",
        "filesrc location=/Users/dimitri/Documents/Projects/perceptual_video_dropper/crates/pv_server/assets/rick.mp4 ! decodebin name=d ",
        "d. ! x264enc ! rtph264pay name=pay0 pt=96 ",
        ")"
    );
    let _ = pipeline("/Users/dimitri/Documents/Projects/perceptual_video_dropper/crates/pv_server/assets/rick.mp4").await;
}
