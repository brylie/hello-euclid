# Hello Euclid — Rust MIDI Plugin Specification

A Euclidean note-repeat / arpeggiator MIDI effect plugin, built in Rust with
nih-plug, targeting VST3 (Windows/macOS/Linux) for use in Cubase and other
VST3 hosts.

This document is a build spec intended for implementation with Claude Code.
It defines architecture, algorithms, parameters, and known integration
gotchas, but leaves specific code structure decisions to the implementer.

---

## 1. Goal

Recreate and extend the core idea of BitWig's Note Repeat "Euclid" mode as a
standalone, host-agnostic VST3 MIDI effect:

- Takes a held MIDI note (or chord) as input from a MIDI track
- Re-triggers that note in a Euclidean rhythm pattern, synced to host tempo
- Exposes pattern parameters (pulses, steps, rotation, rate, gate length)
  for real-time tweaking and automation
- Provides a 2D circular visualization of the pattern with live playhead

This is a MIDI-only effect: no audio input/output ports. It must declare
itself correctly so hosts route it as a MIDI insert on a MIDI/instrument
track rather than expecting it to process audio.

---

## 2. Target environment

- **Language:** Rust, stable toolchain
- **Framework:** nih-plug (check whether `robbert-vdh/nih-plug` — currently
  in maintenance mode — or its active community fork is the better base at
  time of implementation)
- **GUI:** `nih_plug_egui`, using egui's raw `Painter` for a custom circular
  step-sequencer canvas
- **Plugin format:** VST3 (required for Cubase — as of Cubase 15, Steinberg
  has not added CLAP support). Also export CLAP since nih-plug supports it
  for free and it's useful for testing in Bitwig/REAPER.
- **Platforms:** Windows, macOS, Linux (nih-plug's default cross-platform
  bundling via `cargo xtask bundle`)
- **Editor:** VS Code with rust-analyzer, CodeLLDB, and clippy-based checks

### VS Code setup

`.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.check.command": "clippy"
}
```

`.vscode/launch.json` (debug the standalone build; debugging inside a
running Cubase host process is possible via LLDB attach but is not the
primary workflow):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug standalone",
      "cargo": { "args": ["build", "--bin=hello_euclid"] },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

---

## 3. Project structure

```
hello_euclid/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Plugin trait impl, param definitions, process()
│   ├── euclid.rs        # Bjorklund algorithm — pure logic, unit-testable
│   ├── sequencer.rs      # Clock-synced step tracking, note scheduling
│   └── editor/
│       ├── mod.rs        # egui editor setup, shared state wiring
│       └── canvas.rs      # 2D circular step visualization + interaction
```

---

## 4. Cargo.toml

```toml
[package]
name = "hello_euclid"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git" }
nih_plug_egui = { git = "https://github.com/robbert-vdh/nih-plug.git" }

[profile.release]
lto = "thin"
strip = true
```

Verify the exact dependency source before implementation — if the
community fork of nih-plug has diverged meaningfully, prefer it for new
projects.

---

## 5. AudioIOLayout: MIDI-only declaration

This is the part most likely to cause silent host-integration failures if
done wrong. The plugin must declare zero audio input/output channels and
enable MIDI input/output, so hosts treat it as a MIDI effect rather than an
instrument or audio processor.

```rust
impl Plugin for HelloEuclid {
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: None,
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;

    // ... remaining Plugin trait items
}
```

**Test this early.** Before building out the rhythm engine or UI, get a
no-op passthrough plugin with this exact IO layout building, bundled, and
loaded correctly as a MIDI insert in Cubase. Historically some hosts
(Cubase included) have been pickier about MIDI-effect-only plugins than
about instruments or audio FX — confirming the scan and insert workflow
first avoids debugging two problems at once later.

---

## 6. Core algorithm: Euclidean rhythm generation (`euclid.rs`)

Pure, deterministic, no real-time dependencies — fully unit-testable in
isolation from the audio thread.

