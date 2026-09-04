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

- **Tube stage and transformer.** Both already exist in the 610 preamp of
  the compressor lab's 6176, and both would be wanted by any variable-mu
  unit, which is the next family to model.
- **FET.** The 1176's gain element, currently inside that engine.
- **VCA.** The Distressor's, likewise, and shared with every mainstream
  VCA compressor.
- **Variable-mu element.** The gain element of a whole family the plug-ins
  do not cover yet, so it will arrive with the first unit that needs one.

The diode bridge was on this list and has since been built, arriving with
the Neve 33609 as this note predicted. It is the one case so far admitted on
the second half of the rule, *about to* be shared rather than already
shared: it has one user today, and a second, the EMI TG12413, is next but
one in the plug-in's build order. That is a weaker justification than the
photocell's and is recorded as such.

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
