# Signal Flow

How a note travels through AccSyn, from MIDI in to audio out. AccSyn is
monophonic, so there is a single signal path — no voice allocation. Note that
the oscillator mixer comes *before* the amplifier and filter.

Click the diagram to open it full size.

[![Block diagram of the Accidental Synthesizer signal path, from the virtual and
hardware MIDI inputs, through the four oscillators, per-oscillator boost,
oscillator mixer, amplifier and filter, then the twelve effects in order, to the
output stage and audio device.](images/signal-flow.svg)](images/signal-flow.svg)

The effects are laid out in the same three rows as the [Effects
tab](./controls.md#effects-tab), and they process in that order — left to right,
top to bottom. For what each control does, see [Controls](./controls.md).
