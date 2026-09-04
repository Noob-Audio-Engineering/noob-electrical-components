# noob-electrical-components-blackmer-cell

David Blackmer's log-antilog gain cell: the part that makes a VCA compressor
sound like one.

This crate is the **cell**, not "a VCA", and the distinction is why it exists.
"VCA" is a functional category covering Blackmer's log-antilog cell, the
operational transconductance amplifier and a field-effect transistor used as a
variable resistor. Those share a word, not an equation. An audit of the
plug-ins built on this repository found three "tube stages" that turned out to
be three different circuits wearing one name, and a component called `Vca`
would have been the fourth.

## The law

From US 3,714,462: an input amplifier with two feedback paths through the
collector-emitter circuits of opposite-conductivity transistors turns a signal
current into a log voltage for both halves of the waveform without rectifying
it; a second pair takes the antilogarithm; and the control voltage is summed
with the log signal at the bases. Adding a voltage in the log domain multiplies
in the linear domain, so gain is exponential in the control voltage and exactly
so. Blackmer's own stated objective was "a constant decibels per volt control
characteristic".

`gain_db` therefore takes **millivolts**, not decibels. The reason this is a
component rather than a multiply is the 6.1 mV/dB constant, its tolerance and
its temperature coefficient, and a caller passing decibels has thrown all three
away.

## What it knows, and where each number comes from

| Property | Value | Source |
|---|---|---|
| Control law | gain in dB is the port difference over the constant | Blackmer, US 3,714,462 |
| Control constant | 6.1 mV/dB typical, 6.0 to 6.2 across grades | THAT 2180 datasheet |
| Temperature coefficient | +0.33 %/°C, referenced to a 27 °C chip | THAT 2180 datasheet |
| Control-law linearity | 0.5 % typical, 2 % maximum over 100 dB | THAT 2180 datasheet |
| Gain at zero control | 0.0 dB | THAT 2180 datasheet |
| Off isolation | 110 dB minimum, 115 dB typical | THAT 2180 datasheet |
| Symmetry window, untrimmed | ±1.6 mV, A grade | THAT 2150 datasheet |
| Distortion, untrimmed at unity | 0.005 / 0.010 / 0.030 % by grade | THAT 2180 datasheet |

The temperature coefficient is the one figure that checks itself. A junction's
thermal voltage is proportional to absolute temperature, so a control law built
from junctions should scale the same way, and one over the reference in kelvin
is 0.3332 %/°C. The published 0.33 %/°C is a two-figure rounding of that. The
datasheet and the derivation agree and neither was fitted to the other.

## What it must not know

The resistor that converts a voltage into the cell's input current, the
current-to-voltage converter after it, the detector, the threshold, the ratio,
the make-up gain, the meter. Those are the machine. They differ completely
between the two units documented to contain this cell, and they belong in the
plug-in. This is the same line the photocell crate draws when it knows its own
dark and lit resistances and nothing about the 70.7 kΩ series resistor an LA-2A
puts in front of it.

## Who is documented to contain one

Two units, on two manufacturers' own drawings, which is the standard the
photocell met rather than the weaker one the diode bridge met:

- the **dbx 160**, whose schematic calls out a plug-in module lettered
  `VCA (200)`, reference designator M1;
- the **SSL 4000 G bus compressor**, whose card 82E26 has `DBX 202C` lettered
  on it by SSL, at both the audio and the sidechain positions.

A third unit joins them on weaker evidence. The **API 2500**'s cell is
reported to be a THAT 2180, the monolithic descendant of the same design,
but that comes from a reviewer identifying chips in 2001 rather than from a
manufacturer's drawing. It corroborates the part without being allowed to
shape it, and it needed nothing this crate does not already hold, which is a
small piece of evidence that the boundary is drawn in the right place.