```rust
pub fn bjorklund(pulses: usize, steps: usize) -> Vec<bool> {
    if pulses == 0 || steps == 0 { return vec![false; steps]; }
    if pulses >= steps { return vec![true; steps]; }

    let mut groups: Vec<Vec<bool>> = (0..pulses).map(|_| vec![true]).collect();
    let mut remainders: Vec<Vec<bool>> = (0..(steps - pulses)).map(|_| vec![false]).collect();

    while remainders.len() > 1 {
        let combine_count = groups.len().min(remainders.len());
        let mut new_groups = Vec::with_capacity(combine_count);
        for i in 0..combine_count {
            let mut g = groups[i].clone();
            g.extend(remainders[i].clone());
            new_groups.push(g);
        }
        let leftover_groups = groups.split_off(combine_count.min(groups.len()));
        let leftover_remainders = remainders.split_off(combine_count.min(remainders.len()));

        groups = new_groups;
        remainders = if !leftover_groups.is_empty() { leftover_groups } else { leftover_remainders };
    }

    groups.into_iter().chain(remainders).flatten().collect()
}

/// Rotate a pattern by `offset` steps (for the rotation parameter).
pub fn rotate(pattern: &[bool], offset: usize) -> Vec<bool> {
    if pattern.is_empty() { return vec![]; }
    let n = pattern.len();
    let offset = offset % n;
    pattern[offset..].iter().chain(pattern[..offset].iter()).copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e_3_8() {
        // Classic Cuban tresillo: X..X..X.
        let result = bjorklund(3, 8);
        assert_eq!(result, vec![true, false, false, true, false, false, true, false]);
    }

    #[test]
    fn test_pulses_equal_steps() {
        assert_eq!(bjorklund(4, 4), vec![true, true, true, true]);
    }

    #[test]
    fn test_zero_pulses() {
        assert_eq!(bjorklund(0, 8), vec![false; 8]);
    }

    #[test]
    fn test_rotation() {
        let pattern = vec![true, false, false, true];
        assert_eq!(rotate(&pattern, 1), vec![false, false, true, true]);
    }
}
```

Add test cases for other canonical Euclidean rhythms (E(5,8), E(2,5),
E(7,16)) against known reference patterns to build confidence before wiring
into the sequencer.

---

## 7. Parameters

```rust
#[derive(Params)]
struct EuclidParams {
    #[id = "pulses"]
    pub pulses: IntParam,       // range 1..=16

    #[id = "steps"]
    pub steps: IntParam,        // range 1..=32, pulses <= steps enforced

    #[id = "rotation"]
    pub rotation: IntParam,     // range 0..=31, wraps via modulo at use site

    #[id = "rate"]
    pub rate: EnumParam<NoteValue>,  // 1/4, 1/8, 1/8T, 1/16, 1/16T, 1/32

    #[id = "gate_length"]
    pub gate_length: FloatParam, // 0.05..=1.0, fraction of step duration

    #[id = "octave_range"]
    pub octave_range: IntParam,  // 0..=3, optional v2 feature: octave-jump variation
}
```

`pulses` should be clamped so it never exceeds `steps` — validate in the
param's `.with_callback()` or at read-time in `process()`.

---

## 8. Sequencer / timing logic (`sequencer.rs`)

This is the highest-risk section for real-time correctness bugs. Key
architecture points:

- Each `process()` call, read `context.transport()` for tempo, sample
  position, and playing state.
- Convert the `rate` param into samples-per-step using tempo and sample
  rate: `samples_per_step = (60.0 / bpm) * beat_fraction * sample_rate`.
- Maintain `current_step: usize` and `samples_until_next_step: f64` as
  plugin state, decremented per-sample within the buffer loop.
- **Input note handling:** when a MIDI note-on arrives from the host,
  latch it as the "seed" pitch/velocity/channel — do not pass it through
  unmodified. The plugin generates new note-on/off events at that pitch,
  gated by the Euclidean pattern.
- **Input note-off:** clear the latched note and immediately send an
  all-notes-off for the pattern's currently active pitch, to avoid stuck
  notes if a step is mid-gate when the source note releases.
- **Gate length:** don't fire note-off immediately after note-on at each
  active step. Schedule it `gate_length * samples_per_step` later via a
  small pending-note-off queue (a `Vec` or ring buffer of
  `(pitch, channel, samples_remaining)` is sufficient at this scale).
- **Chord input (stretch goal):** if multiple notes are held, either (a)
  cycle through them per step, or (b) fire polyphonic Euclidean patterns
  per note — decide behavior explicitly rather than leaving it undefined,
  since untested polyphonic input is a likely source of stuck-note bugs.

```rust
fn process(
    &mut self,
    buffer: &mut Buffer,
    _aux: &mut AuxiliaryBuffers,
    context: &mut impl ProcessContext<Self>,
) -> ProcessStatus {
    let mut next_event = context.next_event();

    for (sample_id, _channel_samples) in buffer.iter_samples().enumerate() {
        while let Some(event) = next_event {
            if event.timing() > sample_id as u32 { break; }
            match event {
                NoteEvent::NoteOn { note, velocity, channel, .. } => {
                    self.latched_note = Some((note, velocity, channel));
                }
                NoteEvent::NoteOff { .. } => {
                    if let Some((note, _, channel)) = self.latched_note.take() {
                        context.send_event(NoteEvent::NoteOff {
                            timing: sample_id as u32,
                            voice_id: None,
                            channel,
                            note,
                            velocity: 0.0,
                        });
                    }
                }
                _ => {}
            }
            next_event = context.next_event();
        }

        self.process_pending_note_offs(context, sample_id as u32);

        self.samples_until_next_step -= 1.0;
        if self.samples_until_next_step <= 0.0 {
            self.advance_step(context, sample_id as u32);
        }
    }

    ProcessStatus::Normal
}
```

