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
use core_foundation::runloop::CFRunLoop;

slint::include_modules!();

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(long)]
    pub headless: bool,
}

fn main() {
    let application = AccidentalSynth::new().expect("Could not initialize the UI framework.");
    let cli_arguments = Arguments::parse();

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

    log::debug!("Run the main modules");
    audio.run(ui_update_sender.clone());
    midi.run(ui_update_sender)
        .expect("Could not initialize midi module. Exiting.");
    synthesizer
        .run(midi_message_receiver, audio_sample_buffer_receiver)
        .expect("Could not initialize synthesizer module. Exiting.");
    ui.run(
        &application.as_weak(),
        midi_setting_update_sender,
        audio_output_device_sender,
    )
    .expect("Could build the user interface. Exiting.");

    // Temporary run loop to keep the application alive until I add the ui loop to replace it
    println!("Will Loop Forever. Press Ctrl-c to Exit");

    if cli_arguments.headless {
        CFRunLoop::run_current();
    } else {
        application
            .run()
            .expect("Could not create the user interface.");
    }
}
