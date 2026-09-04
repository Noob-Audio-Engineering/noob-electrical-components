# noob-electrical-components-photocell

The photoconductive element at the heart of an optical compressor, and the
T4-family cell built around it.

Two things live here and the difference matters. The resistance laws and
the distortion term are properties of any photoresistor: the Tube-Tech
CL-1B, whose potted element is emphatically not a T4 and which refuses this
crate's timing entirely, still obeys them. `Cell` and its time constants
are the T4 specifically, an electroluminescent panel glued to a
cadmium-sulphide photoresistor, as the LA-2A and LA-3A use.

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

This crate is the element and the cell alone. The divider it shunts, the
sidechain that drives it and the make-up gain after it belong to the
machine using it.

## Two named simplifications, and one operating limit

The photoconductive exponent is fixed. A real cell's is not: the datasheet
figure is defined only between 10 and 100 lux, published values span 0.6 to
0.9, and the physical literature uses a dual-slope power law rather than a
single exponent. No source gives the endpoints a light-dependent exponent
would need, so it stays fixed and is documented rather than invented.

The electroluminescent law is an alternating-current one whose brightness
also rises with drive frequency, and this drives it from a rectified,
smoothed envelope, which discards that. A real panel is brighter for
high-frequency-dense programme at the same level. A compressor built on
this may still react more to highs through a sidechain emphasis filter, but
that is a different mechanism and should not be mistaken for this one.

Together those put a hard corner where a real cell has a gradual one: above
about 4.2 V of drive, generation is pinned at the state's clamp and neither
law contributes further, so 5 V and 50 V are identical. Keep a sidechain
inside that range. The limit is asserted, along with the fact that the
model stays responsive above it.

```toml
[dependencies]
noob-electrical-components = { git = "https://github.com/Noob-Audio-Engineering/noob-electrical-components", features = ["photocell"] }
```
