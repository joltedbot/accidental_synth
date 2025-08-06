use crate::synthesizer::Synthesizer;

mod audio;
mod envelope;
mod midi;
mod synthesizer;

fn main() {
    env_logger::init();
    log::info!("Starting Accidental Synthesizer");

    log::debug!("Initialize the synthesizer module");
    let mut synthesizer = Synthesizer::new();

    log::debug!("Initialize the midi module");
    let mut midi = midi::Midi::new();

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
