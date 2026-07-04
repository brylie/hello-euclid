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
pub mod euclid;
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
pub struct HelloEuclid {
    params: Arc<EuclidParams>,
    sequencer: Sequencer,
}

impl HelloEuclid {
    /// Convert a `NoteValue` to its beat fraction for tempo calculations.
    /// Used to convert rate parameter to samples per step.
    fn note_value_to_beat_fraction(note_value: NoteValue) -> f64 {
        match note_value {
            NoteValue::Quarter => 1.0,
            NoteValue::Eighth => 0.5,
            NoteValue::EighthTriplet => 1.0 / 3.0,
            NoteValue::Sixteenth => 0.25,
            NoteValue::SixteenthTriplet => 1.0 / 6.0,
            NoteValue::ThirtySecond => 0.125,
        }
    }
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
        let sample_rate = f64::from(context.transport().sample_rate);
        let bpm = context.transport().tempo.unwrap_or(120.0);

        // Get beat fraction from the rate parameter
        let beat_fraction = Self::note_value_to_beat_fraction(self.params.rate.value());

        // Calculate samples per step
        let samples_per_step =
            Sequencer::calculate_samples_per_step(bpm, beat_fraction, sample_rate);
        let gate_length = self.params.gate_length.value();

        // Recalculate the Euclidean pattern; params are guaranteed non-negative
        #[allow(clippy::cast_sign_loss)]
        let pulses = (self.params.pulses.value() as usize).min(self.params.steps.value() as usize);
        #[allow(clippy::cast_sign_loss)]
        let steps = self.params.steps.value() as usize;
        #[allow(clippy::cast_sign_loss)]
        let rotation = self.params.rotation.value() as usize;

        let pattern = if self.sequencer.custom_pattern_mode {
            // Use custom pattern if user has manually edited steps
            self.sequencer.custom_pattern.clone()
        } else {
            // Generate Euclidean pattern
            let mut pat = euclid::bjorklund(pulses, steps);
            pat = euclid::rotate(&pat, rotation);
            pat
        };

        let mut next_event = context.next_event();

        for (sample_id, _channel_samples) in buffer.iter_samples().enumerate() {
            // Process all MIDI events at this sample
            // Timing is always within a single buffer, safe to truncate
            #[allow(clippy::cast_possible_truncation)]
            let timing = sample_id as u32;

            while let Some(event) = next_event {
                if event.timing() > timing {
                    break;
                }

                match event {
                    NoteEvent::NoteOn {
                        note,
                        velocity,
                        channel,
                        ..
                    } => {
                        // Latch this note; don't pass it through
                        self.sequencer.latched_note = Some((note, velocity, channel));
                    }
                    NoteEvent::NoteOff { .. } => {
                        // Release the latched note and send all-notes-off
                        self.sequencer.clear_and_send_all_notes_off(context, timing);
                    }
                    _ => {
                        // Ignore other event types
                    }
                }
                next_event = context.next_event();
            }

            // Process pending note-offs
            let _ = self
                .sequencer
                .process_pending_note_offs(context, timing, samples_per_step);

            // Advance sequencer by one sample
            self.sequencer.samples_until_next_step -= 1.0;
            if self.sequencer.samples_until_next_step <= 0.0 {
                self.sequencer.samples_until_next_step += samples_per_step;
                self.sequencer.advance_step(
                    &pattern,
                    gate_length,
                    samples_per_step,
                    context,
                    timing,
                );
            }
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {
        // Clear any pending notes when the plugin is deactivated
        self.sequencer.latched_note = None;
        self.sequencer.pending_note_offs.clear();
    }
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
