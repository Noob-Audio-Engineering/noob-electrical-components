# noob-electrical-components-transformer

The audio transformer's low end: the roll-off its magnetising inductance
puts under the band, and the flux its core can carry.

A transformer works by not storing the signal. Current in the primary
magnetises the core, the changing flux induces a voltage in the secondary,
and the design goal is for as little of the primary current as possible to
go into the magnetisation itself. Both behaviours here are that arrangement
failing at the bottom of the band, in the two different ways it can:

- **It runs out of reactance.** The magnetising inductance sits across the
  source and its impedance falls with frequency, so below some corner it
  shunts the signal instead of transforming it. That is `Rolloff`, and it is
  linear — it happens at every level.
- **It runs out of core.** Flux is the integral of the applied voltage, so a
  low note puts far more flux through the core than a high one at the same
  level, and past some amount the core cannot carry any more. That is
  `Core`, and it is a nonlinearity — it happens only when the signal is both
  loud and low.

The second is why transformer distortion is a low-frequency phenomenon, and
why a model that ties it to level alone has it half right at best.

## What is here and what is not

Here: a corner, a Q, a flux limit, and the law that limit obeys.

Not here: which corner a given revision uses, and the filters built from
them. **This crate designs no filter and holds no state.** A `Rolloff` is a
description of an analogue response; the sample rate it has to be realised
at, the topology that survives a 6 Hz corner at 192 kHz, and the denormal
handling belong to the caller, because they are properties of the arithmetic
rather than of the part. Filters are infrastructure in this repository, and
that line is what stops a component crate turning into a DSP library.

Also not here: **the top end.** A real transformer rolls off up there too,
from leakage inductance and winding capacitance, and one of the two units
this part was drawn from models it. It was left out because only one of them
does, so there is no second implementation to reconcile a shape against, and
because the numbers that one uses were fitted to the response of a whole
chain — resamplers and anti-aliasing included — rather than read off a
transformer. They are a machine's calibration wearing a part's name.

The core is here on one user, which looks like the same situation and is
not. Its law is not a corner somebody tuned; it is the standard
flux-saturation approximation and it is unambiguously this part. Leaving it
outside would also have had a transformer's core borrowing a *valve's*
saturating curve from `noob-electrical-components-small-signal-triode`,
because the two happen to share an algebraic family, and a wrong dependency
is worse than a part drawn from one user.

## What is estimated

The roll-off's form is exact: a magnetising inductance across a source is a
single pole, and a pole pair is what a blocking network adds to it. **No
corner ships here**, because a corner is a property of one wound part and
every unit's is different.

The core's law is an approximation with a name — integrate, saturate,
subtract — and it is **not** a hysteresis model. No minor loops, no
remanence, no coercivity, no eddy-current or hysteresis losses, so it cannot
show the memory a real core has: drive it hard and stop, and it forgets
immediately. What it gets right is the part that is audible in a preamp,
that saturation follows flux and so arrives at low frequencies first.
Anything needing the memory wants Jiles-Atherton or a wave-digital core.

The knee exponent is this crate's own number, with no source behind it, and
says so at its definition.

## Sources

- Paiva, Pakarinen, Välimäki and Tikander, "Real-Time Audio Transformer
  Emulation for Virtual Tube Amplifiers", EURASIP JASP 2011, for the
  gyrator-capacitor treatment and the measurement that these distort at low
  frequencies only.
- Holters and Zölzer, DAFx-16, for the hysteresis model this deliberately is
  not.

```toml
[dependencies]
noob-electrical-components = { git = "https://github.com/Noob-Audio-Engineering/noob-electrical-components", features = ["transformer"] }
```
