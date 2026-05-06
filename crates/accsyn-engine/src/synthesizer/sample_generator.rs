use crate::modules::amplifier::amplify_stereo;
use crate::modules::effects::Effects;
use crate::modules::envelope::Envelope;
use crate::modules::filter::Filter;
use crate::modules::lfo::Lfo;
use crate::modules::mixer::{MixerInput, output_mix, quad_mix};
use crate::modules::oscillator::{HardSyncRole, Oscillator, WaveShape};
use crate::synthesizer;
use crate::synthesizer::constants::SAMPLE_PRODUCER_LOOP_SLEEP_DURATION_MICROSECONDS;
use crate::synthesizer::{CurrentNote, ModuleParameters};
use accsyn_core::audio_events::OutputStreamParameters;
use accsyn_core::math::load_f32_from_atomic_u32;
use accsyn_core::synth_events::{EnvelopeIndex, LFOIndex, OscillatorIndex};
use anyhow::Result;
use crossbeam_channel::Receiver;
use rtrb::Producer;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

struct Modules {
    amp_envelope: Envelope,
    filter_envelope: Envelope,
    pitch_envelope: Envelope,
    filter_lfo: Lfo,
    filter: Filter,
    mod_wheel_lfo: Lfo,
    effects: Effects,
    oscillators: [Oscillator; 4],
    #[cfg(debug_assertions)]
    profile_counter: u64,
}

/// Creates the synthesizer audio sample generation process thread
///
/// # Errors
///
/// Returns an error if the new sample ring buffer channel receiver is not connected before it tries to start the
/// thread or if the thread cannot be spawned
pub fn sample_generator(
    sample_buffer_receiver: Receiver<Producer<f32>>,
    output_stream_parameters: OutputStreamParameters,
    current_note: &Arc<CurrentNote>,
    module_parameters: &Arc<ModuleParameters>,
) -> Result<()> {
    let current_note = current_note.clone();
    let module_parameters = module_parameters.clone();

    log::debug!(target: "synthesizer::sample_generator", "Blocking till we receive a sample buffer producer from the audio module");
    let sample_buffer = sample_buffer_receiver.recv()?;
    log::debug!(target: "synthesizer::sample_generator", "Sample buffer producer received from the audio module");

    thread::spawn(move || {
        generate_audio_samples(
            &sample_buffer_receiver,
            &output_stream_parameters,
            &current_note,
            &module_parameters,
            sample_buffer,
        );
    });

    log::info!(target: "synthesizer::sample_generator", "Synthesizer audio loop was successfully created.");

    Ok(())
}

