# noob-electrical-components

Physical models of the electrical components audio hardware is built from,
so the plug-ins I write can share a part instead of each keeping its own
copy of it.

Each component is its own crate. A facade crate, `noob-electrical-components`,
re-exports them behind a feature each, so a plug-in writes one dependency
line and compiles in only what it uses:

```toml
[dependencies]
noob-electrical-components = { git = "https://github.com/Noob-Audio-Engineering/noob-electrical-components", features = ["photocell"] }
```

## What is here

| Crate | Feature | What it models |
|---|---|---|
| `noob-electrical-components-photocell` | `photocell` | The photoconductive element: its resistance and distortion laws, plus the T4-family cell built around it, with carrier traps and the programme dependence they cause. |
| `noob-electrical-components-diode-bridge` | `diode-bridge` | The balanced diode bridge used as a gain element: four matched diodes whose floating common nodes make its law a hyperbolic tangent. |
| `noob-electrical-components-blackmer-cell` | `blackmer-cell` | David Blackmer's log-antilog gain cell: the constant-decibels-per-volt control law behind the dbx 202 and the THAT 2180, with its tolerances, temperature coefficient and even-order symmetry residual. |

## What belongs here, and what does not

A component earns a place **once something real shares it, or is about to**.
I am not trying to atomise a codebase into parts that each have one caller.
An abstraction pulled out of a single user is usually the wrong shape for the
second one, and I would rather discover the right shape from two real users
than guess it from one.

The photocell qualified on both counts. Two compressors already shared it,
the LA-2A and the LA-3A, which use the same cell and differ only in how hard
they light it and in the resistances around it. And a third established
where its edge lies by deliberately *not* using it: the Tube-Tech CL-1B is
an optical compressor whose timing lives in an op-amp sidechain rather than
in the cell's own carriers, so borrowing this cell would have forced a
60 ms attack and a half-second release onto a machine whose release knob
runs to ten seconds. A boundary that has been tested by a real refusal is
worth more than one argued from first principles.

That refusal then drew the line inside the crate too. The CL-1B rejects the
T4's timing, its panel law and its every time constant, and still
implements the photoconductor's distortion term identically, so that term
is a property of any photoresistor rather than of the T4. The crate holds
both, and says which is which.

**Circuitry does not belong here.** A component is the part. The resistive
divider it shunts, the sidechain that drives it and the make-up gain after
it are the machine, and those differ from unit to unit while the part does
not. The photocell crate knows its own dark and lit resistances, because
those are properties of the cell; it knows nothing about the 70.7 kΩ series
resistor an LA-2A puts in front of it.

**General signal processing does not belong here either.** Filters,
oversamplers and antiderivative anti-aliasing are infrastructure rather than
components. They have no physical part behind them and they belong wherever
the DSP lives.

## Coming candidates

Named here so the boundary stays deliberate rather than accreting. None of
these is built yet, and each waits for a second real user:

- **Transformer.** Already exists in the 610 preamp of the compressor lab's
  6176, and would be wanted by any variable-mu unit.
- **Remote-cutoff triode.** The gain element of the variable-mu family, and
  not the same thing as the 610's tube stage. This entry used to read "tube
  stage and transformer" and to claim a variable-mu unit would want the
  610's; that was wrong. The 610's triode model was fitted for 12AX7-class
  valves, which have no remote-cutoff characteristic, so the two differ in
  functional form and not merely in parameters. If this crate ever holds
  both they are two components with two names, and this is the reason.

  **And when it is built, both tubes must be fitted by one documented
  procedure.** Two researchers tried to settle whether the candidate tubes
  share a shape, fitting transconductance to `exp(-(w/V0)^n)`, and the
  useful result is that published data cannot answer it, and it is now
  measured rather than inferred. One tube's maker plots it under four
  operating conditions, and fitting all four gives exponents of 1.00, 0.84,
  0.71 and 0.59, moving monotonically with the supply, every fit good to
  under half a decibel. One tube, four conditions, a factor of 1.7. The
  other tube's exponent moves from 2.16 to 1.71 depending on whether it is
  anchored on interior or endpoint values of its own single curve. So the
  exponent is condition-dependent and anchor-dependent, and two exponents
  read off two datasheets by two methods are not comparable at all.

  So the constraint is procedural rather than numerical. If this component
  is built, every tube's parameters come from **one** fitting procedure
  using the same class of anchor points, written down. Two people each
  fitting their own tube their own way and comparing the results afterwards
  would be agreeing on an artefact.

  **The constraint has three clauses and each was learned by getting it
  wrong.** One documented fitting procedure, because the procedure moves the
  answer. The same class of anchor points, because a fit to interior points
  is not a fit to endpoints. And curves measured in the **same topology**,
  because one tube's published plot is a cascode connection where the
  section's plate floats at the next tube's cathode, the other's is a
  single-section characteristic at a fixed plate voltage, and **neither is
  the grounded-cathode stage either compressor actually runs**. Without all
  three, two implementations agree on a number while disagreeing about the
  curve, which is worse than not sharing at all, because it looks like
  corroboration.

  **Where it finally landed: the shape question is unanswerable from
  published data, and no exponent quoted during the argument should be
  reused.** The figures that circulated were 1.01, 1.10, 1.58, 1.71, 1.98
  and 2.16, and each was someone's honest reading. The exponent moves by a
  factor of 1.7 across four operating conditions on a single page for a
  single tube; it moves from 2.16 to 1.71 for the other tube on a change of
  anchor points within one curve; and it moves from 2.16 to 1.58 between two
  manufacturers' curves for that same tube. Condition-dependent,
  anchor-dependent, procedure-dependent and maker-dependent. It is not a
  property of a tube, so it was never a quantity on which two tubes could be
  compared.

  **And this is now the sharpest argument for not building the part yet.**
  The parameter set a shared component would ship with today has been shown
  by its own author to be wrong in the deep end, six-fold off a published
  plate-resistance curve by ten volts of bias, which is where a limiter
  spends its loudest moments. Building now would cast that into a crate and
  give it to every future user.

  Two near-misses on the way there are worth keeping, because both looked
  like evidence. Three datasheets agreeing on an amplification factor say
  nothing about its bias dependence when all three quote one operating
  point. And two average tapers agreeing say nothing about shape, because an
  average slope is a first-moment statistic and is blind to the curvature an
  exponent measures.
