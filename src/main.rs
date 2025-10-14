mod audio;
mod math;
mod midi;
mod modules;
mod synthesizer;

use crate::audio::Audio;
use crate::midi::Midi;
use crate::synthesizer::Synthesizer;
use core_foundation::runloop::CFRunLoop;

fn main() {
    env_logger::init();
    log::info!("Starting Accidental Synthesizer");

    log::debug!("Initialize the audio module");
    let mut audio = Audio::new().expect("Could not initialize audio module. Exiting.");

    log::debug!("Initialize the synthesizer module");
    let mut synthesizer = Synthesizer::new(audio.get_sample_rate());

    log::debug!("Initialize the midi module");
    let mut midi = Midi::new();

    let output_device_receiver = audio.get_sample_buffer_receiver();
    let midi_message_receiver = midi.get_midi_message_receiver();

    log::debug!("Run the main modules");
    audio.run();
    midi.run()
        .expect("Could not initialize midi module. Exiting.");
    synthesizer.run(midi_message_receiver, output_device_receiver);

    // Temporary run loop to keep the application alive until I add the ui loop to replace it
    println!("Will Loop Forever. Press Ctrl-c to Exit");
    CFRunLoop::run_current();
}
