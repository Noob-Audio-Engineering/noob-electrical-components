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
| `noob-electrical-components-remote-cutoff-triode` | `remote-cutoff-triode` | The remote-cutoff triode: the gain element of a variable-mu limiter, with a parameter set per valve type, one of them refitted where the published law was never constrained. |
| `noob-electrical-components-diode-arm-pair` | `diode-arm-pair` | The diode gain element of the EMI TG12413 and its Zener limiter lineage: two arms of series junctions on a common supply rail, run in reverse breakdown. Four diodes like the bridge above and a different circuit in six structural respects; each crate says so at length, because the two will otherwise be merged. |
| `noob-electrical-components-log-rms-detector` | `log-rms-detector` | Blackmer's log-domain true-RMS detector: a bilateral log converter charging a capacitor against a constant current, whose rate-limited release and step-dependent attack are one time constant seen from two sides. The technique and no ballistics, because that boundary was drawn by a refusal. |
| `noob-electrical-components-fet-variable-resistor` | `fet-variable-resistor` | A junction field-effect transistor used as a voltage-controlled variable resistor: the 1176's gain element, its saturating control law and the channel resistance its own drain-source swing modulates. One of the three circuits "VCA" would have covered, and the crate names the other two. |
| `noob-electrical-components-small-signal-triode` | `small-signal-triode` | The ordinary preamp valve: half a 12AX7-class double triode in a class-A common-cathode stage, as a fixed-shape saturating law whose bias sets the asymmetry and never the gain. A second valve, and not the remote-cutoff one; each crate says why the other cannot serve, because these two will otherwise be merged. |
| `noob-electrical-components-transformer` | `transformer` | An audio transformer's low end: the roll-off its magnetising inductance puts under the band, as a corner and a Q of either order, and the flux its core can carry before the rest of it stops reaching the secondary. The linear failure and the nonlinear one, which is why transformer distortion arrives at the bottom of the band first. |

## What belongs here, and what does not

A component earns a place **once something real shares it, or is about to**.
I am not trying to atomise a codebase into parts that each have one caller.
An abstraction pulled out of a single user is usually the wrong shape for the
second one, and I would rather discover the right shape from two real users
than guess it from one.

The photocell qualified on both counts. Two compressors already shared it,
the LA-2A and the LA-3A, which use the same cell — every time constant, the
panel law and the exponent — and differ in how hard they light it, in the
resistances around it, and in one number of the part's own: the lit
resistance, 500 Ω against 400 Ω, each derived from that unit's published
maximum gain reduction. This page used to say they differ *only* in the
lighting and the surrounding resistances, which was wrong by one number,
and the number it was wrong about is one of the three a photoresistor
has. And a third established
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

**And the refuser is now a caller, which is what finally tested the general
half.** The CL-1B does not reimplement that distortion term; it calls this
crate's, with its own strength and reference amplitude, so the three
optical compressors here run one law at three depths. Wiring the rest of it
up found the one thing the general half was missing. The resistance law had
the T4's two endpoints and the single conductance scale that ties them
built into it, because in a T4 the lit resistance *is* the scale: `K_G` is
`1/R_MIN − 1/R_DARK`, so full carriers land exactly on `R_MIN`. In the
CL-1B they are two unrelated numbers. Its scale is solved from a
service-manual calibration, 250 mV of side-chain drive for exactly 10 dB of
reduction, while its minimum resistance is a separate estimate whose only
job is to set a maximum reduction nobody publishes. `Photoresistor` holds
the three separately and `Photoresistor::T4` is the case where two of them
coincide. Two users sharing an actual T4B could never have shown that,
because for them the question never arises; it took a third that shares the
law and nothing else.

The crate's own documentation names which unit supplied which half, and
both cases are asserted in its tests, because three fields where two would
apparently do is exactly what a later reader tidies away. The tie is real
for the T4 and only for the T4, and the test that pins it is there so
deleting the third field fails rather than passes.

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

## The rule changed on 2026-09-04

**Every electrical component a plug-in models belongs here**, on the user's instruction, whatever
its user count. What follows below was the rule until then, and it is kept because the reasoning
still applies to *how* a component is drawn even though it no longer decides *whether* one exists.

