use crate::audio::Audio;
use crate::synthesizer::Synthesizer;

mod audio;
mod synthesizer;

fn main() {
    env_logger::init();
    log::info!("Starting Accidental Synthesizer");

    log::debug!("Initialize the Synthesizer Module");
    let mut synthesizer = Synthesizer::new();
    synthesizer
        .run()
        .expect("Could not start synthesizer. Exiting.");

    // Temporary run loop to keep the application alive until I add the Slint ui loop to replace it
    println!("Will Loop Forever. Press Ctrl-c to Exit");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(100));
    }
}
