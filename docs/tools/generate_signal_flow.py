#!/usr/bin/env python3
"""Generate the Signal Flow block diagram for the mdbook manual.

Writes ``docs/src/images/signal-flow.svg``. Run by hand after changing the
signal chain, then commit the regenerated SVG:

    python3 docs/tools/generate_signal_flow.py

Standard library only — this is deliberately not wired into the mdbook build,
so neither the book nor CI gains a dependency.

Geometry is computed rather than hand-placed: modules are declared as data and
coordinates are derived from a handful of metrics below, so blocks in a row
share a y by construction and connectors are drawn from block edges. Adding an
effect means adding one entry to EFFECT_ROWS and re-running.

The chain mirrors ``generate_audio_samples`` in
``crates/accsyn-engine/src/synthesizer/sample_generator.rs``. Note that the
oscillator mixer runs *before* the amplifier and filter.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Literal

BoxKind = Literal["module", "effect", "mod", "source", "group"]

# --------------------------------------------------------------------------
# Palette and type
# --------------------------------------------------------------------------

PAPER = "#ffffff"
INK = "#151a20"
INK_SOFT = "#5b6672"
AUDIO = "#151a20"
MOD = "#7c8a99"
MIDI = "#2f6fb0"
GROUP_FILL = "#eef1f4"
GROUP_EDGE = "#b6c0ca"

# Stroke colour -> arrowhead marker id. This single mapping drives both the
# <marker> definitions and the marker-end lookup, so the two cannot drift.
ARROW_HEADS = {AUDIO: "audio", MOD: "mod", MIDI: "midi"}

FONT = "'Helvetica Neue', Helvetica, Arial, sans-serif"
# Sized so the diagram stays readable when mdbook scales it down to the ~760px
# content column; it is also click-to-zoom on the page.
SIZE_TITLE = 17
SIZE_SUB = 11.5
SIZE_LABEL = 11.5
SIZE_EFFECT = 12.5
SIZE_NUMBER = 10

# --------------------------------------------------------------------------
# Canvas metrics
# --------------------------------------------------------------------------

CANVAS_W = 1400

RAIL_X = 40  # MIDI control rail down the left edge
MAIN_X = 190  # main audio column — gutter holds the MIDI branch labels
MAIN_W = 800
MAIN_R = MAIN_X + MAIN_W

GUTTER_CX = (RAIL_X + MAIN_X) / 2  # centre line for left-rail branch labels

MOD_X = 1040  # modulator column
MOD_W = 230

# Elbow lanes in the gap between the main column and the modulators, spread so
# the three upward modulation routes do not overlap.
LANE_PITCH_ENV = MOD_X - 20
LANE_MOD_WHEEL = MOD_X - 30
LANE_FILTER_LFO = MOD_X - 40

# Vertical bands
Y_MIDI = 44
H_MIDI = 62
Y_MERGE = 132
Y_PITCH = 178
Y_OSC = 206
H_OSC = 112
# Boost is a stage *within* each oscillator, not a shared one, so it is drawn as
# a footer strip inside the oscillator block rather than as its own band.
H_FOOTER = 28
Y_MIXER = 360
Y_AMP = 464
Y_FILTER = 556
H_FILTER = 72
Y_FX = 656
H_BAR = 64
H_DEVICE = 72

# Everything below the effects group is derived from its height — see the
# "Derived layout" block after the content definitions.

# Effects grid
FX_PAD = 18
FX_GAP = 20
FX_BLOCK_H = 54
FX_MIN_BLOCK_W = 92  # below this, effect names stop fitting
FX_ROW_GAP = 42
FX_TITLE_H = 34

OSC_GAP = 22
OSC_SYNC_GAP = 56  # wider gap between Osc 1 and Osc 2 to hold the sync arrow


# --------------------------------------------------------------------------
# Primitives
# --------------------------------------------------------------------------


@dataclass
class Box:
    """A positioned block. Edge helpers keep connectors off hand-typed numbers."""

    x: float
    y: float
    w: float
    h: float
    title: str
    lines: list[str] = field(default_factory=list)
    kind: BoxKind = "module"
    number: str = ""
    # An inset strip along the bottom of the block, divided off by a rule, for a
    # stage that belongs to this module rather than being one of its own.
    footer: str = ""

    @property
    def left(self) -> float:
        return self.x

    @property
    def right(self) -> float:
        return self.x + self.w

    @property
    def top(self) -> float:
        return self.y

    @property
    def bottom(self) -> float:
        return self.y + self.h

    @property
    def cx(self) -> float:
        return self.x + self.w / 2

    @property
    def cy(self) -> float:
        return self.y + self.h / 2


def esc(text: str) -> str:
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def num(value: float) -> str:
    return f"{value:g}"


def text(
    x: float,
    y: float,
    content: str,
    size: float = SIZE_LABEL,
    fill: str = INK,
    anchor: str = "middle",
    weight: str = "normal",
    spacing: float = 0.0,
) -> str:
    extra = f' letter-spacing="{num(spacing)}"' if spacing else ""
    return (
        f'<text x="{num(x)}" y="{num(y)}" font-family="{FONT}" '
        f'font-size="{num(size)}" font-weight="{weight}" fill="{fill}" '
        f'text-anchor="{anchor}"{extra}>{esc(content)}</text>'
    )


def rect(
    box: Box,
    fill: str,
    stroke: str = "none",
    radius: float = 3,
    width: float = 1.4,
) -> str:
    return (
        f'<rect x="{num(box.x)}" y="{num(box.y)}" width="{num(box.w)}" '
        f'height="{num(box.h)}" rx="{num(radius)}" fill="{fill}" '
        f'stroke="{stroke}" stroke-width="{num(width)}"/>'
    )


def path(
    points: list[tuple[float, float]],
    stroke: str = AUDIO,
    width: float = 1.6,
    dashed: bool = False,
    arrow: bool = True,
) -> str:
    d = " ".join(
        ("M" if index == 0 else "L") + f" {num(px)} {num(py)}"
        for index, (px, py) in enumerate(points)
    )
    marker = ""
    if arrow:
        head = ARROW_HEADS.get(stroke)
        if head is None:
            raise ValueError(
                f"No arrowhead marker defined for stroke {stroke!r}. Add it to "
                "ARROW_HEADS (which also emits the <marker> defs), or pass "
                "arrow=False."
            )
        marker = f' marker-end="url(#arrow-{head})"'
    dash = ' stroke-dasharray="5 4"' if dashed else ""
    return (
        f'<path d="{d}" fill="none" stroke="{stroke}" '
        f'stroke-width="{num(width)}" stroke-linejoin="round"{dash}{marker}/>'
    )


def junction(x: float, y: float, fill: str) -> str:
    return f'<circle cx="{num(x)}" cy="{num(y)}" r="3.2" fill="{fill}"/>'


def draw_box(box: Box) -> list[str]:
    """Render a block with its title and subtitle lines, centred."""
    out: list[str] = []
    if box.kind == "effect":
        out.append(rect(box, PAPER, INK, radius=3, width=1.3))
        label_fill, title_size = INK, FX_LABEL_SIZE
    elif box.kind == "mod":
        out.append(rect(box, INK, radius=3))
        label_fill, title_size = PAPER, 12
    elif box.kind == "source":
        out.append(rect(box, PAPER, INK, radius=3, width=1.6))
        label_fill, title_size = INK, 13
    else:
        out.append(rect(box, INK, radius=3))
        label_fill, title_size = PAPER, SIZE_TITLE

    sub_fill = INK_SOFT if box.kind in ("effect", "source") else "#a9b4c0"
    block_h = title_size + len(box.lines) * (SIZE_SUB + 3)
    # A footer strip takes the bottom of the block, so the title and subtitles
    # centre in the space that is left rather than in the whole box.
    content_cy = box.cy - (H_FOOTER / 2 if box.footer else 0)
    cursor = content_cy - block_h / 2 + title_size

    if box.number:
        out.append(
            text(
                box.left + 8,
                box.top + 14,
                box.number,
                size=SIZE_NUMBER,
                fill=INK_SOFT,
                anchor="start",
                weight="bold",
            )
        )

    out.append(
        text(
            box.cx,
            cursor,
            box.title,
            size=title_size,
            fill=label_fill,
            weight="bold",
            spacing=0.4,
        )
    )
    for line in box.lines:
        cursor += SIZE_SUB + 3
        out.append(text(box.cx, cursor, line, size=SIZE_SUB, fill=sub_fill))

    if box.footer:
        rule_y = box.bottom - H_FOOTER
        out.append(
            f'<line x1="{num(box.left + 10)}" y1="{num(rule_y)}" '
            f'x2="{num(box.right - 10)}" y2="{num(rule_y)}" '
            f'stroke="{PAPER}" stroke-width="1" opacity="0.28"/>'
        )
        out.append(
            text(
                box.cx,
                rule_y + H_FOOTER / 2 + 4,
                box.footer,
                size=SIZE_SUB,
                fill=PAPER,
                weight="bold",
                spacing=0.3,
            )
        )
    return out


# --------------------------------------------------------------------------
# Content — edit these to change the diagram
# --------------------------------------------------------------------------

OSCILLATORS = [
    ("SUB OSC", ["One Octave Below", "Wave Shape + Shape Params"]),
    ("OSC 1", ["Sync Source", "Wave Shape + Shape Params"]),
    ("OSC 2", ["Sync Target", "Wave Shape + Shape Params"]),
    ("OSC 3", ["Independent", "Wave Shape + Shape Params"]),
]

# Each oscillator has its own clipper boost stage, driven by its Boost control
# and by aftertouch, applied before the signal reaches the mixer. It is a stage
# inside the oscillator, so it is drawn inside the block — not as a shared band,
# which would wrongly imply one boost across all four.
OSC_FOOTER = "BOOST + AFTERTOUCH"

# Rows mirror the Effects tab in crates/accidental-synth/ui/effects-panel.slint
# (row1 / row2 / row3). That reading order is also the DSP processing order.
EFFECT_ROWS = [
    ["Saturation", "Colour Compressor", "Wave Folder", "Bit Crusher", "Clipper"],
    ["Gate Clipping", "Wave Rectifier", "Chorus", "Flanger", "Auto-Pan"],
    ["Tremolo", "Delay"],
]

# Vertical positions are expressed relative to the band each modulator feeds,
# so they follow if a band moves.
MODULATORS = [
    ("PITCH ENVELOPE", Y_OSC, ["Per-Oscillator Amount"], "GATE"),
    ("MOD WHEEL LFO", Y_OSC + 72, ["Vibrato — CC 1"], "CLOCK"),
    ("AMP ENVELOPE", Y_AMP, ["Attack Decay Sustain Release"], "GATE"),
    ("FILTER ENVELOPE", Y_FILTER - 16, ["Invertible"], "GATE"),
    ("FILTER LFO", Y_FILTER + 56, ["Clock Syncable"], "CLOCK"),
]
H_MOD = 56

# --------------------------------------------------------------------------
# Derived layout
#
# The effects group's height follows from how many rows EFFECT_ROWS declares,
# and the output stage, audio device and canvas height all follow from that.
# Adding an effect — or a whole new row — reflows the diagram instead of
# silently colliding with the blocks underneath.
# --------------------------------------------------------------------------

FX_ROW_COUNT = len(EFFECT_ROWS)

# Horizontal reflow. Block width follows from the widest row, so adding an
# effect to an existing row narrows every block to fit rather than spilling
# past the group border into the modulator column. Rows are NOT auto-wrapped:
# they mirror the Effects tab, so where the break falls is a deliberate choice
# that belongs in EFFECT_ROWS, not in this file's arithmetic.
FX_MAX_COLUMNS = max(len(row) for row in EFFECT_ROWS)
FX_INNER_W = MAIN_W - 2 * FX_PAD - 4
FX_BLOCK_W = (FX_INNER_W - (FX_MAX_COLUMNS - 1) * FX_GAP) / FX_MAX_COLUMNS

if FX_BLOCK_W < FX_MIN_BLOCK_W:
    raise SystemExit(
        f"Effect blocks would be {FX_BLOCK_W:.0f}px wide with "
        f"{FX_MAX_COLUMNS} in a row, below the {FX_MIN_BLOCK_W}px minimum. "
        "Split EFFECT_ROWS across more rows to match the Effects tab, or "
        "widen MAIN_W and CANVAS_W together."
    )

# Effect labels are centred in a block, so a narrower grid needs smaller type or
# the longest name bleeds past its border. 0.58em is a serviceable average glyph
# width for bold Helvetica, but it is an estimate with no font metrics behind
# it, and it is measured on the label with the most *characters* rather than the
# widest glyphs. FX_LABEL_PAD is therefore deliberately generous: it absorbs a
# few percent of error in either assumption, so an underestimate shows up as a
# slightly small label rather than one bleeding over the block border.
FX_LABEL_PAD = 20
FX_LONGEST_LABEL = max(len(name) for row in EFFECT_ROWS for name in row)
FX_LABEL_SIZE = min(
    SIZE_EFFECT, (FX_BLOCK_W - FX_LABEL_PAD) / (0.58 * FX_LONGEST_LABEL)
)

FX_H = (
    FX_TITLE_H
    + FX_ROW_COUNT * FX_BLOCK_H
    + (FX_ROW_COUNT - 1) * FX_ROW_GAP
    + FX_PAD
)
Y_OUTPUT = Y_FX + FX_H + 46
Y_DEVICE = Y_OUTPUT + H_BAR + 28
CANVAS_H = Y_DEVICE + H_DEVICE + 44


def build() -> str:
    parts: list[str] = []

    # ---- defs -------------------------------------------------------------
    markers = "".join(
        f'<marker id="arrow-{name}" viewBox="0 0 10 10" refX="9" refY="5" '
        f'markerWidth="6" markerHeight="6" orient="auto-start-reverse">'
        f'<path d="M 0 0 L 10 5 L 0 10 z" fill="{colour}"/></marker>'
        for colour, name in ARROW_HEADS.items()
    )
    parts.append(f"<defs>{markers}</defs>")
    parts.append(
        f'<rect x="0" y="0" width="{CANVAS_W}" height="{CANVAS_H}" fill="{PAPER}"/>'
    )

    # ---- MIDI sources -----------------------------------------------------
    src_w = (MAIN_W - 40) / 2
    virtual = Box(
        MAIN_X,
        Y_MIDI,
        src_w,
        H_MIDI,
        "VIRTUAL MIDI PORT",
        ['"AccSyn MIDI Input" — always available'],
        kind="source",
    )
    device = Box(
        MAIN_X + src_w + 40,
        Y_MIDI,
        src_w,
        H_MIDI,
        "MIDI INPUT DEVICE",
        ["Selectable · Channel Filter"],
        kind="source",
    )
    for box in (virtual, device):
        parts.extend(draw_box(box))
        parts.append(path([(box.cx, box.bottom), (box.cx, Y_MERGE)], MIDI, arrow=False))

    parts.append(path([(virtual.cx, Y_MERGE), (device.cx, Y_MERGE)], MIDI, arrow=False))
    parts.append(junction(virtual.cx, Y_MERGE, MIDI))
    parts.append(junction(device.cx, Y_MERGE, MIDI))
    parts.append(
        text(
            (virtual.cx + device.cx) / 2,
            # Below the merge bar: above it the label would cross the two drop
            # lines coming down from the source blocks.
            Y_MERGE + 18,
            "Note · Velocity · Pitch Bend · Mod Wheel · Aftertouch · CC · Program Change · Clock",
            size=SIZE_LABEL,
            fill=INK_SOFT,
        )
    )

    # ---- MIDI rail down the left ------------------------------------------
    y_velocity = Y_AMP + H_BAR / 2
    y_keytrack = Y_FILTER + H_FILTER / 2
    # Level of the boost strips, so the aftertouch feed lands on them.
    y_aftertouch = Y_OSC + H_OSC - H_FOOTER / 2
    parts.append(
        path(
            [(virtual.cx, Y_MERGE), (RAIL_X, Y_MERGE), (RAIL_X, y_keytrack)],
            MIDI,
            arrow=False,
        )
    )
    for y_branch in (Y_PITCH, y_aftertouch, y_velocity):
        parts.append(junction(RAIL_X, y_branch, MIDI))

    # ---- Oscillators ------------------------------------------------------
    osc_w = (MAIN_W - (2 * OSC_GAP + OSC_SYNC_GAP)) / 4
    gaps = [OSC_GAP, OSC_SYNC_GAP, OSC_GAP]
    oscillators: list[Box] = []
    cursor_x = MAIN_X
    for index, (title, lines) in enumerate(OSCILLATORS):
        oscillators.append(
            Box(cursor_x, Y_OSC, osc_w, H_OSC, title, lines, footer=OSC_FOOTER)
        )
        if index < len(gaps):
            cursor_x += osc_w + gaps[index]

    # Pitch rail: MIDI note pitch, joined by pitch envelope and vibrato
    parts.append(
        path([(RAIL_X, Y_PITCH), (LANE_PITCH_ENV, Y_PITCH)], MIDI, arrow=False)
    )
    parts.append(
        text(
            oscillators[0].cx - 6,
            Y_PITCH - 8,
            "PITCH  ·  Tune · Portamento · Pitch Bend · Key Sync",
            size=SIZE_LABEL,
            fill=MIDI,
            anchor="start",
        )
    )
    for box in oscillators:
        parts.append(junction(box.cx, Y_PITCH, MIDI))
        parts.append(path([(box.cx, Y_PITCH), (box.cx, box.top)], MIDI))
        parts.extend(draw_box(box))

    # Hard sync, drawn in the widened gap between Osc 1 and Osc 2
    sync_y = oscillators[1].cy
    parts.append(
        path(
            [(oscillators[1].right, sync_y), (oscillators[2].left, sync_y)],
            AUDIO,
            width=1.4,
        )
    )
    parts.append(
        text(
            (oscillators[1].right + oscillators[2].left) / 2,
            sync_y - 9,
            "SYNC",
            size=SIZE_NUMBER,
            fill=INK,
            weight="bold",
        )
    )

    # ---- Aftertouch into the boost strips ----------------------------------
    # Aftertouch drives each oscillator's boost, which is unusual enough that a
    # musician would not guess it — so it is drawn, not just captioned. It runs
    # in at boost level and hops the gaps between the blocks, reading as one
    # signal feeding all four.
    parts.append(
        path([(RAIL_X, y_aftertouch), (oscillators[0].left, y_aftertouch)], MIDI)
    )
    parts.append(
        text(GUTTER_CX, y_aftertouch - 8, "AFTERTOUCH", size=SIZE_LABEL, fill=MIDI)
    )
    for left, right in zip(oscillators, oscillators[1:]):
        parts.append(
            path([(left.right, y_aftertouch), (right.left, y_aftertouch)], MIDI)
        )

    # ---- Main chain bars ---------------------------------------------------
    mixer = Box(
        MAIN_X,
        Y_MIXER,
        MAIN_W,
        H_BAR,
        "OSCILLATOR MIXER",
        ["Level · Balance · Mute per Oscillator"],
    )
    amp = Box(
        MAIN_X, Y_AMP, MAIN_W, H_BAR, "AMPLIFIER", ["Velocity × Amp Envelope"]
    )
    filt = Box(
        MAIN_X,
        Y_FILTER,
        MAIN_W,
        H_FILTER,
        "FILTER",
        ["Resonant Ladder Lowpass · 1–4 Poles", "Cutoff · Resonance · Key Tracking"],
    )

    for box in oscillators:
        parts.append(path([(box.cx, box.bottom), (box.cx, mixer.top)], AUDIO))
    parts.extend(draw_box(mixer))
    parts.append(path([(mixer.cx, mixer.bottom), (mixer.cx, amp.top)], AUDIO))
    parts.extend(draw_box(amp))
    parts.append(path([(amp.cx, amp.bottom), (amp.cx, filt.top)], AUDIO))
    parts.extend(draw_box(filt))

    parts.append(path([(RAIL_X, y_velocity), (amp.left, y_velocity)], MIDI))
    parts.append(
        text(GUTTER_CX, y_velocity - 8, "VELOCITY", size=SIZE_LABEL, fill=MIDI)
    )
    parts.append(path([(RAIL_X, y_keytrack), (filt.left, y_keytrack)], MIDI))
    parts.append(
        text(GUTTER_CX, y_keytrack - 8, "NOTE / KEY TRACK", size=SIZE_LABEL, fill=MIDI)
    )

    # ---- Effects ----------------------------------------------------------
    group = Box(MAIN_X, Y_FX, MAIN_W, FX_H, "", kind="group")
    parts.append(rect(group, GROUP_FILL, GROUP_EDGE, radius=6, width=1.2))
    parts.append(
        text(
            group.left + FX_PAD,
            group.top + 22,
            "EFFECTS  —  processed in order, each bypassable",
            size=12,
            fill=INK,
            anchor="start",
            weight="bold",
        )
    )
    parts.append(path([(filt.cx, filt.bottom), (filt.cx, group.top)], AUDIO))

    row_x = group.left + FX_PAD + 2
    rows: list[list[Box]] = []
    counter = 0
    for row_index, names in enumerate(EFFECT_ROWS):
        row_y = group.top + FX_TITLE_H + row_index * (FX_BLOCK_H + FX_ROW_GAP)
        row: list[Box] = []
        for column, name in enumerate(names):
            counter += 1
            row.append(
                Box(
                    row_x + column * (FX_BLOCK_W + FX_GAP),
                    row_y,
                    FX_BLOCK_W,
                    FX_BLOCK_H,
                    name,
                    kind="effect",
                    number=str(counter),
                )
            )
        rows.append(row)

    # Within-row connectors
    for row in rows:
        for left, right in zip(row, row[1:]):
            parts.append(path([(left.right, left.cy), (right.left, right.cy)], AUDIO))

    # Wrap connectors: end of a row sweeps back to the start of the next,
    # so every row reads left-to-right exactly like the Effects tab.
    sweep_right = group.right - 6
    sweep_left = group.left + 9
    for upper, lower in zip(rows, rows[1:]):
        last, first = upper[-1], lower[0]
        gap_y = (last.bottom + first.top) / 2
        parts.append(
            path(
                [
                    (last.right, last.cy),
                    (sweep_right, last.cy),
                    (sweep_right, gap_y),
                    (sweep_left, gap_y),
                    (sweep_left, first.cy),
                    (first.left, first.cy),
                ],
                AUDIO,
            )
        )

    for row in rows:
        for box in row:
            parts.extend(draw_box(box))

    # ---- Output -----------------------------------------------------------
    output = Box(
        MAIN_X,
        Y_OUTPUT,
        MAIN_W,
        H_BAR,
        # Paired with OSCILLATOR MIXER: both are the mixer module (quad_mix and
        # output_mix), and the UI groups them as Mixer > Per-Oscillator and
        # Mixer > Output. Naming this one just "Output" hid that pairing.
        "OUTPUT MIXER",
        ["Master Level · Balance · Mute · Polarity"],
    )
    # Leave from under the last effect, then square up to the centre so the
    # output stage is entered head-on rather than off to one side.
    last_effect = rows[-1][-1]
    turn_y = (group.bottom + output.top) / 2
    parts.append(
        path(
            [
                (last_effect.cx, group.bottom),
                (last_effect.cx, turn_y),
                (output.cx, turn_y),
                (output.cx, output.top),
            ],
            AUDIO,
        )
    )
    parts.extend(draw_box(output))

    device_box = Box(
        MAIN_X,
        Y_DEVICE,
        MAIN_W,
        H_DEVICE,
        "AUDIO OUTPUT DEVICE",
        ["Core Audio · Selected Device"],
        kind="source",
    )
    for offset, channel in ((-120, "LEFT"), (120, "RIGHT")):
        x_channel = output.cx + offset
        parts.append(
            path(
                [(x_channel, output.bottom), (x_channel, device_box.top)],
                AUDIO,
            )
        )
        parts.append(
            text(
                x_channel + 8,
                (output.bottom + device_box.top) / 2 + 4,
                channel,
                size=SIZE_LABEL,
                fill=INK_SOFT,
                anchor="start",
            )
        )
    parts.extend(draw_box(device_box))

    # ---- Modulator column --------------------------------------------------
    targets = {
        "PITCH ENVELOPE": (LANE_PITCH_ENV, Y_PITCH),
        "MOD WHEEL LFO": (LANE_MOD_WHEEL, Y_PITCH),
        "AMP ENVELOPE": (amp.right, amp.cy),
        "FILTER ENVELOPE": (filt.right, filt.top + 22),
        "FILTER LFO": (filt.right, filt.bottom - 22),
    }
    for title, mod_y, lines, stub in MODULATORS:
        box = Box(MOD_X, mod_y, MOD_W, H_MOD, title, lines, kind="mod")
        parts.extend(draw_box(box))

        # Short stub arrow for the gate/clock input, minilogue-style, rather
        # than routing these all the way back to the MIDI rail.
        parts.append(
            path([(box.right + 30, box.cy), (box.right, box.cy)], MIDI, width=1.4)
        )
        parts.append(
            text(
                box.right + 34,
                box.cy - 7,
                stub,
                size=SIZE_NUMBER,
                fill=MIDI,
                anchor="start",
            )
        )

        target_x, target_y = targets[title]
        if title in ("PITCH ENVELOPE", "MOD WHEEL LFO"):
            points = [
                (box.left, box.cy),
                (target_x, box.cy),
                (target_x, target_y),
            ]
            parts.append(path(points, MOD, dashed=True))
        elif title == "FILTER LFO":
            elbow_x = LANE_FILTER_LFO
            points = [
                (box.left, box.cy),
                (elbow_x, box.cy),
                (elbow_x, target_y),
                (target_x, target_y),
            ]
            parts.append(path(points, MOD, dashed=True))
        else:
            parts.append(path([(box.left, box.cy), (target_x, target_y)], MOD, dashed=True))

    # ---- Legend ------------------------------------------------------------
    legend_y = Y_FX + 60
    legend = Box(MOD_X, legend_y, MOD_W, 106, "", kind="group")
    parts.append(rect(legend, PAPER, GROUP_EDGE, radius=6, width=1.2))
    parts.append(
        text(
            legend.left + 16,
            legend.top + 24,
            "SIGNAL KEY",
            size=11,
            fill=INK,
            anchor="start",
            weight="bold",
            spacing=0.6,
        )
    )
    entries = [
        (AUDIO, False, "Audio"),
        (MOD, True, "Modulation"),
        (MIDI, False, "MIDI / Note Control"),
    ]
    for index, (colour, dashed, label) in enumerate(entries):
        entry_y = legend.top + 46 + index * 20
        parts.append(
            path(
                [(legend.left + 16, entry_y), (legend.left + 56, entry_y)],
                colour,
                dashed=dashed,
                arrow=False,
            )
        )
        parts.append(
            text(
                legend.left + 66,
                entry_y + 4,
                label,
                size=SIZE_LABEL,
                fill=INK,
                anchor="start",
            )
        )

    body = "\n".join(parts)
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {CANVAS_W} {CANVAS_H}" '
        f'width="{CANVAS_W}" height="{CANVAS_H}" role="img" '
        f'aria-label="Accidental Synthesizer signal flow block diagram">\n'
        f"{body}\n</svg>\n"
    )


# The manual links the SVG directly so it can be opened full size, and a
# top-level .svg renders as a document where scripts execute — unlike the inline
# <img> embed, which sandboxes them. Every label is currently a literal in this
# file, so nothing can inject; this guard exists so that stays true if the
# labels are ever sourced from a patch file or config.
UNSAFE_SVG = re.compile(r"<script|<foreignObject|\son[a-z]+\s*=", re.IGNORECASE)


def assert_no_script_vectors(svg: str) -> None:
    match = UNSAFE_SVG.search(svg)
    if match:
        raise SystemExit(
            f"Refusing to write: generated SVG contains {match.group(0)!r}. "
            "The manual links this file directly, so scripting constructs are "
            "live when a reader opens it full size. Route all text through "
            "text()/esc() rather than emitting raw markup."
        )


def main() -> None:
    destination = (
        Path(__file__).resolve().parents[1] / "src" / "images" / "signal-flow.svg"
    )
    svg = build()
    assert_no_script_vectors(svg)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(svg, encoding="utf-8")
    print(f"Wrote {destination}")


if __name__ == "__main__":
    main()