- **FET.** The 1176's gain element, currently inside that engine.
- **Log-domain RMS detector.** The true-RMS technique, and only the
  technique. The dbx 160 dossier refused to extract it with one user and
  named the condition for revisiting: a second true-RMS unit. The API 2500
  is that unit, and it is better documented in one respect, since the dbx
  contains a potted module with no published datasheet while the API is
  reported to hold the part whose datasheet the dbx dossier already borrows.
  So the condition is closer to met than it was.

  **It still waits, and for a reason worth stating**, because it is not the
  one that stopped the diode bridge. The API 2500 is not being built. There
  is exactly one implementation of this detector in the plug-in, and an
  abstraction pulled out of a single implementation is usually the wrong
  shape for the second, which is this repository's founding argument.
  Extract it when a second true-RMS unit is **built**, not when a second
  one is known to exist.

  The boundary is already drawn by a refusal, which is the strongest kind.
  The dbx has no attack or release controls because its detector *is* its
  ballistics; the API has fourteen because its panel ballistics are a second
  stage after the detector. The component must hold neither.

Two entries have left this list, and the difference between how they left
it is the most useful thing on this page.

**The diode bridge** was admitted on the second half of the rule, *about to*
be shared rather than already shared, on the expectation that the EMI TG12413
would be its second user. **That expectation was wrong.** The TG12413's four
diodes are two series branches sharing a rail rather than a ring, and as drawn
they run in reverse breakdown, so it is not a bridge and this crate does not
serve it. The bridge still has exactly one user, the Neve 33609, and it is
now the standing example of why the weaker half of the rule is weaker: a
predicted second user is not a second user.

**The Blackmer cell** was on this list as "VCA", which was the wrong name for
it, and it has been built under the right one. It qualified on the strong half
of the rule and then some, with two documented users on two manufacturers' own
drawings: `VCA (200)` on dbx's schematic for the 160, and `DBX 202C` on SSL's
card 82E26 for the bus compressor. Renaming it mattered as much as building
it. "VCA" is a functional category covering the log-antilog cell, the
operational transconductance amplifier and a field-effect transistor used as a
variable resistor, which share a word rather than an equation; a crate called
`Vca` would have been the fourth tube stage.

**So the rule is tightened, and this is the change.** A component is admitted
when two units are documented to contain it, not when one does and another is
expected to. The *about to be shared* clause cost nothing to write and has now
been wrong once, and it was wrong in the most expensive way available: the
prediction came from a survey, the survey was believed, and the crate was
built before the second unit had been researched closely enough to notice it
was a different circuit. The next candidate to face this is the remote-cutoff
triode, which has the Fairchild 670 built and the Universal Audio 176 planned,
and it will wait for the 176 to actually contain one.

The cheap way to hold this line is to write the part in one separable place
inside the first plug-in that needs it, and lift it out when the second
arrives. That costs a small refactor and buys a shape derived from two real
users instead of one real user and a guess.

## Standards

Every test that checks a published figure asserts that figure and names its
source. Where no figure is published, the test says so and asserts what *is*
established, which for a component is usually a direction, an ordering or a
shape rather than a magnitude. A test that quietly compares a model against
its own output can never fail, and an audit of my plug-ins found nine of
those, so this is a rule rather than a preference.

## Building

```sh
cargo test
cargo clippy --all-targets --all-features
cargo doc --workspace --no-deps --all-features
```

## Licence

MIT or Apache-2.0, at your option.
