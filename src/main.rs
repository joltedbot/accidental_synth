mod audio;
mod math;
mod midi;
mod modules;
mod synthesizer;
mod ui;

use crate::audio::Audio;
use crate::midi::Midi;
use crate::synthesizer::Synthesizer;
use crate::ui::UI;
use clap::Parser;

slint::include_modules!();

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arguments {}

fn main() {
    let application = AccidentalSynth::new().expect("Could not initialize the UI framework.");
    let _ = Arguments::parse();

    env_logger::init();
    log::info!("Starting Accidental Synthesizer");

    let mut ui = UI::new();

    log::debug!("Initialize the audio module");
    let mut audio = Audio::new().expect("Could not initialize audio module. Exiting.");

    log::debug!("Initialize the synthesizer module");
    let mut synthesizer = Synthesizer::new(audio.get_sample_rate());

    log::debug!("Initialize the midi module");
    let mut midi = Midi::new();

    let audio_sample_buffer_receiver = audio.get_sample_buffer_receiver();
    let audio_output_device_sender = audio.get_device_update_sender();
    let midi_message_receiver = midi.get_midi_message_receiver();
    let midi_setting_update_sender = midi.get_device_update_sender();
    let ui_update_sender = ui.get_ui_update_sender();
    let synthesizer_update_sender = synthesizer.get_ui_update_sender();

    log::debug!("Run the main modules");
    audio
        .run(ui_update_sender.clone())
        .expect("Could not initialize audio module. Exiting.");
    midi.run(ui_update_sender.clone())
        .expect("Could not initialize midi module. Exiting.");
    synthesizer
        .run(
            midi_message_receiver,
            audio_sample_buffer_receiver,
            ui_update_sender,
        )
        .expect("Could not initialize the synthesizer module. Exiting.");
    ui.run(
        &application.as_weak(),
        midi_setting_update_sender,
        &audio_output_device_sender,
        &synthesizer_update_sender,
    )
    .expect("Could build the user interface. Exiting.");

    println!("Will Loop Forever. Press Ctrl-c to Exit");
    application
        .run()
        .expect("Could not create the user interface.");
}
