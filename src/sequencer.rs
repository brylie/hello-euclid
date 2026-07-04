//! Sequencer logic for timing, note scheduling, and gate management.
//!
//! This module handles:
//! - Clock-synced step tracking based on host tempo
//! - MIDI note latching and scheduling
//! - Gate-length management for held notes
//! - Pending note-off queue to avoid stuck notes

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use nih_plug::prelude::{NoteEvent, ProcessContext};

use crate::HelloEuclid;

/// Sequencer state for managing rhythm timing and note scheduling.
pub struct Sequencer {
    /// Current step index (shared with UI thread via Arc<AtomicUsize>)
    pub current_step: Arc<AtomicUsize>,

    /// Samples until the next step advance
    pub samples_until_next_step: f64,

    /// Latched note: (pitch, velocity, channel)
    pub latched_note: Option<(u8, f32, u8)>,

    /// Pending note-offs: (pitch, channel, `samples_remaining`)
    pub pending_note_offs: Vec<(u8, u8, f64)>,

    /// Whether the pattern has been manually edited (custom mode)
    pub custom_pattern_mode: bool,

    /// Custom pattern if user has toggled steps
    pub custom_pattern: Vec<bool>,
}

impl Sequencer {
    /// Create a new sequencer instance.
    pub fn new() -> Self {
        Self {
            current_step: Arc::new(AtomicUsize::new(0)),
            samples_until_next_step: 0.0,
            latched_note: None,
            pending_note_offs: Vec::new(),
            custom_pattern_mode: false,
            custom_pattern: Vec::new(),
        }
    }

    /// Calculate samples per step based on tempo and note value.
    /// Formula: `(60.0 / bpm) * beat_fraction * sample_rate`
    pub fn calculate_samples_per_step(bpm: f64, beat_fraction: f64, sample_rate: f64) -> f64 {
        (60.0 / bpm) * beat_fraction * sample_rate
    }

    /// Process pending note-offs, decrementing timers and sending note-off events
    /// when the time comes. Returns the count of note-offs sent.
    pub fn process_pending_note_offs(
        &mut self,
        context: &mut impl ProcessContext<HelloEuclid>,
        sample_id: u32,
        _samples_per_step: f64,
    ) -> usize {
        let mut completed_count = 0;

        self.pending_note_offs
            .retain_mut(|(pitch, channel, samples_remaining)| {
                *samples_remaining -= 1.0;

                if *samples_remaining <= 0.0 {
                    context.send_event(NoteEvent::NoteOff {
                        timing: sample_id,
                        voice_id: None,
                        channel: *channel,
                        note: *pitch,
                        velocity: 0.0,
                    });
                    completed_count += 1;
                    false // Remove this entry
                } else {
                    true // Keep this entry
                }
            });

        completed_count
    }

    /// Advance to the next step, firing note-on if the pattern is active at
    /// this step.
    pub fn advance_step(
        &mut self,
        pattern: &[bool],
        gate_length: f32,
        samples_per_step: f64,
        context: &mut impl ProcessContext<HelloEuclid>,
        sample_id: u32,
    ) {
        let n = pattern.len();
        if n == 0 {
            return;
        }

        let current_step = self.current_step.load(Ordering::Relaxed);
        let step_index = current_step % n;

        // Update for UI thread
        self.current_step
            .store(current_step.wrapping_add(1), Ordering::Relaxed);

        // Check if this step is active in the pattern
        if pattern[step_index] {
            if let Some((pitch, velocity, channel)) = self.latched_note {
                // Send note-on
                context.send_event(NoteEvent::NoteOn {
                    timing: sample_id,
                    voice_id: None,
                    channel,
                    note: pitch,
                    velocity,
                });

                // Schedule note-off based on gate length
                let gate_samples = f64::from(gate_length) * samples_per_step;
                self.pending_note_offs.push((pitch, channel, gate_samples));
            }
        }
    }

    /// Clear all latched notes and pending note-offs (called on note-off or
    /// plugin deactivation).
    pub fn clear_and_send_all_notes_off(
        &mut self,
        context: &mut impl ProcessContext<HelloEuclid>,
        sample_id: u32,
    ) {
        // Send all-notes-off for the latched note if it exists
        if let Some((pitch, _, channel)) = self.latched_note.take() {
            context.send_event(NoteEvent::NoteOff {
                timing: sample_id,
                voice_id: None,
                channel,
                note: pitch,
                velocity: 0.0,
            });
        }

        // Clear pending note-offs (they would have timed out anyway)
        self.pending_note_offs.clear();
    }
}

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}
