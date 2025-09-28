use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use cpal::Stream;
use cpal::traits::DeviceTrait;
use crate::audio::OutputDevice;
use crate::math::load_f32_from_atomic_u32;
use crate::modules::amplifier::amplify_stereo;
use crate::modules::envelope::Envelope;
use crate::modules::filter::Filter;
use crate::modules::lfo::Lfo;
use crate::modules::mixer::{output_mix, quad_mix, MixerInput};
use crate::modules::oscillator::{HardSyncRole, Oscillator, WaveShape};
use crate::synthesizer;
use crate::synthesizer::{CurrentNote, ModuleParameters, OscillatorIndex};

pub fn create_synthesizer(
    output_device: &OutputDevice,
    sample_rate: u32,
    current_note: &Arc<CurrentNote>,
    module_parameters: &Arc<ModuleParameters>,
) -> anyhow::Result<Stream> {
    let current_note = current_note.clone();
    let module_parameters = module_parameters.clone();

    log::info!("Initializing the filter module");
    let mut filter = Filter::new(sample_rate);
    let mut amp_envelope = Envelope::new(sample_rate);
    let mut filter_envelope = Envelope::new(sample_rate);
    let mut filter_lfo = Lfo::new(sample_rate);
    let mut lfo1 = Lfo::new(sample_rate);

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

    let default_device_stream_config = output_device.device.default_output_config()?.config();
    let number_of_channels = output_device.channels.total;
    let left_channel_index = output_device.channels.left;
    let right_channel_index = output_device.channels.right;

    log::info!(
        "Creating the synthesizer audio output stream for the device {} with {} channels at sample rate: {}",
        output_device.name,
        number_of_channels,
        sample_rate
    );

    let stream = output_device.device.build_output_stream(
        &default_device_stream_config,
        move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Process the module parameters per buffer
            amp_envelope.set_parameters(&module_parameters.amp_envelope);
            filter_envelope.set_parameters(&module_parameters.filter_envelope);
            filter_lfo.set_parameters(&module_parameters.filter_lfo);
            filter.set_parameters(&module_parameters.filter);
            lfo1.set_parameters(&module_parameters.lfo1);

            for (index, oscillator) in oscillators.iter_mut().enumerate() {
                oscillator.set_parameters(&module_parameters.oscillators[index]);
                oscillator.tune(current_note.midi_note.load(Relaxed));
            }

            // Begin processing the audio buffer
            let mut quad_mixer_inputs: [MixerInput; 4] =
                synthesizer::create_quad_mixer_inputs(&module_parameters);

            let vibrato_amount =
                load_f32_from_atomic_u32(&module_parameters.keyboard.mod_wheel_amount);
            lfo1.set_range(vibrato_amount / 4.0);

            // Split the buffer into frames
            for frame in buffer.chunks_mut(number_of_channels as usize) {
                // Begin generating and processing the samples for the frame
                let vibrato_value = lfo1.generate(None);
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

                let (output_left, output_right) =
                    output_mix(filtered_left, filtered_right, output_level, output_balance);

                // Hand back the processed samples to the frame to be sent to the audio device
                frame[left_channel_index] = output_left;

                // For mono devices just drop the right sample
                if let Some(index) = right_channel_index {
                    frame[index] = output_right;
                }
            }
        },
        |err| {
            log::error!("create_synthesizer(): Error in audio output stream: {err}");
        },
        None,
    )?;

    log::info!("Synthesizer audio output stream was successfully created.");

    Ok(stream)
}