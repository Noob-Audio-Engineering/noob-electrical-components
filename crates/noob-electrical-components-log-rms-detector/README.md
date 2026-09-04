# noob-electrical-components-log-rms-detector

Blackmer's log-domain true-RMS detector: the part that reads the mean of the
square without ever squaring or taking a root.

This crate is the **technique**, not "an RMS detector", and the distinction is
the same one the Blackmer cell crate draws against "VCA". A detector that reads
RMS could be a rectifier into a squarer, a thermal converter, or this: a
bilateral log converter whose two junctions square the signal for free, a
capacitor charged through a junction and discharged by a constant current, and
a square root that is never computed because in the log domain it is a division
by two. Those share a specification, not an equation.

## The law

```text
dL/dt = (D/τ) · ( exp( (L_inst − L) / D ) − 1 )
```

with `L` the stored level in decibels. It is solved exactly over a held sample,
so one step costs one `exp` and one `ln`, the answer does not depend on the
sample rate, and it is unconditionally stable. There is no attack branch and no
release branch **because the circuit has neither**: a rising signal attacks
faster the bigger the step, a falling one decays along a straight line of `D/τ`
decibels per second, and the two are one constant seen from two sides.

## What it knows, and where each number comes from

| Property | Value | Source |
|---|---|---|
| Thermal decibel `D` | `10/ln 10`, exactly | derived; see below |
| Release rate | `D/τ` dB/s, a straight line | the equation's asymptote |
| Attack to 63 % of a step Δ | `τ·ln[(1−e^−u)/(1−e^−0.37u)]`, `u = Δ/D` | the equation, solved |
| Sine against its peak | −3.0103 dB | arithmetic |
| Quarter-duty pulses against their peak | −6.0206 dB | arithmetic |

**Nothing here is a published figure, because nobody publishes this part
separately from the box it sits in.** What replaces a specification is
stronger: the part computes a mean of a square, the mean of a square of a
waveform is arithmetic, and every level in the tests is worked out from the
waveform rather than read back from the model.

## The one constant, and why it is not a measurement

`D` is exactly `10/ln 10`. It is tempting to read it off two datasheet numbers
— a thermal voltage of 25.9 mV over a log constant of 6.1 mV/dB gives 4.246 —
and that is wrong, because the two figures do not correspond: the 6.1 is a
measured typical carrying the junctions' ideality with it and the 25.9 is bare
`kT/q`, so the quotient is 2 % small.

Doing the algebra instead, the ideality and the temperature both cancel,
because the same kind of junction does the logarithm and the averaging, and the
decibel unit is `10/ln 10` whatever they are. That cancellation is why this
crate contains no junction constants at all, and it is most of the reason the
technique is worth having as a component rather than as a filter somebody
tunes. At 4.246 a sine settles 2.98 dB below its peak instead of 3.01, which is
a detector averaging something that is not quite the square.

## What it must not know, and why the line is there

**No attack control, no release control, no threshold, no ratio, no ballistics
of any kind.** That boundary was drawn by a real refusal rather than argued
from first principles.

The dbx 160 has **no attack or release knobs at all**, because its detector
*is* its ballistics: the one time constant its two components set produces its
attack and its release together, and dbx's whole argument for the box is that
you cannot adjust them. The API 2500 has **fourteen** ballistics positions,
because on that unit the panel's ballistics are a separate stage *after* the
detector. Both contain this part. A component carrying an attack control would
be unusable by the first and redundant in the second.

The time constant is a parameter rather than a constant here, and that is the
same line from the other side: the filter cannot run without one, but *which*
one is a capacitor and a current source on somebody's drawing — the dbx's are a
factory-matched pair the drawing marks as one — and those belong to the
machine.

## Who contains one, and on what evidence

- the **dbx 160**, on dbx's own schematic;
- the **API 2500**, reported to be true RMS by reviewers and by API's own copy,
  with no schematic public and nothing below block level from API.

One drawing and one report, and the report was not allowed to shape anything
here. What it contributed is the refusal above, which is a statement about
where the boundary is rather than about what is inside it.

## What is not modelled

The detector's ripple. A real log converter's output carries the excursion at
every zero crossing, which is a real mechanism producing real low-frequency
third harmonic in the units built on it, but how much of it reaches the control
port depends on what the machine does between the two. It emerges from a caller
running this at audio rate rather than being modelled here.

```toml
[dependencies]
noob-electrical-components = { git = "https://github.com/Noob-Audio-Engineering/noob-electrical-components", features = ["log-rms-detector"] }
```
