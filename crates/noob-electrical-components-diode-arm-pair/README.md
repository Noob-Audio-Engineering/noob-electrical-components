# noob-electrical-components-diode-arm-pair

The diode gain element built as two arms of series junctions hanging from
one common supply rail, opposed across the audio. A control source sinks a
bias current down each arm, the signal current transfers from one arm to
the other, and the bias current therefore sets a resistance that whatever
divider the machine puts around it turns into a gain.

EMI used exactly this in the TG12413, the dynamics module of the Abbey Road
TG consoles: four HS2051 diodes in two branches of two, sharing the +20 V
rail. As drawn every cathode faces the rail and every anode faces a
transistor sinking current through it, which means the diodes conduct in
**reverse breakdown** rather than forward. EMI's own limiter lineage runs
through a product called the RS168 *Zener* Limiter, and both companies that
have built recreations with access to the hardware call this element a
zener limiter, so the drawing is probably right.

## This is not the diode bridge

`noob-electrical-components-diode-bridge` also models a part with four
diodes in it, and they will be merged by anyone who reads no further than
that. They are different circuits.

| | diode bridge | this part |
|---|---|---|
| arrangement | a closed **ring**, two opposed pairs | **two branches in series**, both the same way up |
| junctions per arm | **one** | **two** |
| common node | **two, both floating** | **one, the supply rail** |
| operating region | **forward**, biased by an injected current | **reverse breakdown** as drawn |
| control enters at | the two floating nodes | the transistor collectors below each arm |
| balance | ring symmetry; mismatch not modelled | a fixed resistor against two adjust-on-test ones; mismatch modelled |
| the law | `i = I·tanh(u/k)`, explicit in voltage | `u(i)` of (G1), explicit in current |
| used by | Neve 2254 and 33609 | EMI TG12413 |

The bridge crate was written expecting this element to be its second user,
on a survey's word. It is not one, and that finding is why the plug-in
family holding both was renamed from "diode bridge" to "diode".

### What the two genuinely share

Two numbers and a numerical convention, all three cited from outside both
crates rather than taken from one another. The thermal voltage `kT/q` at
300 K is physics and belongs to neither part. The ideality factor is a
published fit to a 1N4148 that both crates borrow because neither models a
diode with a reachable datasheet — 1N4153-class parts there, the HS2051
here — and the two borrowings are independent, so neither is evidence for
the other. And both treat a bias current below a picoamp as an open
circuit, which is a guard against dividing by nothing rather than a
property of any component.

**A shared citation is not a shared component**, so this crate has no
dependency on that one.

### What only looks shared, which is the half that causes the merge

**The law.** The two coincide only at one junction per arm with no bulk
term, which *is* the bridge, so the coincidence says the general law
contains the special one and nothing more. `DiodeArmPair::ring` exists so
that containment can be asserted as a test rather than argued in a
paragraph. As drawn this part is in breakdown, which is tunnelling below
about 5 V and avalanche above about 6 V, neither of which is the diode
exponential, and neither of which yields a hyperbolic tangent when two arms
are put in opposition.

**The scale constants, and this is the trap.** The bridge's thermal scale
is `2·η·V_T`, about 90.7 mV, and this part's forward `v_n` at two junctions
per arm is *also* `2·η·V_T`, about 90.7 mV. Same number, different reason:
the bridge's factor of two is two arms in opposition each contributing one
junction, and this part's is two junctions in series inside one arm. They
also sit in different places in the two formulas, since the bridge's scale
is the whole denominator of its tanh argument while `v_n` here is half of
it. Matching the constants and concluding the parts are the same is wrong
by a factor of two in the argument and, because the third-harmonic ratio of
`tanh(a·sinθ)` is `a²/12`, by a factor of four in the third harmonic.

**The Newton solve.** Both parts hand their machine an implicit node
equation and both machines solve it with a linear seed and one or two
corrections, but the bridge's law is explicit in voltage and is solved in
`u` while this one is explicit in current and is solved in `i`. That is a
code pattern with the variable reversed. It belongs to whichever machine
owns the divider and it is in neither crate.

## What is here and what is not

The pair of arms alone: the differential voltage across it for a signal
current at a bias current, that law's slope, the small-signal resistance it
presents and the inverse of that resistance.

Not here: the series resistance the source drives it through, the divider
that resistance forms with it, the sidechain that produces the bias
current, the coupling capacitors either side or the output ladder after it.
Those are the machine. Solving a particular divider's node equation is the
caller's job, which is what the slope is exported for.

The operating region is a **choice the caller makes**, not a default the
crate hides. The drawing supports two readings of it, the reading changes
the sound, and it cannot be settled from the available evidence, so a
machine can be corrected by changing which constructor it calls rather than
by rewriting the part.

## What is estimated

**All of the numbers, and more completely than for any neighbouring part.**
No factory handbook, no specification and no measurement of the module this
comes from has ever been published; the evidence is one photographed
blueprint, two companies' prose about their own recreations, and
arithmetic.

- The **ideality factor** is a fit to a different diode, as it is in the
  bridge crate, and it enters only through the one-junction scale.
- The **breakdown knee scale** has no source at all. Breakdown is not the
  diode exponential, so the forward figure does not carry over. It is a
  calibration knob with a plausible starting value.
- The **bulk resistance** is an order of magnitude inferred from the
  drawing rather than measured: a fixed 24 Ω on one branch against two
  adjust-on-test resistors in parallel on the other is what a designer
  fits to trim the balance between two branches that must carry the same
  current, and you trim against ohms because ohms is what a device in
  breakdown presents.

Not modelled: temperature, junction capacitance, reverse recovery and the
part's own noise. Temperature is the interesting exclusion, because a
forward junction, a zener below 5 V and an avalanche device above 6 V have
three different signs of coefficient and the device is unidentified, so the
sign is unknown. Modelling it would mean inventing it.

## What the tests can and cannot assert

Nothing is published, so nothing here asserts a measured figure. The tests
assert the derived properties of the law — odd symmetry with matched arms,
even order appearing monotonically with imbalance, the `a²/12` third
harmonic and its quartering at two junctions per arm, the resistance floor
that breakdown has and forward does not — and each says that its reference
is a derivation.

The identity against the diode bridge's tanh is the load-bearing one. It
holds to the last bits an `f32` has, and it is written so that anyone
merging the two crates has to break a test to do it.

## Sources

- EMI, drawing TG12413-D101: D1–D4 HS2051 in two series branches on the
  +20 V rail, the two matched-pair callouts, R14's 20 kΩ series arm and
  R16's 24 Ω against the two adjust-on-test resistors.
- Chandler Limited, who build this circuit under licence from EMI, on the
  1968 RS168 Zener Limiter lineage through the TG12345 and the TG12413,
  and on "a rarely seen diode network".
- Waves, who modelled the module jointly with Abbey Road Studios and had
  the console, naming the element a "Zener diode limiter" three times in
  one user guide.
- C. V. Pines, "Real-Time Virtual Analog Modelling of Diode-Based VCAs",
  DAFx-25, Ancona 2025, pages 63–70, for the ideality factor and thermal
  voltage, and for the odd-symmetry result reached independently for a
  symmetric diode gain element.

```toml
[dependencies]
noob-electrical-components = { git = "https://github.com/Noob-Audio-Engineering/noob-electrical-components", features = ["diode-arm-pair"] }
```
