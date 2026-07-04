//! Sequencer logic for timing, note scheduling, and gate management.
//!
//! This module handles:
//! - Clock-synced step tracking based on host tempo
//! - MIDI note latching and scheduling
//! - Gate-length management for held notes
//! - Pending note-off queue to avoid stuck notes

#![allow(dead_code)] // Many fields are prepared for Phase 3

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// Sequencer state for managing rhythm timing and note scheduling.
pub struct Sequencer {
    /// Current step index (shared with UI thread via Arc<AtomicUsize>)
    pub current_step: Arc<AtomicUsize>,

    /// Samples until the next step advance
    pub samples_until_next_step: f64,

    /// Queue of latched notes (pitch, velocity, channel)
    pub latched_notes: Vec<(u8, f32, u8)>,

    /// Pending note-offs: (pitch, channel, `samples_remaining`)
    pub pending_note_offs: Vec<(u8, u8, f64)>,

    /// Whether the pattern has been manually edited (custom mode)
    pub custom_pattern_mode: bool,

    /// Custom pattern if user has toggled steps
    pub custom_pattern: Vec<bool>,
}

impl Sequencer {
    pub fn new() -> Self {
        Self {
            current_step: Arc::new(AtomicUsize::new(0)),
            samples_until_next_step: 0.0,
            latched_notes: Vec::new(),
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
}

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}
