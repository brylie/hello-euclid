# Hello Euclid

A Euclidean rhythm MIDI effect plugin for note re-triggering and pattern
sequencing. Hold a MIDI note and let the plugin generate polyrhythmic patterns
in real time.

## What It Does

**Hello Euclid** takes a held MIDI note (or chord) and re-triggers it in a
Euclidean rhythm pattern, synced to your host's tempo. This creates organic,
evenly-spaced patterns that work great for:

- Syncopated drum and percussion sequences
- Polyrhythmic arpeggios
- Melodic motif generation
- Creating complex rhythms without manual sequencing

### Example

Hold a single note, set **pulses=3, steps=8**. You hear the note repeated in
the classic Cuban tresillo pattern: **X . . X . . X .**

Adjust **rotation** to shift the pattern. Increase **gate_length** to overlap
notes. Change **rate** to speed up or slow down playback.

## Installation

### macOS (VST3)

1. Download the latest release bundle or build from source (see below)
2. Copy `HelloEuclid.vst3` to `~/Library/Audio/Plug-Ins/VST3/`
3. Rescan plugins in your DAW

### macOS (CLAP)

1. Copy `HelloEuclid.clap` to `~/Library/Audio/Plug-Ins/CLAP/`
2. Rescan plugins in your DAW

### Windows & Linux

Binaries not yet available; see **Building from Source** below.

## Using in Your DAW

### Cubase

1. Create a new MIDI track
2. Insert **Hello Euclid** as a MIDI FX (Insert > MIDI Inserts > Hello Euclid)
3. Hold a note on your MIDI controller or piano roll
4. Adjust parameters in the plugin UI
5. The re-triggered notes appear in real time

### Ableton Live

1. Create a new MIDI track
2. Drag **Hello Euclid** into the MIDI effects chain
3. Hold a note; the plugin generates the pattern
4. Automate parameters for dynamic rhythm changes

### Logic Pro

1. Create a new Software Instrument track
2. Scroll to MIDI FX and select **Hello Euclid**
3. Record-arm the track and hold a note (or use a clip)
4. Tweak parameters in the plugin window

### Bitwig Studio

1. Add a new MIDI FX device to your MIDI track
2. Locate **Hello Euclid** in the device browser
3. Hold a note; the pattern triggers
4. Use the modulation system to automate pulses/steps/rotation for live
   control

### REAPER

1. Add a new MIDI FX to your MIDI track
2. Select **Hello Euclid** from the FX browser
3. Pin the plugin window for easy tweaking
4. Hold a note on your controller; the pattern plays

## Parameters

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Pulses** | 1–16 | Number of hits in the pattern |
| **Steps** | 1–32 | Total number of steps |
| **Rotation** | 0–31 | Phase shift (groove offset) |
| **Rate** | Various | Playback speed (1/4, 1/8, 1/16, etc.) |
| **Gate Length** | 0.05–1.0 | How long each note sustains (as fraction of step) |

### Tips

- **Low pulses, many steps** → sparse, syncopated rhythms
- **High gate_length** → overlapping notes (chord effect)
- **Rotation** → shift the pattern without changing density (groove control)
- **Multiple held notes** → each gets its own independent pattern

## Contributing & Development

For build instructions, testing, linting, and pre-commit hooks, see
[CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.

## Specification

For technical details on the Euclidean algorithm, MIDI I/O layout, parameters,
and architecture, see [specification.md](specification.md).

## Roadmap

- **Phase 1** ✅ Bjorklund algorithm + plugin scaffold
- **Phase 2** ✅ MIDI I/O smoke test
- **Phase 3** ✅ Sequencer logic (sample-accurate timing)
- **Phase 4** 🚧 Circular GUI visualization (egui canvas)
- **Phase 5** 📋 Cross-platform testing and optimization
- **Phase 6** 📋 Dual-layer Euclidean (polyrhythmic accents)
