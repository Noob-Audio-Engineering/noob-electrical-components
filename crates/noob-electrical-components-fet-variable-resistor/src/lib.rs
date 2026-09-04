//! A junction field-effect transistor used as a voltage-controlled
//! variable resistor: the channel a control voltage opens and closes, and
//! the way that channel's resistance bends with the signal across it.
//!
//! Biased into its ohmic region a JFET is a resistor whose value the gate
//! sets. Wire it as the shunt leg of a divider and it is a gain element
//! that needs no amplifier of its own: the control voltage moves the
//! channel resistance, the resistance moves the divider, and the divider
//! moves the gain. UREI's 1176 is the famous one. Its manual calls the part
//! the "VVR FET" and puts it plainly: "the FET acts like a resistor whose
//! resistance is controlled by the voltage applied to its gate. The higher
//! the voltage applied to the gate, the smaller the drain-source resistance
//! will be."
//!
//! Two behaviours make this worth modelling rather than multiplying by a
//! gain, and this crate is both of them and nothing else:
//!
//! 1. **The control law is a curve, not a line.** Attenuation rises at
//!    something close to a constant number of decibels per volt while the
//!    channel is near pinch-off, then flattens once the on-resistance stops
//!    falling against whatever series resistance the machine put in front
//!    of it. [`attenuation_db`] is that shape, and the plateau is why a
//!    feedback limiter's ratio stops climbing at depth.
//! 2. **The resistance depends on the signal across it too.** A JFET's
//!    channel resistance is a function of drain-source voltage as well as
//!    gate voltage, so the divider's gain is not constant within a cycle.
//!    [`conductance_modulation`] is that, and it is the whole of what
//!    people mean by the sound of a FET compressor.
//!
//! # What this is, and what it is not
//!
//! **It is a part, not a category.** "VCA" is a category, and it covers at
//! least three circuits that share a word and not an equation: David
//! Blackmer's log-antilog gain cell, an operational transconductance
//! amplifier, and this. This crate is the third of those three. The first
//! lives next door in `noob-electrical-components-blackmer-cell`, whose law
//! is a constant number of decibels per volt over the whole range and whose
//! residual is even-order symmetry error. The second is not modelled
//! anywhere in this workspace, and if it ever is it gets its own crate
//! rather than joining this one, because a transconductance is not a
//! resistance. A crate called `Vca` would have had to be all three at once
//! and would have been none of them.
//!
//! **It is not a FET used as an amplifier.** Same device, different
//! operating region and different equation: saturation rather than ohmic, a
//! transconductance rather than a resistance, an odd-symmetric square law
//! rather than a resistance modulated by its own drain-source swing. The
//! distinction is not academic — the 1176's Rev A contains both, a FET
//! signal preamp *and* the gain-reduction FET, and only the second is this
//! crate. The preamp's soft clipping is an amplifier stage and belongs to
//! the machine.
//!
//! **It is not the other signal-controlled resistances in this
//! repository**, and the differences are in the law rather than in the
//! parameters:
//!
//! | part | controlled by | law | symmetry | distortion against gain reduction |
//! |---|---|---|---|---|
//! | this crate | gate voltage | resistance modulated by its own drain-source swing | **even** dominant | largest at moderate reduction |
//! | `diode-bridge` | tail current | `I·tanh(u/k)` | odd only | **falls** as reduction rises |
//! | `photocell` | light | conductance a power law in illumination | odd | rises with reduction |
//!
//! Three variable resistances, three equations. That is the reason each is
//! its own crate.
//!
//! **It is not the divider.** The series resistor the channel shunts (27 kΩ
//! in the 1176, R5), the sidechain that develops the gate voltage, and the
//! make-up gain afterwards are the machine. They differ from unit to unit
//! while the part does not. [`conductance_ratio`] hands the caller the
//! conductance and stops there; solving the divider is the caller's job,
//! and it is one line.
//!
//! # How a machine uses it
//!
//! ```
//! use noob_electrical_components_fet_variable_resistor as fet;
//!
//! // The machine's own numbers: how deep this divider can attenuate, how
//! // many dB per volt the sidechain buys, and what a full-scale signal
//! // across the channel is worth in the crate's drive units.
//! let (slope, max_db) = (48.0, 48.0);
//! let scale = fet::swing_scale(0.02, false);
//! let shape = fet::Nonlinearity::new(0.15, 0.05);
//!
//! // Per sample: the control voltage sets the conductance, the previous
//! // swing across the channel modulates it, and the divider closes.
//! let w = fet::conductance(0.3, slope, max_db);
//! let m = fet::conductance_modulation(0.1 * scale, shape);
//! let gain = 1.0 / (1.0 + w * m);
//! assert!(gain > 0.0 && gain < 1.0);
//! ```
//!
//! # What is estimated
//!
//! **The control law's shape is a fitted form, not a device equation.** A
//! JFET's ohmic-region resistance goes as `1 / (1 − V_gs/V_p)`; the
//! exponential approach to a plateau used here is a closed form chosen
//! because it reproduces the two things that matter — the near-constant
//! decibels per volt near pinch-off and the flattening where the
//! on-resistance meets the series resistor — in one expression with no
//! implicit solve. It is not derived from device physics and no measurement
//! anchors it.
//!
//! **The plateau is not a property of the transistor.** It is set by the
//! on-resistance *and* the series resistance together, so this crate takes
//! it as an argument rather than publishing a constant. The 1176 research
//! estimates 35 to 40 dB from a 27 kΩ series resistor and an on-resistance
//! of a few hundred ohms; a model tuned against behaviour may want more.
//!
//! **The nonlinearity coefficients are fitted by whoever uses this**, which
//! is why [`Nonlinearity`] ships no named sets. Nothing published gives a
//! second- or third-order coefficient for a JFET in this service. What *is*
//! published, and what a caller's numbers should respect, is the ordering
//! and the shape:
//!
//! - The distortion is **predominantly second harmonic** ([EDN]). An even
//!   term smaller than the odd one is not this part.
//! - It is small while the swing is small: below roughly 500 mV
//!   peak-to-peak it can be kept "reasonably" low, and under 3 % within
//!   ±250 mV ([EDN]). [`REFERENCE_SWING_VOLTS`] carries that figure.
//! - **Feeding exactly half the drain-source signal back to the gate
//!   cancels the second-order term and leaves a much smaller third-order
//!   residue** ([EDN]). Every 1176 does this through two 2.2 MΩ resistors,
//!   which is why its residual harmonics measure "over 60 dB down" in
//!   normal operation ([GroupDIY]) and why its coefficients are as small as
//!   they are.
//! - Halving the swing that reaches the channel, as the 1176's low-noise
//!   circuit does by dropping the voltage across the gain FET, is a
//!   separate mechanism with an arithmetic consequence: the second-order
//!   product falls by two and the third-order by four.
//!   [`swing_scale`] is that halving.
//!
//! **Not modelled:** temperature, which moves pinch-off; gate-drain
//! capacitance, which puts a frequency dependence on the gate network;
//! and the collapse out of the ohmic region into saturation when the
//! drain-source swing gets large, for which [`MODULATION_FLOOR`] and
//! [`MODULATION_CEILING`] stand in as a bound rather than a model.
//!
//! # Sources
//!
//! - Universal Audio, *Model 1176LN Solid-State Limiting Amplifier,
//!   Operating Instructions* (2009 reissue, part 65-00046), pp. 30–31 and
//!   figures 3 and 4: the VVR FET as the shunt element of a divider whose
//!   series element is 27 kΩ, the 2.2 MΩ gate network, and the low-noise
//!   circuit's purpose of keeping the FET "as much within a linear region
//!   as possible".
//! - [EDN]: EDN, *A guide to using FETs for voltage controlled circuits*,
//!   parts 1 and 2, for the ohmic-region resistance law, the drain-source
//!   dependence and its second-harmonic character, the ±250 mV figure, and
//!   the half-signal-to-gate linearisation.
//! - [GroupDIY]: GroupDIY, *UREI 1178 fet distortion*, for the residual
//!   harmonics in normal operation and the source bootstrapping that keeps
//!   the drain-source swing small.
//!
//! [EDN]: https://www.edn.com/a-guide-to-using-fets-for-voltage-controlled-circuits-part-1/
//! [GroupDIY]: https://groupdiy.com/threads/urei-1178-fet-distortion.82140/

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Drain-source swing at which a JFET's ohmic-region distortion stops being
/// negligible, in volts.
///
/// EDN quotes distortion below 3 % within ±250 mV and calls it "reasonably"
/// low below about 500 mV peak-to-peak. It is a threshold of audibility
/// rather than a coefficient, so it belongs here as the natural unit of
/// [`conductance_modulation`]'s drive: at a drive of 1 the channel is being
/// worked as hard as that figure describes.
///
/// A machine whose signals are not in volts converts through
/// [`swing_scale`] rather than using this directly, because where its
/// full scale sits in volts is a property of the machine.
pub const REFERENCE_SWING_VOLTS: f32 = 0.25;