// This is the core audio hot path for the application and needs to avoid extra allocations and calls.
// The logic is linear and clear
#[allow(clippy::too_many_lines)]
fn generate_audio_samples(
    sample_buffer_receiver: &Receiver<Producer<f32>>,
    output_stream_parameters: &OutputStreamParameters,
    current_note: &Arc<CurrentNote>,
    module_parameters: &Arc<ModuleParameters>,
    mut sample_buffer: Producer<f32>,
) {
    let sample_rate = output_stream_parameters.sample_rate.load(Relaxed);
    log::info!(target: "synthesizer::sample_generator", "Creating the synthesizer audio loop at sample rate: {sample_rate}");

    let mut previous_sample_rate = 0;
    let mut previous_buffer_size = 0;

    let mut modules = initialize_synth_modules(sample_rate);

    let mut current_buffer_size = output_stream_parameters.buffer_size.load(Relaxed);
    let mut stereo_buffer_size = current_buffer_size as usize * 2;
    let mut local_buffer = Vec::<f32>::with_capacity(stereo_buffer_size);

    loop {
        #[cfg(debug_assertions)]
        let profile_start = std::time::Instant::now();

        current_buffer_size = output_stream_parameters.buffer_size.load(Relaxed);
        stereo_buffer_size = current_buffer_size as usize * 2;

        let current_sample_rate = output_stream_parameters.sample_rate.load(Relaxed);
        if current_sample_rate != previous_sample_rate {
            log::info!(
                target: "synthesizer::sample_generator",
                "Sample rate changed from {previous_sample_rate} Hz to {current_sample_rate} Hz. Reinitializing modules."
            );

            modules = initialize_synth_modules(current_sample_rate);
            previous_sample_rate = current_sample_rate;
        }

        if current_buffer_size != previous_buffer_size {
            log::info!(
                target: "synthesizer::sample_generator",
                "Buffer size changed from {previous_buffer_size} samples to {current_buffer_size} samples."
            );

            if stereo_buffer_size > local_buffer.capacity() {
                local_buffer.reserve(stereo_buffer_size - local_buffer.len());
            }

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
            .pitch_envelope
            .set_parameters(&module_parameters.envelopes[EnvelopeIndex::Pitch as usize]);
        modules
            .filter_lfo
            .set_parameters(&module_parameters.lfos[LFOIndex::Filter as usize]);
        modules.filter.set_parameters(&module_parameters.filter);
        modules
            .mod_wheel_lfo
            .set_parameters(&module_parameters.lfos[LFOIndex::ModWheel as usize]);

        modules.effects.set_parameters(&module_parameters.effects);

        for (index, oscillator) in modules.oscillators.iter_mut().enumerate() {
            oscillator.set_aftertouch(module_parameters.keyboard.aftertouch_amount.load());
            oscillator.set_parameters(&module_parameters.oscillators[index]);
            oscillator.tune(current_note.midi_note.load(Relaxed));
        }

        // Begin processing the audio buffer
        let mut quad_mixer_inputs: [MixerInput; 4] =
            synthesizer::create_quad_mixer_inputs(module_parameters);

        let vibrato_amount = module_parameters.keyboard.mod_wheel_amount.load();
        modules.mod_wheel_lfo.set_range(vibrato_amount / 4.0);

        let output_level = module_parameters.mixer.level.load();
        let output_balance = module_parameters.mixer.balance.load();
        let output_is_muted = module_parameters.mixer.is_muted.load(Relaxed);
        let velocity = load_f32_from_atomic_u32(&current_note.velocity);

        // Loop Here

        while local_buffer.len() < stereo_buffer_size {
            // Begin generating and processing the samples for the frame
            modules
                .filter_envelope
                .check_gate(&module_parameters.envelopes[EnvelopeIndex::Filter as usize].gate_flag);
            modules
                .amp_envelope
                .check_gate(&module_parameters.envelopes[EnvelopeIndex::Amp as usize].gate_flag);
            modules
                .pitch_envelope
                .check_gate(&module_parameters.envelopes[EnvelopeIndex::Pitch as usize].gate_flag);

            let pitch_envelope_value = modules.pitch_envelope.generate();

            let vibrato_value = modules.mod_wheel_lfo.generate(None);

            for (index, input) in quad_mixer_inputs.iter_mut().enumerate() {
                let pitch_envelope_amount = module_parameters.oscillators[index]
                    .pitch_envelope_amount
                    .load();
                input.sample = modules.oscillators[index].generate(
                    Some(vibrato_value),
                    Some(pitch_envelope_value * pitch_envelope_amount),
                );
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

            let (mut effected_left, mut effected_right) =
                modules.effects.process((filtered_left, filtered_right));

            if module_parameters.keyboard.polarity_flipped.load(Relaxed) {
                effected_left *= -1.0;
                effected_right *= -1.0;
            }

            // Final output level control
            let (output_left, output_right) = output_mix(
                (effected_left, effected_right),
                output_level,
                output_balance,
                output_is_muted,
            );

            local_buffer.push(output_left);
            local_buffer.push(output_right);
        }

        // Add a performance counter in debug builds only. use RUST_LOG=hot_loop cargo run to print perf data
        #[cfg(debug_assertions)]
        {
            modules.profile_counter += 1;
            if modules.profile_counter.is_multiple_of(500) {
                let budget_us =
                    f64::from(current_buffer_size) / f64::from(current_sample_rate) * 1_000_000.0;
                let elapsed_us = profile_start.elapsed().as_micros();
                // elapsed_us is a profiling microsecond count; u128→f64 precision loss is acceptable for a percentage log
                #[allow(clippy::cast_precision_loss)]
                let elapsed_us_f64 = elapsed_us as f64;
                log::debug!(target: "hot_loop",
                    "sample_generator: {}μs / {:.0}μs budget ({:.1}%)",
                    elapsed_us,
                    budget_us,
                    elapsed_us_f64 / budget_us * 100.0
                );
            }
        }

        // Wait for sample_buffer to have enough capacity to accept local_buffer, then break to restart the main loop
        loop {
            // Check for a new sample buffer producer
            if let Ok(new_sample_buffer) = sample_buffer_receiver.try_recv() {
                sample_buffer = new_sample_buffer;
                log::debug!(
                    target: "synthesizer::sample_generator",
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
}

fn initialize_synth_modules(sample_rate: u32) -> Modules {
    let mut modules = Modules {
        amp_envelope: Envelope::new(sample_rate),
        filter_envelope: Envelope::new(sample_rate),
        pitch_envelope: Envelope::new(sample_rate),
        filter_lfo: Lfo::new(sample_rate),
        filter: Filter::new(sample_rate),
        mod_wheel_lfo: Lfo::new(sample_rate),
        effects: Effects::new(sample_rate),
        oscillators: [
            Oscillator::new(sample_rate, WaveShape::default()),
            Oscillator::new(sample_rate, WaveShape::default()),
            Oscillator::new(sample_rate, WaveShape::default()),
            Oscillator::new(sample_rate, WaveShape::default()),
        ],
        #[cfg(debug_assertions)]
        profile_counter: 0,
    };

    modules.oscillators[OscillatorIndex::Sub as usize].set_is_sub_oscillator(true);
    let oscillator_hard_sync_buffer = Arc::new(AtomicBool::new(false));
    modules.oscillators[OscillatorIndex::One as usize]
        .set_hard_sync_role(HardSyncRole::Source(oscillator_hard_sync_buffer.clone()));
    modules.oscillators[OscillatorIndex::Two as usize]
        .set_hard_sync_role(HardSyncRole::Synced(oscillator_hard_sync_buffer.clone()));

    modules
}
