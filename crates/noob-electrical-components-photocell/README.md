# noob-electrical-components-photocell

The light-dependent resistor at the heart of an optical compressor: an
electroluminescent panel glued to a cadmium-sulphide photoresistor, as the
T4 modules of the LA-2A and LA-3A use.

Its behaviour is not a set of time constants somebody chose. The panel's
light follows the Alfrey-Taylor law, the photoresistor's conductance
follows a power law in that light, and its carriers fall into traps and
climb out again. The ten-millisecond attack, the sixty-millisecond first
release stage, the half-to-five-second second stage and the programme
dependence all fall out of that, which is why a compressor built on this
cell behaves like the hardware without anybody dialling in a curve.

Three variants are carried, the positions an LA-2A offers. What is
documented about them and what is an estimate is set out at `CELL_SPEEDS`
and `CellParams::fast_share`; in short, the ordering is a manufacturer's
description, the magnitudes are estimates, and the one difference with a
physical basis is the third, faster photocell the earliest cells carried,
which no speed multiplier can express because the point is a shape.

This crate is the cell alone. The divider it shunts, the sidechain that
drives it and the make-up gain after it belong to the machine using it.

```toml
[dependencies]
noob-electrical-components = { git = "https://github.com/Noob-Audio-Engineering/noob-electrical-components", features = ["photocell"] }
```