/// The factor by which a reduced-drive circuit cuts the swing reaching the
/// channel.
///
/// The 1176's low-noise circuit is described as "reduced voltage to the
/// gain-reduction FET", fitted so the FET "stayed as much within a linear
/// region as possible". Halving is the value that circuit takes, and the
/// arithmetic that follows is the point of it: the second-order product
/// falls by two and the third-order by four.
pub const HALF_SWING: f32 = 0.5;

/// Lower bound on [`conductance_modulation`].
///
/// Not physics. The polynomial is a local fit around a small swing, and a
/// large enough drive would drive it negative and invert the divider the
/// caller builds from it. Bounding it is what a real channel does by
/// leaving the ohmic region altogether; this is a guard rail in the place
/// where that model is missing.
pub const MODULATION_FLOOR: f32 = 0.5;

/// Upper bound on [`conductance_modulation`], for the reason given at
/// [`MODULATION_FLOOR`].
pub const MODULATION_CEILING: f32 = 2.0;

/// The channel's signal-dependent terms: how much its conductance moves
/// with the signal across it, and in what symmetry.
///
/// Both are dimensionless multipliers on the drive of
/// [`conductance_modulation`], so both are fitted against a machine's own
/// calibration and neither is published for any transistor. See the crate
/// documentation for what *is* published and what a fitted pair should
/// respect: the even term dominates unless the gate is fed half the
/// drain-source signal, and it stays small while the swing does.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Nonlinearity {
    /// Even-order term. It makes the second harmonic, and in a JFET used
    /// this way it is the dominant one; it is what a listener means by the
    /// warmth of a FET gain element.
    pub even_order: f32,
    /// Odd-order term. It makes the third harmonic. Gate linearisation
    /// leaves this behind after it has cancelled the even term, so a
    /// well-linearised channel is the one place where this is the larger of
    /// the two.
    pub odd_order: f32,
}

