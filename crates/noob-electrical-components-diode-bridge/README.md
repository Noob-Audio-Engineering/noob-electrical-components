# noob-electrical-components-diode-bridge

The balanced diode bridge used as a gain element: four matched diodes wired
as two pairs between two signal rails, with a DC control current entering
one floating common node and leaving the other. Forward-biasing the diodes
sets how much signal current the bridge passes, so the control current sets
a resistance and whatever divider the machine puts around it sets the gain.

Neve used exactly this as the attenuator of the 2254 and the 33609. It also
turns up in the Siemens U273 and the EMI TG12413.

## Why the law is a hyperbolic tangent

A single diode shunting a resistor gives an equation that is implicit in
the output voltage, and solving it needs the Lambert W function, or the
Wright omega recasting of it.

A bridge is not that circuit. Both common nodes float, so each pair is a
current divider steered by the differential voltage with the control
current as its tail. That is structurally a long-tailed pair, and its
characteristic is

```text
i(u) = I · tanh( u / (2·η·V_T) )
```

with no implicit resistive term left to solve. Three things follow, and
they are the reasons to model the part rather than multiply by a gain:

- **It is odd**, so the bridge makes no even harmonics at all. Any even
  order in a real unit comes from the transformers, the amplifier or a
  mismatch between the four diodes, which is why Neve specified matched
  pairs.
- **The small-signal resistance is `r = k / I`**, in closed form and
  trivially invertible, so there is no gain parameterisation to solve
  numerically.
- **Distortion falls as gain reduction rises**, because more control
  current means less resistance, less voltage across the bridge, and a
  smaller tanh argument. A model that ties bridge distortion to the amount
  of gain reduction has it backwards.

## What is here and what is not

The bridge alone: the current for a given differential voltage and control
current, that law's slope, its antiderivative for antialiasing, and the
resistance it implies.

Not here: the series and shunt resistors that turn it into an attenuator,
the sidechain that produces the control current, the shaping network
between them, or the transformers on either side. Those are the machine,
and they differ between the 2254, the 33609 and the TG12413 while the
bridge does not. Solving the node equation of a particular divider is the
caller's job, which is what the slope is exported for.

## What is estimated

The topology and the law are derived and exact for ideal matched diodes.
The numbers are not measured. The ideality factor and saturation current
come from Pines' fit to a 1N4148; these bridges use 1N4153-class parts, and
no reachable 1N4153 datasheet publishes either figure. They enter only
through the thermal scale, which is a single calibratable constant rather
than a structural assumption, so a machine with a level annotation to fit
against should fit against it.

Not modelled: temperature, which the law is proportional to and which moves
it about 7 % over 20 °C; junction capacitance, which rises with forward
bias and lifts the top end as gain falls; and reverse recovery.

## What the tests can and cannot assert

No published measurement of one of these bridges exists, and no vendor
publishes a harmonic spectrum for the units built on them. So the tests
assert the derived properties of the law — odd symmetry, saturation at the
control current, the resistance relation, the third-harmonic expansion —
and say in each case that the reference is a derivation rather than a
measurement.

The odd symmetry is the one worth trusting most, because it was reached
twice independently: from the topology here, and by Pines for a symmetric
bridge, concluding that "only odd harmonics are present".

One correction the tests record: the Neve dossier's section 4.5 gives the
third-harmonic ratio of `tanh(a·sinθ)` as `a²/24`. It is `a²/12`. The
algebra, this crate's DFT and an independent check all agree, and the
consequence is that the bridge's own distortion at the annotated drive
level is about twice what that section states.

## Sources

- AMS Neve, *33609/J Limiter Compressor Technical Handbook*: the D14–D17
  bridge and the resistors around it.
- Neve, drawing D/10,022/A, the 2254's B185 card: D1–D4, HBX 31, with four
  330 pF compensation capacitors.
- C. V. Pines, "Real-Time Virtual Analog Modelling of Diode-Based VCAs",
  DAFx-25, Ancona 2025, pages 63–70: the diode parameters, the
  odd-symmetry result reached independently for a symmetric bridge, and
  the recommendation to block DC either side of a diode gain element.