Fill in `advance_step` and `process_pending_note_offs` per the gate-length
and pattern-lookup logic described above.

---

## 9. GUI: circular step visualization (`editor/canvas.rs`)

Use egui's `Painter` directly rather than a widget library, since this is
a custom visualization, not a standard control.

```rust
fn draw_euclid_circle(ui: &mut egui::Ui, pattern: &[bool], current_step: usize) {
    let (response, painter) = ui.allocate_painter(egui::vec2(300.0, 300.0), egui::Sense::click());
    let center = response.rect.center();
    let radius = 120.0;
    let n = pattern.len().max(1);

    for (i, &active) in pattern.iter().enumerate() {
        let angle = (i as f32 / n as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let pos = center + egui::vec2(angle.cos(), angle.sin()) * radius;

        let color = if i == current_step {
            egui::Color32::from_rgb(255, 200, 50)   // playhead
        } else if active {
            egui::Color32::from_rgb(80, 160, 255)   // active pulse
        } else {
            egui::Color32::from_gray(60)             // inactive step
        };

        painter.circle_filled(pos, if active { 10.0 } else { 6.0 }, color);

        if response.clicked() {
            if let Some(click_pos) = response.interact_pointer_pos() {
                if (click_pos - pos).length() < 12.0 {
                    // Toggle this step; switch pattern source to "custom"
                    // mode so manual edits aren't overwritten by the next
                    // Euclidean recalculation.
                }
            }
        }
    }
}
```

### Thread-safety for the playhead

The GUI and audio-processing run on different threads. Do not lock across
them. Share `current_step` via `Arc<AtomicUsize>`:

- Audio thread: `current_step.store(step, Ordering::Relaxed)` when
  advancing.
- GUI thread: `current_step.load(Ordering::Relaxed)` each redraw frame.

This is the standard nih-plug pattern for exposing real-time state to the
editor without introducing audio-thread blocking.

---

## 10. Feature notes for future iteration (not required for v1)

- **Rotation** as a distinct parameter from pulses/steps gives phase
  offset without changing the pattern's density — this is the "groove"
  control equivalent to BitWig's version.
- **Dual Euclidean layers**: a second independent pulses/steps pattern
  modulating velocity/accent instead of gate produces polyrhythmic accent
  patterns using the same underlying algorithm — no new math needed, just
  a second pattern buffer and a different consumption path in
  `advance_step`.
- **Reusability**: `euclid.rs` has no plugin-framework dependencies and
  could be extracted as a standalone crate later — reusable in a CLI tool,
  a generative-composition script, or a different plugin entirely.

---

## 11. Build and test workflow

1. **Unit tests first**: run `cargo test` against `euclid.rs` before
   touching plugin scaffolding at all.
2. **Standalone build**: `cargo xtask bundle hello_euclid --release`
   using the standalone target to verify MIDI logic without DAW variables
   in play. Feed it a virtual MIDI input if testing standalone playback.
3. **IO layout smoke test**: build a no-op version (see Section 5) and
   confirm Cubase correctly scans it and offers it as a MIDI insert
   before adding the rhythm engine.
4. **VST3 export**: build the full plugin, copy the bundle to your VST3
   folder, rescan in Cubase.
5. **Insert on a MIDI track** (not an instrument slot) and confirm:
   - Held notes produce gated repeats matching the visible pattern
   - Note-off cleanly stops the pattern with no stuck notes
   - Parameter automation (pulses, steps, rate) responds smoothly without
     audio/MIDI glitches when changed during playback
   - Tempo changes mid-playback correctly rescale step timing

---

## 12. Open questions to resolve during implementation

- Which nih-plug lineage to build against (`robbert-vdh/nih-plug` vs.
  active community fork) — check current state at implementation time.
- Chord/polyphonic input behavior (cycle vs. simultaneous per-note
  patterns) — decide and document before writing tests for it.
- Whether "custom" step-toggling in the UI should persist across
  pulses/steps parameter changes, or reset to the Euclidean default —
  affects state management design in `sequencer.rs`.