impl Nonlinearity {
    /// A channel with the given even- and odd-order terms.
    pub const fn new(even_order: f32, odd_order: f32) -> Self {
        Nonlinearity {
            even_order,
            odd_order,
        }
    }

    /// A perfectly linear channel: a resistance the signal does not move.
    ///
    /// No JFET is this. It is the reference the others are heard against,
    /// and it is what a machine passes when it wants the control law
    /// without the colour.
    pub const LINEAR: Self = Nonlinearity::new(0.0, 0.0);
}

/// Attenuation in decibels the channel produces at a given control voltage.
///
/// `slope_db_per_volt` is the initial slope near pinch-off and `max_db` the
/// plateau the divider reaches when the channel resistance stops falling
/// against the series resistance. The law is
///
/// ```text
/// G(v) = −max · (1 − exp(−slope · v / max))
/// ```
///
/// which starts at 0 dB with exactly `slope` decibels per volt and
/// approaches `−max` from above, never reaching it. The result is negative
/// or zero; a negative control voltage is not meaningful and returns a
/// positive number rather than being clamped, because clamping is a
/// decision about the sidechain and the sidechain is the machine's.
///
/// Both parameters are arguments rather than constants because neither is a
/// property of the transistor alone: the plateau is set by the series
/// resistance as much as by the on-resistance, and the slope is set by how
/// many volts the machine's sidechain develops. See the crate
/// documentation on what is estimated.
#[inline]
pub fn attenuation_db(control_v: f32, slope_db_per_volt: f32, max_db: f32) -> f32 {
    -max_db * (1.0 - (-slope_db_per_volt * control_v / max_db).exp())
}