A fourth unit, the **Distressor**, is usually described as using a
Blackmer-style cell, but that is an inference from how it behaves rather than a
part read off a drawing, and the model of it in the plug-in stands one
distortion constant in for the whole cell. It is free to consume this crate. It
did not shape it. That asymmetry of evidence is recorded deliberately, because
a component shaped by a unit nobody has a schematic for is how the tube-stage
mistake happened.

The two units that do contain the cell share it and share **nothing** about how
they decide what to do with it. The dbx uses a true-RMS detector working in the
log domain, whose attack and release are one locked pair; the SSL uses a
precision rectifier into a passive network with two independent time constants.
That is the argument for this boundary rather than a wider one: the family
resemblance people hear between the two really does come from the cell, and the
large difference in how they behave on programme really does come from the
detector, which is why no detector is modelled here.

## The shape of the residual is not settled, and this crate says so

The magnitude of the even-order residual is published several times over. Its
**shape** is published nowhere, and the two units documented to contain the
cell are modelled with two different ones, so `EvenResidual` carries both and
the caller chooses.

- **`HalfPathMismatch`,** `y = x + ε·|x|`, is the mechanism the symmetry trim
  pin exists to null: the two halves of the waveform go through
  opposite-conductivity transistors, and if those paths are not matched the two
  halves are not amplified identically. Its relative second harmonic does not
  depend on level.
- **`Squarer`,** `y = x + ε·x²`, is a smooth even curvature — not what a gain
  mismatch between two paths produces, but what a curvature common to both
  does. Its relative second harmonic is proportional to level.

What pulls each way is set out in full at `EvenResidual`, and neither argument
wins. THAT's table publishes the two conditions away from unity gain with the
*same* distortion, 0.020 %, although one raises the input 10 dB and the other
lowers it 5 dB, which no residual proportional to input level can produce; and
a level-independent residual is flat where that table clearly is not. So the
crate holds both rather than picking one, which is what this repository's
remote-cutoff triode entry was written to insist on.

There is a consequence a caller should know before choosing. The squarer is
exactly second order, so its output bandwidth is exactly twice its input
bandwidth and two-times oversampling contains it with nothing left to fold. The
mismatch shape has a corner at the origin, its spectrum does not stop, and no
oversampling ratio contains it.

## The distortion is in the caller's units, and it says which

`thd_unity` is the published second-harmonic ratio and `thd_peak` is the peak
amplitude, in the caller's own units, at which that figure was measured. That
second field exists because neither documented user works in volts. One is a
compressor whose residual is level independent and has no reference level at
all; the other works in sample amplitude and carries its own volts-per-sample
calibration, which belongs to that console and not to this part. An interface
that took volts and nothing else would have forced a multiply into volts and
back on every sample for one of them and meant nothing for the other.

`DBV_PEAK_VOLTS` is the answer for a caller working in volts, and it is the
default, so nothing is silently assumed about a caller who never thinks about
it.

## The direct-current term, and where the coupling goes

Both shapes have a non-zero mean, so `process` emits a small offset exactly as
the part does. It is not removed here, because the capacitor that removes it in
hardware is downstream of the cell and because a high-pass filter is
infrastructure rather than a component. What is here is `process_coupled`, the
seam a caller subtracts its own running mean of the residual at. Both real
users needed that seam, at different corner frequencies and with different
filters, and neither wanted the filter — which is the boundary landing where it
should.

## What is estimated, and what is missing

Two things, both flagged in the code and in the tests:

- **The shape of the linearity bow.** The magnitude is published, 0.5 %
  typical over a 100 dB span, but no datasheet says how that error is
  distributed within the span. The half-cycle sine used here is an estimate.
  The tests assert the published bound and never the curve, and the default
  cell has the bow switched off so a caller who wants no invention gets none.
- **Distortion does not vary with gain.** THAT publish three conditions, and
  the two away from unity gain are three times worse than the one at it. Three
  points do not establish a surface, and fitting one through them would replace
  a published measurement with an invention, so only the unity-gain figure is
  modelled and the full table is carried in the code for whoever closes the
  gap. A caller running the cell at high gain reduction gets less distortion
  than the part really produces.
