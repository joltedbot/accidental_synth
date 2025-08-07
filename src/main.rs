mod audio;
mod midi;
mod modules;
mod synthesizer;

use crate::audio::Audio;
use crate::midi::Midi;
use crate::synthesizer::Synthesizer;

fn main() {
    env_logger::init();
    log::info!("Starting Accidental Synthesizer");

    log::debug!("Initialize the audio module");
    let mut audio = Audio::new().expect("Could not initialize audio module. Exiting.");

    log::debug!("Initialize the synthesizer module");
    let default_audio_output_device = audio.default_output_device();
    let sample_rate = audio.sample_rate();
    let mut synthesizer = Synthesizer::new(default_audio_output_device, sample_rate);

    log::debug!("Initialize the midi module");
    let mut midi = Midi::new();

    midi.run()
        .expect("Could not initialize midi module. Exiting.");

    synthesizer
        .run(midi.get_midi_message_receiver())
        .expect("Could not start synthesizer. Exiting.");

    // Temporary run loop to keep the application alive until I add the Slint ui loop to replace it
    println!("Will Loop Forever. Press Ctrl-c to Exit");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(100));
    }
}
