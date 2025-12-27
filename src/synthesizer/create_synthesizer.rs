use crate::audio::OutputStreamParameters;
use crate::math::load_f32_from_atomic_u32;
use crate::modules::amplifier::amplify_stereo;
use crate::modules::envelope::Envelope;
use crate::modules::filter::Filter;
use crate::modules::lfo::Lfo;
use crate::modules::mixer::{MixerInput, output_mix, quad_mix};
use crate::modules::oscillator::{HardSyncRole, Oscillator, WaveShape};
use crate::synthesizer;
use crate::synthesizer::constants::SAMPLE_PRODUCER_LOOP_SLEEP_DURATION_MICROSECONDS;
use crate::synthesizer::{CurrentNote, EnvelopeIndex, ModuleParameters, OscillatorIndex};
use anyhow::Result;
use crossbeam_channel::Receiver;
use log::info;
use rtrb::Producer;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

struct Modules {
    amp_envelope: Envelope,
    filter_envelope: Envelope,
    filter_lfo: Lfo,
    filter: Filter,
    mod_wheel_lfo: Lfo,
    oscillators: [Oscillator; 4],
}

pub fn create_synthesizer(
    sample_buffer_receiver: Receiver<Producer<f32>>,
    output_stream_parameters: OutputStreamParameters,
    current_note: &Arc<CurrentNote>,
    module_parameters: &Arc<ModuleParameters>,
) -> Result<()> {
    let current_note = current_note.clone();
    let module_parameters = module_parameters.clone();

    log::debug!("Blocking till we receive a sample buffer producer from the audio module");
    let mut sample_buffer = sample_buffer_receiver.recv()?;
    log::debug!("Sample buffer producer received from the audio module");

    thread::spawn(move || {
        let sample_rate = output_stream_parameters.sample_rate.load(Relaxed);
        info!("Creating the synthesizer audio loop at sample rate: {sample_rate}");

        let mut previous_sample_rate = 0;
        let mut previous_buffer_size = 0;

        let mut modules = initialize_synth_modules(sample_rate);

        let mut current_buffer_size = output_stream_parameters.buffer_size.load(Relaxed);
        let mut stereo_buffer_size = current_buffer_size as usize * 2;
        let mut local_buffer = Vec::<f32>::with_capacity(stereo_buffer_size);

        loop {
            current_buffer_size = output_stream_parameters.buffer_size.load(Relaxed);
            stereo_buffer_size = current_buffer_size as usize * 2;

            let current_sample_rate = output_stream_parameters.sample_rate.load(Relaxed);
            if current_sample_rate != previous_sample_rate {
                info!(
                    "Sample rate changed from {previous_sample_rate} Hz to {current_sample_rate} Hz. Reinitializing modules."
                );

                modules = initialize_synth_modules(current_sample_rate);
                previous_sample_rate = current_sample_rate;
            }

            if current_buffer_size != previous_buffer_size {
                info!(
                    "Buffer size changed from {previous_buffer_size} samples to {current_buffer_size} samples."
                );
                previous_buffer_size = current_buffer_size;
            }

            // Process the module parameters per buffer
            modules
                .amp_envelope
                .set_parameters(&module_parameters.envelopes[EnvelopeIndex::Amp as usize]);
            modules
                .filter_envelope
                .set_parameters(&module_parameters.envelopes[EnvelopeIndex::Filter as usize]);
            modules
                .filter_lfo
                .set_parameters(&module_parameters.filter_lfo);
            modules.filter.set_parameters(&module_parameters.filter);
            modules
                .mod_wheel_lfo
                .set_parameters(&module_parameters.mod_wheel_lfo);

            for (index, oscillator) in modules.oscillators.iter_mut().enumerate() {
                oscillator.set_parameters(&module_parameters.oscillators[index]);
                oscillator.tune(current_note.midi_note.load(Relaxed));
            }

            // Begin processing the audio buffer
            let mut quad_mixer_inputs: [MixerInput; 4] =
                synthesizer::create_quad_mixer_inputs(&module_parameters);

            let vibrato_amount =
                load_f32_from_atomic_u32(&module_parameters.keyboard.mod_wheel_amount);
            modules.mod_wheel_lfo.set_range(vibrato_amount / 4.0);

            let output_level = load_f32_from_atomic_u32(&module_parameters.mixer.output_level);
            let output_balance = load_f32_from_atomic_u32(&module_parameters.mixer.output_balance);
            let output_is_muted = module_parameters.mixer.output_is_muted.load(Relaxed);
            let velocity = load_f32_from_atomic_u32(&current_note.velocity);

            // Loop Here

            while local_buffer.len() < stereo_buffer_size {
                // Begin generating and processing the samples for the frame
                modules.filter_envelope.check_gate(
                    &module_parameters.envelopes[EnvelopeIndex::Filter as usize].gate_flag,
                );
                modules.amp_envelope.check_gate(
                    &module_parameters.envelopes[EnvelopeIndex::Amp as usize].gate_flag,
                );
                let vibrato_value = modules.mod_wheel_lfo.generate(None);
                for (index, input) in quad_mixer_inputs.iter_mut().enumerate() {
                    input.sample = modules.oscillators[index].generate(Some(vibrato_value));
                }

                // Any per-oscillator processing should happen before this stereo mix down
                let (oscillator_mix_left, oscillator_mix_right) = quad_mix(quad_mixer_inputs);

                let amp_envelope_value = Some(modules.amp_envelope.generate());

                let (left_envelope_sample, right_envelope_sample) = amplify_stereo(
                    oscillator_mix_left,
                    oscillator_mix_right,
                    Some(velocity),
                    amp_envelope_value,
                );

                let filter_envelope_value = modules.filter_envelope.generate();
                let filter_lfo_value = modules.filter_lfo.generate(None);
                let filter_modulation = filter_envelope_value + filter_lfo_value;

                let (filtered_left, filtered_right) = modules.filter.process(
                    left_envelope_sample,
                    right_envelope_sample,
                    Some(filter_modulation),
                );

                // Final output level control
                let (output_left, output_right) = output_mix(
                    (filtered_left, filtered_right),
                    output_level,
                    output_balance,
                    output_is_muted,
                );

                local_buffer.push(output_left);
                local_buffer.push(output_right);
            }

            // Wait for sample_buffer to have enough capacity to accept local_buffer, then break to restart the main loop
            loop {
                // Check for a new sample buffer producer
                if let Ok(new_sample_buffer) = sample_buffer_receiver.try_recv() {
                    sample_buffer = new_sample_buffer;
                    log::debug!(
                        "create_synthesizer(): Receive a new sample buffer. Replacing the old one"
                    );
                }

                if let Ok(chunk) = sample_buffer.write_chunk_uninit(stereo_buffer_size) {
                    _ = chunk.fill_from_iter(local_buffer.drain(..));
                    break;
                }

                thread::sleep(Duration::from_micros(
                    SAMPLE_PRODUCER_LOOP_SLEEP_DURATION_MICROSECONDS,
                ));
            }
        }
    });

    info!("Synthesizer audio loop was successfully created.");

    Ok(())
}

fn initialize_synth_modules(sample_rate: u32) -> Modules {
    let mut modules = Modules {
        amp_envelope: Envelope::new(sample_rate),
        filter_envelope: Envelope::new(sample_rate),
        filter_lfo: Lfo::new(sample_rate),
        filter: Filter::new(sample_rate),
        mod_wheel_lfo: Lfo::new(sample_rate),
        oscillators: [
            Oscillator::new(sample_rate, WaveShape::default()),
            Oscillator::new(sample_rate, WaveShape::default()),
            Oscillator::new(sample_rate, WaveShape::default()),
            Oscillator::new(sample_rate, WaveShape::default()),
        ],
    };

    modules.oscillators[OscillatorIndex::Sub as usize].set_is_sub_oscillator(true);
    let oscillator_hard_sync_buffer = Arc::new(AtomicBool::new(false));
    modules.oscillators[OscillatorIndex::One as usize]
        .set_hard_sync_role(HardSyncRole::Source(oscillator_hard_sync_buffer.clone()));
    modules.oscillators[OscillatorIndex::Two as usize]
        .set_hard_sync_role(HardSyncRole::Synced(oscillator_hard_sync_buffer.clone()));

    modules
}
