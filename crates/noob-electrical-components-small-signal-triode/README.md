# noob-electrical-components-small-signal-triode

The ordinary preamp valve: half a 12AX7-class double triode in a class-A
common-cathode stage, and the saturating law it obeys. Two of these with
variable negative feedback between them are the whole audio path of a
Universal Audio 610.

## This is not the remote-cutoff triode

The other valve in this repository is
`noob-electrical-components-remote-cutoff-triode`, the gain element of the
variable-mu family. **They are two components and neither can serve for the
other**, and the reason is not that they want different numbers.

A remote-cutoff valve is wound with a varying grid pitch, so different parts
of its grid stop conducting at different bias voltages and its
transconductance falls away in a long shallow tail. A variable-mu compressor
gets all of its gain reduction by moving that bias. This valve has no tail:
its bias is a voicing control and not a gain control, and the law is
normalised so that the bias **cannot** reach the gain — moving it changes the
asymmetry of the curve and nothing else. There is no bias at which this stage
is 20 dB down, so a control voltage applied to it would have nothing to do.

That is a difference of functional form rather than of parameters, so no
refit closes it. A test here measures it: the whole bias range moves the
small-signal gain by under 0.01 dB, where a remote-cutoff valve's whole
range is bias-controlled gain.

Nor can it be settled numerically, by fitting an exponent to each valve and
comparing. The other crate records why: fitting one valve's transconductance
across the four operating conditions its own maker plots gives 1.00, 0.84,
0.71 and 0.59, every fit good to under half a decibel. One valve, one page,
a factor of 1.7. An exponent read off a datasheet is not a property of a
valve, so it was never a quantity two valves could be compared on.

The workspace README used to claim that a variable-mu unit would want the
610's tube stage. It was wrong, it has been corrected there, and this is the
correction restated where somebody reaching for the wrong crate would read
it.

## The law

```text
S(v) = v / (1 + |v|^n)^(1/n)          Yeh, Abel and Smith's tanh-like family
T(v) = ( S(v + b) − S(b) ) / S'(b)    the stage, biased at b and normalised
```

The stage sits at a bias point up the bend rather than at the curve's
inflection, which is what makes a single-ended triode asymmetric. Referring
the curve back to zero means silence in gives silence out; dividing by the
slope there means the bias sets the colour without moving the level, which
matters to any machine that walks the bias with a sagging supply. The
exponent `n` sets how abruptly the stage closes onto its asymptote, not
where.

## What is here and what is not

Here: the curve, and the two numbers that fix it.

Not here: the amplitude a machine drives the curve at, the feedback a gain
switch trades against attenuation, the supply sag that walks the bias, the
table that picks one revision's bias over another's, the oversampling, and
every filter around it. **No bias or knee value ships in this crate** — a
voicing's numbers are the machine's.

Also not here: the antiderivative. `S` has an elementary one only for
integer `n`, and real voicings do not use integers, so a machine applying
first-order antiderivative anti-aliasing has to tabulate it. That table is a
technique rather than a part. This is the opposite decision from the diode
bridge, whose antiderivative is a closed form of its law and does live in its
crate; here there is no closed form, and `s_curve` and `s_slope` are public
precisely so a machine can integrate the law without keeping a second copy
of it.

## What is estimated

All of it, in the sense that matters. This is a shape fitted to published
behaviour, not a device equation: no plate voltage, no amplification factor,
no load line, no plate current. It is not Koren's law, nor Dempwolf and
Zölzer's, nor a Child-Langmuir fit, and it will not become one by having
parameters added to it. Anything that needs a plate curve needs a different
model.

What it reproduces is the character the sources agree on for a single-ended
triode gain stage: a decaying harmonic series dominated by the second,
distortion that grows in proportion to level, and a knee rather than a clip.
The tests assert those three and nothing stronger, because no independent
measurement of one of these stages is published.

## Sources

- Yeh, Abel and Smith, "Simplified, Physically-Informed Models of Distortion
  and Overdrive Guitar Effects Pedals", DAFx-07, Bordeaux, for the family.
- Blencowe, "Designing Valve Preamps for Guitar and Bass", chapter 1, for the
  common gain stage: unequally spaced grid curves, the second-harmonic
  dominance, distortion proportional to level, and self-rectification.

```toml
[dependencies]
noob-electrical-components = { git = "https://github.com/Noob-Audio-Engineering/noob-electrical-components", features = ["small-signal-triode"] }
```
