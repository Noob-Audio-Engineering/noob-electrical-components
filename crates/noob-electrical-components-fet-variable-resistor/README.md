# noob-electrical-components-fet-variable-resistor

A junction field-effect transistor used as a voltage-controlled variable
resistor: the gain element of the 1176, and the reason a FET compressor
sounds like one.

Biased into its ohmic region a JFET is a resistor whose value the gate sets.
Wire it as the shunt leg of a divider and it is a gain element that needs no
amplifier of its own. UREI's manual puts it plainly: "the FET acts like a
resistor whose resistance is controlled by the voltage applied to its gate.
The higher the voltage applied to the gate, the smaller the drain-source
resistance will be."

## This is a part, not a category

"VCA" is a category, and it covers at least three circuits that share a word
and not an equation: David Blackmer's log-antilog gain cell, an operational
transconductance amplifier, and this. **This crate is the third of the
three.**

- The first is next door as `noob-electrical-components-blackmer-cell`. Its
  law is a constant number of decibels per volt across the whole range, exact
  by construction, and its residual is a symmetry error.
- The second is not modelled anywhere in this repository or in the plug-ins
  that use it. If it ever is, it gets its own crate, because a
  transconductance is not a resistance and no shared type would fit both.

A crate called `Vca` would have had to be all three at once and would have
been none of them. The Blackmer cell was named for its inventor for exactly
this reason after an audit found three "tube stages" in the plug-ins that
turned out to be three different circuits wearing one name; this is the same
rule applied a second time.

## And it is not a FET amplifier

Same device, different operating region, different equation: saturation
rather than ohmic, a transconductance rather than a resistance, an
odd-symmetric square law rather than a resistance modulated by its own
drain-source swing.

The 1176's Rev A settles this better than an argument could, because it
contains **both**. Its signal preamp is built on a FET and its gain-reduction
element is a FET, and the two are not the same component. Only the second is
here. The preamp's soft clipping is an amplifier stage and stays in the
plug-in with the line amp and the shelf.

## Nor the other signal-controlled resistances in this repository

| Crate | Controlled by | Law | Symmetry | Distortion against gain reduction |
|---|---|---|---|---|
| this one | gate voltage | resistance modulated by its own drain-source swing | **even** dominant | largest at moderate reduction |
| `diode-bridge` | tail current | `I·tanh(u/k)` | odd only | **falls** as reduction rises |
| `photocell` | light | conductance a power law in illumination | odd | rises with reduction |

Three variable resistances, three equations. That is why each is its own
crate rather than one `VariableResistor` with a mode switch.

## What it knows

Two behaviours, which is the whole crate:

| Property | Form | Where it comes from |
|---|---|---|
| Control law | `−max·(1 − exp(−slope·v/max))` | Fitted shape. A JFET's ohmic resistance goes as `1/(1 − V_gs/V_p)`; this is a closed form that reproduces the near-constant dB per volt near pinch-off and the plateau, in one expression with no implicit solve. |
| Plateau and slope | the caller's | The plateau is set by the on-resistance **and** the series resistance together, so it is not a property of the transistor. The 1176 dossier estimates 35 to 40 dB from 27 kΩ against a few hundred ohms. |
| Channel nonlinearity | `1 + even·u + odd·u²` on the conductance | The drain-source dependence of channel resistance. The coefficients are fitted by the caller; nothing published gives them for any transistor. |
| Reference swing | 250 mV | EDN: distortion under 3 % within ±250 mV, "reasonably" low below about 500 mV peak to peak. The one published number here. |
| Half swing | 0.5 | The 1176's low-noise circuit, "reduced voltage to the gain-reduction FET", fitted so the FET "stayed as much within a linear region as possible". |

The even term makes the second harmonic and the odd term the third, one
order higher than each looks, because the modulation multiplies a signal
that is itself the drive. The even one dominates, which is the character EDN
describes and the character people mean by the sound of a FET compressor.

## What it must not know

The 27 kΩ series resistor the channel shunts, the sidechain that develops
the gate voltage, the ratio ladder and diode bias that scale it, the preamp
and line amp after it, and the transformers either side. Those are the
machine. `conductance_ratio` hands over a conductance and stops; closing the
divider is one line in the caller, and it is the caller's line because the
divider differs from unit to unit while the channel does not.

The 1176's "all buttons in" mode is a good test of that line and falls on
the machine's side of it. Pulling the ratio buttons in shifts the bias and
sags the supply, and the model answers by moving the plateau, offsetting the
control voltage and scaling the even-order term. Those are things the
machine does *to* the part, expressed in the part's own parameters, so they
are arguments passed in rather than a mode inside the crate.

## Who is documented to contain one

One unit, on its manufacturer's own drawing: the **UREI / Universal Audio
1176**, whose manual gives the gain-reduction schematic with the FET as the
shunt element of a divider whose series element is 27 kΩ, and names the part
the "VVR FET" (figures 3 and 4, pp. 30–31). The **6176** is the same
limiter behind a 610 preamp and so is the same evidence rather than a second
piece of it.

That is one documented user, and under the rule as it stood until 4
September 2026 this part would still be waiting; it was on the coming
candidates list for that reason. It is here because the rule changed and
every component a plug-in models now belongs in this repository. The
evidence is recorded as it is rather than as the old rule would have wanted
it, because a crate that overstates what fixed its shape is how the
tube-stage mistake happened.

Two related units are named in the plug-in's dossiers and neither shaped
this crate. The **1178** is the stereo sibling and a repair thread about it
supplies the residual-harmonics figure quoted above; the **Distressor** has
a FET-flavoured mode but is modelled through a different gain element. If
either is ever built against this crate, its coefficients are its own.

## What is estimated, and what is missing

- **The control law's shape.** Not derived from device physics and not
  anchored by any measurement. It is chosen because it reproduces the two
  behaviours that matter in one expression.
- **The nonlinearity coefficients.** Fitted by whoever uses the crate, which
  is why no named sets ship here. What the sources fix is the ordering — the
  even term dominates unless the gate is fed half the drain-source signal —
  and the scale at which it stays small.
- **The bound on the modulation.** The polynomial is a local fit and a large
  enough drive would send it negative and invert the caller's divider. The
  bound stands in for the channel leaving its ohmic region altogether, which
  is not modelled. It is a guard rail, and the code says so rather than
  dressing it as physics.
- **Not modelled at all:** temperature, which moves pinch-off; gate-drain
  capacitance, which puts a frequency dependence on the gate network; and
  the gate linearisation itself, whose effect reaches the model only through
  coefficients a caller fitted with it already in place.

## Sources

- Universal Audio, *Model 1176LN Solid-State Limiting Amplifier, Operating
  Instructions* (2009 reissue, part 65-00046), pp. 30–31, figures 3 and 4.
- EDN, *A guide to using FETs for voltage controlled circuits*, parts
  [1](https://www.edn.com/a-guide-to-using-fets-for-voltage-controlled-circuits-part-1/)
  and [2](https://www.edn.com/a-guide-to-using-fets-for-voltage-controlled-circuits-part-2/):
  the ohmic-region resistance law, the drain-source dependence and its
  second-harmonic character, the ±250 mV figure, and the
  half-signal-to-gate linearisation.
- GroupDIY, [*UREI 1178 fet distortion*](https://groupdiy.com/threads/urei-1178-fet-distortion.82140/):
  residual harmonics over 60 dB down in normal operation, and the source
  bootstrapping that keeps the drain-source swing small.
