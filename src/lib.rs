//! Hello Euclid — A Euclidean rhythm MIDI effect plugin
//!
//! Takes a held MIDI note as input and re-triggers it in a Euclidean rhythm pattern,
//! synced to host tempo. Exposes pattern parameters (pulses, steps, rotation, rate, gate length)
//! for real-time tweaking and automation.

#![warn(
    clippy::all,
    clippy::pedantic,
    missing_docs,
    unsafe_code,
    unused_results
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::items_after_statements
)]

use nih_plug::prelude::*;
use std::sync::Arc;

pub mod editor;
mod euclid;
mod sequencer;

use sequencer::Sequencer;

/// Parameter definitions for the Euclidean rhythm plugin.
#[derive(Params)]
pub struct EuclidParams {
    /// Number of active steps (pulses) in the Euclidean pattern.
    /// Range: 1..=16
    #[id = "pulses"]
    pub pulses: IntParam,

    /// Total number of steps in the pattern.
    /// Range: 1..=32, with pulses clamped to not exceed steps
    #[id = "steps"]
    pub steps: IntParam,

    /// Rotation offset for the pattern (phase shift).
    /// Range: 0..=31, wraps via modulo at use site
    #[id = "rotation"]
    pub rotation: IntParam,

    /// Note repeat rate, synced to host tempo.
    /// Options: 1/4, 1/8, 1/8T, 1/16, 1/16T, 1/32
    #[id = "rate"]
    pub rate: EnumParam<NoteValue>,

    /// Gate length as a fraction of step duration.
    /// Range: 0.05..=1.0 (5% to 100%)
    #[id = "gate_length"]
    pub gate_length: FloatParam,
}

impl Default for EuclidParams {
    fn default() -> Self {
        Self {
            pulses: IntParam::new("Pulses", 3, IntRange::Linear { min: 1, max: 16 }),
            steps: IntParam::new("Steps", 8, IntRange::Linear { min: 1, max: 32 }),
            rotation: IntParam::new("Rotation", 0, IntRange::Linear { min: 0, max: 31 }),
            rate: EnumParam::new("Rate", NoteValue::Eighth),
            gate_length: FloatParam::new(
                "Gate Length",
                0.5,
                FloatRange::Linear {
                    min: 0.05,
                    max: 1.0,
                },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

/// Supported note rates for the sequencer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum NoteValue {
    /// Quarter note (1/4)
    Quarter,
    /// Eighth note (1/8)
    Eighth,
    /// Eighth note triplet (1/8T)
    EighthTriplet,
    /// Sixteenth note (1/16)
    Sixteenth,
    /// Sixteenth note triplet (1/16T)
    SixteenthTriplet,
    /// Thirty-second note (1/32)
    ThirtySecond,
}

/// The main Hello Euclid plugin struct.
#[allow(dead_code)] // sequencer field used in Phase 3
pub struct HelloEuclid {
    params: Arc<EuclidParams>,
    sequencer: Sequencer,
}

impl Default for HelloEuclid {
    fn default() -> Self {
        Self {
            params: Arc::new(EuclidParams::default()),
            sequencer: Sequencer::new(),
        }
    }
}

impl Plugin for HelloEuclid {
    const NAME: &'static str = "Hello Euclid";
    const VENDOR: &'static str = "Brylie";
    const URL: &'static str = "https://github.com/RustAudio/nih-plug";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: None,
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        None // TODO: Implement GUI in Phase 4
    }

    fn initialize(
        &mut self,
        _audio_io_layouts: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // For now, pass through MIDI events unchanged (no-op mode)
        // This allows testing the MIDI IO layout in the host
        let mut next_event = context.next_event();

        for (sample_id, _channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() > u32::try_from(sample_id).unwrap_or(u32::MAX) {
                    break;
                }

                // Pass through all MIDI events for now
                context.send_event(event);
                next_event = context.next_event();
            }
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl ClapPlugin for HelloEuclid {
    const CLAP_ID: &'static str = "com.example.hello-euclid";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Euclidean rhythm MIDI effect");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect];
}

impl Vst3Plugin for HelloEuclid {
    const VST3_CLASS_ID: [u8; 16] = *b"HelloEuclidVST3\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let _plugin = HelloEuclid::default();
        // Just verify the plugin can be instantiated without panicking
    }
}

// Export the plugin for VST3 and CLAP hosts
nih_export_vst3!(HelloEuclid);
nih_export_clap!(HelloEuclid);