/// The channel conductance implied by an attenuation, normalised by the
/// series resistance it works against.
///
/// For a divider of series `R` and shunt `r`, an attenuation of `g` means
/// `g = r / (R + r)`, so this returns `1/g − 1 = R / r`. It is the channel
/// conductance in units of one over the series resistance, which is the
/// only form the part can offer without being told a resistance the machine
/// owns. Zero at 0 dB (an open channel), 1 where the channel resistance
/// equals the series resistance (−6.02 dB), and unbounded as the
/// attenuation deepens.
///
/// This is what a caller multiplies by [`conductance_modulation`] before
/// closing its divider.
#[inline]
pub fn conductance_ratio(attenuation_db: f32) -> f32 {
    let g = 10f32.powf(attenuation_db / 20.0);
    1.0 / g - 1.0
}

/// The normalised channel conductance at a control voltage: the two steps
/// above in one call.
///
/// Equivalent to [`conductance_ratio`] of [`attenuation_db`], and the form
/// most callers want, since the decibels are an intermediate the divider
/// never sees.
#[inline]
pub fn conductance(control_v: f32, slope_db_per_volt: f32, max_db: f32) -> f32 {
    conductance_ratio(attenuation_db(control_v, slope_db_per_volt, max_db))
}

/// The scale from a machine's signal across the channel to the drive
/// [`conductance_modulation`] takes.
///
/// `reference` is the machine's own signal amplitude standing for
/// [`REFERENCE_SWING_VOLTS`] — the level at which the channel is worked as
/// hard as the ±250 mV figure describes. Where that lands in a machine's
/// units depends on where it puts full scale, so the machine supplies it.
///
/// `half_swing` is whether the machine reduces the voltage across the
/// channel, as the 1176's low-noise circuit does; it applies
/// [`HALF_SWING`]. Returning a scale rather than a scaled signal is
/// deliberate: it is loop-invariant and a caller can hoist it out.
#[inline]
pub fn swing_scale(reference: f32, half_swing: bool) -> f32 {
    if half_swing {
        HALF_SWING / reference
    } else {
        1.0 / reference
    }
}

/// How much the signal across the channel moves its conductance, as a
/// multiplier on the conductance the control voltage set.
///
/// `drive` is the signal across the channel in units of
/// [`REFERENCE_SWING_VOLTS`], which is what [`swing_scale`] produces. The
/// law is `1 + even·u + odd·u²`, bounded to
/// [`MODULATION_FLOOR`]..=[`MODULATION_CEILING`].
///
/// The even term is first order in the drive and so puts a *second*
/// harmonic on the output, the odd term second order and so a *third*: the
/// modulation multiplies a signal that is itself the drive, which raises
/// every order by one. That is why a JFET used this way is heard as a
/// second-harmonic device even though its dominant coefficient looks
/// linear.
///
/// A caller with a memoryless path must feed it the previous sample's
/// swing; using the current one makes the divider implicit in its own
/// output.
#[inline]
pub fn conductance_modulation(drive: f32, shape: Nonlinearity) -> f32 {
    (1.0 + shape.even_order * drive + shape.odd_order * drive * drive)
        .clamp(MODULATION_FLOOR, MODULATION_CEILING)
}

#[cfg(test)]
mod tests;