So a part is no longer required to wait for a second documented user. What is still required is
everything the old rule taught about shape:

- **A component is a part, never a category.** "VCA" would have covered a log-antilog cell, an
  operational transconductance amplifier and a field-effect transistor, which share a word and not
  an equation. It is the Blackmer cell for that reason, and the same test applies to every crate
  added from here. Two of those three are now here under their own names, as
  `blackmer-cell` and `fet-variable-resistor`, and their laws have nothing in common: a constant
  number of decibels per volt against a saturating resistance modulated by its own drain-source
  swing. One crate would have had to be both.
- **The crate holds the part and nothing around it.** The resistors that bias it, the detector that
  drives it and the make-up gain after it are the machine, and they differ from unit to unit while
  the part does not.
- **A shape derived from one implementation is usually wrong for the second.** Where two plug-in
  engines already contain the same part, reconcile both against the crate rather than lifting
  whichever was written first.

  That has now happened once, to the Blackmer cell, and what it found is the
  argument for doing it this way. The two engines agreed on the control law
  exactly and disagreed about the shape of the even-order residual: one writes
  it as a gain mismatch between the two half-wave paths, the other as a smooth
  squared term. The crate, written with no real users at all, had picked the
  squared term while its own prose described the mechanism that gives the
  other. So the reconciliation changed the crate rather than either engine. It
  now carries both shapes, names the mechanism each stands for, and sets out
  which published figures pull which way — because the datasheet's own three
  distortion rows do not settle it, and a crate built from either user alone
  would have shipped a shape it had no right to.
- **Record the evidence, and its asymmetry.** A part read off a manufacturer's drawing and a part
  inferred from behaviour are both admissible now, but they are not the same thing and the crate
  says which is which.

## Candidates, and what became of them

Named here so the boundary stays deliberate rather than accreting. **This
list is now a history rather than a queue**: on 2026-09-04 every component
the plug-in modelled was moved here, so nothing on it is waiting any more.
The entries are kept because how each one arrived is worth more than the
fact that it did, and because two of them were argued about at length on
grounds that turned out to be wrong.

- **Log-domain RMS detector — built**, as
  `noob-electrical-components-log-rms-detector`. This was the last entry here
  to be argued about at length, and the argument is worth keeping because what
  settled it was the rule changing rather than any new evidence arriving.

  It waited on two independent grounds. The first was the admission rule as
  written: two units *documented* to contain the part, meaning on a
  manufacturer's drawing. The dbx 160 meets that. The API 2500 does not, and
  its own dossier is why — no API schematic exists publicly and nothing below
  block level comes from API, so the part identity is a reviewer's report. The
  second ground would have applied even if the first were met: there was one
  implementation of the detector in the plug-in, and an abstraction pulled from
  a single implementation is usually the wrong shape for the second.

  **Neither ground has gone away; the rule moved past them.** The crate
  therefore exists on one drawing and one report, and it says so on its own
  front page rather than letting the report look like corroboration. What the
  reported second unit did contribute survives intact, and it is the boundary:
  the dbx has no attack or release controls because its detector *is* its
  ballistics, the API has fourteen because its panel ballistics are a second
  stage after the detector, and the component holds neither.

**The transformer has been built**, and its entry here named the wrong second
user. It read that the part "already exists in the 610 preamp of the
compressor lab's 6176, and would be wanted by any variable-mu unit". The
first half was right. The second was a guess, and the unit that actually
turned out to share the part is the 1176 sitting next to the 610 in the same
box, which nobody had named. That is the diode bridge's lesson from the other
side: the bridge was admitted on a predicted second user who never arrived,
and the transformer waited on a predicted second user while its real one was
already in the building. A prediction about *which* unit shares a part is
worth no more than a prediction *that* one will.

**Everything has now left this list**, and the differences between how they
left are the most useful thing on this page. Two are worth reading before
adding anything new.

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
was a different circuit. The next candidate to face this was the remote-cutoff
triode, and it never had to: the rule above changed first. It is built, with a
parameter set per valve type, so a second unit fits its own valve against its
own curves rather than inheriting one on a prediction.

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
