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

  **And when it is built, it must not assume the family has one shape.**
  Both candidate tubes were fitted to the same law, a transconductance
  decaying as `exp(-(w/V0)^n)`, and their exponents were measured from the
  manufacturers' own published curves rather than assumed. The Fairchild's
  true remote-cutoff tube comes out at 1.01, give or take 0.16, which is a
  clean exponential; the 176's semi-remote-cutoff tube comes out at 2.16,
  which accelerates. Forcing the second figure onto the first costs an order
  of magnitude in fit residual, and measuring both over the same span widens
  the gap rather than closing it. So `n` is a per-tube-type parameter and
  every tube needs its own fit against its own curve. Fitting one and
  assuming the other is the specific mistake this note exists to prevent,
  and it was nearly made twice: once from three datasheets that all quote a
  single operating point, and once from comparing two average tapers, which
  is a first-moment statistic and is blind to the curvature the exponent
  measures.
- **FET.** The 1176's gain element, currently inside that engine.

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
