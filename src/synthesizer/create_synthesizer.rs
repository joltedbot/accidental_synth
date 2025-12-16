use crate::math::load_f32_from_atomic_u32;
use crate::modules::amplifier::amplify_stereo;
use crate::modules::envelope::Envelope;
use crate::modules::filter::Filter;
use crate::modules::lfo::Lfo;
use crate::modules::mixer::{MixerInput, output_mix, quad_mix};
use crate::modules::oscillator::{HardSyncRole, Oscillator, WaveShape};
use crate::synthesizer;
use crate::synthesizer::constants::{
    LOCAL_BUFFER_CAPACITY, SAMPLE_PRODUCER_LOOP_SLEEP_DURATION_MICROSECONDS,
};
use crate::synthesizer::{CurrentNote, ModuleParameters, OscillatorIndex};
use anyhow::Result;
use crossbeam_channel::Receiver;
use rtrb::Producer;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;
use crate::audio::OutputStreamParameters;

pub fn create_synthesizer(
    sample_buffer_receiver: Receiver<Producer<f32>>,
    output_stream_parameters: OutputStreamParameters,
    current_note: &Arc<CurrentNote>,
    module_parameters: &Arc<ModuleParameters>,
) -> Result<()> {
    let current_note = current_note.clone();
    let module_parameters = module_parameters.clone();

    let sample_rate = output_stream_parameters.sample_rate.load(Relaxed);
    
    log::info!("Initializing the synthesizer audio creation loop");
    let mut filter = Filter::new(sample_rate);
    let mut amp_envelope = Envelope::new(sample_rate);
    let mut filter_envelope = Envelope::new(sample_rate);
    let mut filter_lfo = Lfo::new(sample_rate);
    let mut mod_wheel_lfo = Lfo::new(sample_rate);

    let mut oscillators = [
        Oscillator::new(sample_rate, WaveShape::default()),
        Oscillator::new(sample_rate, WaveShape::default()),
        Oscillator::new(sample_rate, WaveShape::default()),
        Oscillator::new(sample_rate, WaveShape::default()),
    ];
    oscillators[OscillatorIndex::Sub as usize].set_is_sub_oscillator(true);

    let oscillator_hard_sync_buffer = Arc::new(AtomicBool::new(false));
    oscillators[OscillatorIndex::One as usize]
        .set_hard_sync_role(HardSyncRole::Source(oscillator_hard_sync_buffer.clone()));
    oscillators[OscillatorIndex::Two as usize]
        .set_hard_sync_role(HardSyncRole::Synced(oscillator_hard_sync_buffer.clone()));

    log::debug!("Blocking till we receive a sample buffer producer from the audio module");
    let mut sample_buffer = sample_buffer_receiver.recv()?;
    log::debug!("Sample buffer producer received from the audio module");

    log::info!("Creating the synthesizer audio loop at sample rate: {sample_rate}");

   thread::spawn(move || {
        loop {
            // Process the module parameters per buffer
            amp_envelope.set_parameters(&module_parameters.amp_envelope);
            filter_envelope.set_parameters(&module_parameters.filter_envelope);
            filter_lfo.set_parameters(&module_parameters.filter_lfo);
            filter.set_parameters(&module_parameters.filter);
            mod_wheel_lfo.set_parameters(&module_parameters.mod_wheel_lfo);

            for (index, oscillator) in oscillators.iter_mut().enumerate() {
                oscillator.set_parameters(&module_parameters.oscillators[index]);
                oscillator.tune(current_note.midi_note.load(Relaxed));
            }

            // Begin processing the audio buffer
            let mut quad_mixer_inputs: [MixerInput; 4] =
                synthesizer::create_quad_mixer_inputs(&module_parameters);

            let vibrato_amount =
                load_f32_from_atomic_u32(&module_parameters.keyboard.mod_wheel_amount);
            mod_wheel_lfo.set_range(vibrato_amount / 4.0);

            // Loop Here
            let mut local_buffer = Vec::<f32>::with_capacity(LOCAL_BUFFER_CAPACITY);

            while local_buffer.len() < LOCAL_BUFFER_CAPACITY {
                // Begin generating and processing the samples for the frame
                let vibrato_value = mod_wheel_lfo.generate(None);
                for (index, input) in quad_mixer_inputs.iter_mut().enumerate() {
                    input.sample = oscillators[index].generate(Some(vibrato_value));
                }

                // Any per-oscillator processing should happen before this stereo mix down
                let (oscillator_mix_left, oscillator_mix_right) = quad_mix(quad_mixer_inputs);

                let amp_envelope_value = Some(amp_envelope.generate());

                let (left_envelope_sample, right_envelope_sample) = amplify_stereo(
                    oscillator_mix_left,
                    oscillator_mix_right,
                    Some(load_f32_from_atomic_u32(&current_note.velocity)),
                    amp_envelope_value,
                );

                let filter_envelope_value = filter_envelope.generate();
                let filter_lfo_value = filter_lfo.generate(None);
                let filter_modulation = filter_envelope_value + filter_lfo_value;

                let (filtered_left, filtered_right) = filter.process(
                    left_envelope_sample,
                    right_envelope_sample,
                    Some(filter_modulation),
                );

                // Final output level control
                let output_level = load_f32_from_atomic_u32(&module_parameters.mixer.output_level);
                let output_balance =
                    load_f32_from_atomic_u32(&module_parameters.mixer.output_balance);
                let output_is_muted = module_parameters.mixer.output_is_muted.load(Relaxed);

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

                if let Ok(chunk) = sample_buffer.write_chunk_uninit(LOCAL_BUFFER_CAPACITY) {
                    _ = chunk.fill_from_iter(local_buffer);
                    break;
                }

                thread::sleep(Duration::from_micros(
                    SAMPLE_PRODUCER_LOOP_SLEEP_DURATION_MICROSECONDS,
                ));
            }
        }
   });


    log::info!("Synthesizer audio loop was successfully created.");

    Ok(())
}
