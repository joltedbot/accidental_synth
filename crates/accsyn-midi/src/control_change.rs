use accsyn_types::midi_events::CC;

pub fn get_supported_cc_from_cc_number(cc_number: u8, cc_value: u8) -> Option<CC> {
    match cc_number {
        1 => Some(CC::ModWheel(cc_value)),
        3 => Some(CC::VelocityCurve(cc_value)),
        5 => Some(CC::PitchBendRange(cc_value)),
        7 => Some(CC::Volume(cc_value)),
        8 => Some(CC::Mute(cc_value)),
        10 => Some(CC::Balance(cc_value)),
        12 => Some(CC::SubOscillatorShapeParameter1(cc_value)),
        13 => Some(CC::SubOscillatorShapeParameter2(cc_value)),
        14 => Some(CC::Oscillator1ShapeParameter1(cc_value)),
        15 => Some(CC::Oscillator1ShapeParameter2(cc_value)),
        16 => Some(CC::Oscillator2ShapeParameter1(cc_value)),
        17 => Some(CC::Oscillator2ShapeParameter2(cc_value)),
        18 => Some(CC::Oscillator3ShapeParameter1(cc_value)),
        19 => Some(CC::Oscillator3ShapeParameter2(cc_value)),
        20 => Some(CC::OscillatorKeySyncEnabled(cc_value)),
        37 => Some(CC::PortamentoTime(cc_value)),
        38 => Some(CC::OscillatorHardSync(cc_value)),
        40 => Some(CC::SubOscillatorShape(cc_value)),
        41 => Some(CC::Oscillator1Shape(cc_value)),
        42 => Some(CC::Oscillator2Shape(cc_value)),
        43 => Some(CC::Oscillator3Shape(cc_value)),
        44 => Some(CC::SubOscillatorCourseTune(cc_value)),
        45 => Some(CC::Oscillator1CourseTune(cc_value)),
        46 => Some(CC::Oscillator2CourseTune(cc_value)),
        47 => Some(CC::Oscillator3CourseTune(cc_value)),
        48 => Some(CC::SubOscillatorFineTune(cc_value)),
        49 => Some(CC::Oscillator1FineTune(cc_value)),
        50 => Some(CC::Oscillator2FineTune(cc_value)),
        51 => Some(CC::Oscillator3FineTune(cc_value)),
        52 => Some(CC::SubOscillatorLevel(cc_value)),
        53 => Some(CC::Oscillator1Level(cc_value)),
        54 => Some(CC::Oscillator2Level(cc_value)),
        55 => Some(CC::Oscillator3Level(cc_value)),
        56 => Some(CC::SubOscillatorMute(cc_value)),
        57 => Some(CC::Oscillator1Mute(cc_value)),
        58 => Some(CC::Oscillator2Mute(cc_value)),
        59 => Some(CC::Oscillator3Mute(cc_value)),
        60 => Some(CC::SubOscillatorBalance(cc_value)),
        61 => Some(CC::Oscillator1Balance(cc_value)),
        62 => Some(CC::Oscillator2Balance(cc_value)),
        63 => Some(CC::Oscillator3Balance(cc_value)),
        64 => Some(CC::Sustain(cc_value)),
        65 => Some(CC::PortamentoEnabled(cc_value)),
        66 => Some(CC::SubOscillatorClipBoost(cc_value)),
        67 => Some(CC::Oscillator1ClipBoost(cc_value)),
        68 => Some(CC::Oscillator2ClipBoost(cc_value)),
        69 => Some(CC::Oscillator3ClipBoost(cc_value)),
        70 => Some(CC::FilterPoles(cc_value)),
        71 => Some(CC::FilterResonance(cc_value)),
        72 => Some(CC::AmpEGReleaseTime(cc_value)),
        73 => Some(CC::AmpEGAttackTime(cc_value)),
        74 => Some(CC::FilterCutoff(cc_value)),
        75 => Some(CC::AmpEGDecayTime(cc_value)),
        79 => Some(CC::AmpEGSustainLevel(cc_value)),
        80 => Some(CC::AmpEGInverted(cc_value)),
        85 => Some(CC::FilterEnvelopeAttackTime(cc_value)),
        86 => Some(CC::FilterEnvelopeDecayTime(cc_value)),
        87 => Some(CC::FilterEnvelopeSustainLevel(cc_value)),
        88 => Some(CC::FilterEnvelopeReleaseTime(cc_value)),
        89 => Some(CC::FilterEnvelopeInverted(cc_value)),
        90 => Some(CC::FilterEnvelopeAmount(cc_value)),
        91 => Some(CC::KeyTrackingAmount(cc_value)),
        102 => Some(CC::ModWheelLFOFrequency(cc_value)),
        103 => Some(CC::ModWheelLFOCenterValue(cc_value)),
        104 => Some(CC::ModWheelLFORange(cc_value)),
        105 => Some(CC::ModWheelLFOWaveShape(cc_value)),
        106 => Some(CC::ModWheelLFOPhase(cc_value)),
        107 => Some(CC::ModWheelLFOReset),
        108 => Some(CC::FilterModLFOFrequency(cc_value)),
        109 => Some(CC::FilterModLFOAmount(cc_value)),
        110 => Some(CC::FilterModLFOWaveShape(cc_value)),
        111 => Some(CC::FilterModLFOPhase(cc_value)),
        112 => Some(CC::FilterModLFOReset),
        123 => Some(CC::AllNotesOff),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_supported_cc_returns_some_for_known_ccs() {
        assert_eq!(
            get_supported_cc_from_cc_number(1, 64),
            Some(CC::ModWheel(64))
        );
        assert_eq!(
            get_supported_cc_from_cc_number(74, 100),
            Some(CC::FilterCutoff(100))
        );
        assert_eq!(
            get_supported_cc_from_cc_number(107, 0),
            Some(CC::ModWheelLFOReset)
        );
        assert_eq!(
            get_supported_cc_from_cc_number(123, 0),
            Some(CC::AllNotesOff)
        );
    }

    #[test]
    fn get_supported_cc_returns_none_for_out_of_range_cc_number() {
        assert_eq!(get_supported_cc_from_cc_number(200, 127), None);
    }
}
