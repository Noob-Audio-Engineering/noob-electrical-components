//! The balanced diode bridge used as a gain element, whose law is a
//! hyperbolic tangent.
//!
//! Four matched diodes are wired as two pairs between two signal rails,
//! the anodes of one pair joined at a floating node and the cathodes of
//! the other joined at a second floating node. A DC control current enters
//! one common node and leaves the other, forward-biasing all four and
//! setting how much signal current the bridge will pass. Neve used exactly
//! this as the attenuator of the 2254 and the 33609.
//!
//! It was written expecting the EMI TG12413 to be its second user. **It is
//! not**, and the section below says why, because the reason is more useful
//! than the expectation was.
//!
//! # Why it is a tanh and not a Lambert W
//!
//! A single diode shunting a resistor gives an equation that is implicit
//! in the output voltage, and solving it needs the Lambert W function, or
//! the Wright omega recasting of it that Pines uses.
//!
//! A bridge is not that circuit. Because both common nodes float, each
//! pair is a current divider steered by the differential voltage, with the
//! control current as its tail. That is structurally a long-tailed pair,
//! and its transfer characteristic is
//!
//! ```text
//! i(u) = I · tanh( u / (2·η·V_T) )
//! ```
//!
//! with no implicit resistive term to solve for. The two floating nodes
//! are what remove it. Three things follow, and they are the reasons to
//! model the part rather than multiply by a gain:
//!
//! 1. **It is an odd function**, so the bridge itself makes no even
//!    harmonics at all. Any even order in a real unit comes from the
//!    transformers, the amplifier or a mismatch between the four diodes,
//!    which is why Neve specified matched pairs.
//! 2. **The small-signal resistance is `r = k / I`**: control current sets
//!    resistance, resistance sets whatever divider the machine puts
//!    around it, and that sets the gain.
//! 3. **Distortion falls as gain reduction rises.** More control current
//!    means less resistance, which means less voltage across the bridge
//!    for the same source, which means a smaller tanh argument. A model
//!    that ties bridge distortion to the amount of gain reduction has it
//!    backwards.
//!
//! # What is the part and what is the machine
//!
//! This crate is the bridge alone: the current it passes for a given
//! differential voltage and control current, that law's slope and its
//! antiderivative, and the resistance the law implies. The series and
//! shunt resistors that turn it into an attenuator, the sidechain that
//! produces the control current, the shaping network between them and the
//! transformers on either side all belong to the machine, because they
//! differ between the 2254 and the 33609 while the bridge itself does not.
//!
//! Solving the node equation of a particular divider is therefore the
//! caller's job. [`current`] and [`slope`] are what that solve needs.
//!
//! # What this models, and what it does not
//!
//! **What it models:** four diodes in a **closed ring**, **one junction per
//! arm**, both common nodes **floating**, **forward-biased** by an injected
//! control current. That is Neve's element in the 2254 and the 33609, and
//! within it the derivation is exact for ideal matched diodes.
//!
//! **What it does not model:** any other arrangement of four diodes. The
//! name "diode bridge" is a family name and a bridge is a ring of four, so
//! the name was never the thing that was wrong; the wrong thing was the
//! assumption that any diode gain element would fit it.
//!
//! The EMI TG12413 is the case that showed it. Its four HS2051s are **two
//! branches of two diodes in series**, both the same way up, whose common
//! node is the +20 V supply rail rather than a floating one, and as drawn
//! they are in **reverse breakdown** rather than forward conduction. Six of
//! the thirteen rows in a side-by-side comparison of the two elements are
//! structural rather than differences of value, and the operating region is
//! the largest of them: breakdown is tunnelling below about 5 V and
//! avalanche above about 6 V, neither is the diode exponential, and neither
//! yields a hyperbolic tangent when two arms are put in opposition.
//!
//! **The shortest way to see it, on the reading most generous to this
//! crate.** Suppose the TG's diodes are forward-biased after all. Then the
//! law *is* a tanh, because two junctions in series per arm still subtract
//! to a logarithm of a current ratio — but with **twice the constant**:
//!
//! ```text
//! Neve, one junction per arm:   i = I · tanh( u / (2·η·V_T) )
//! TG,   two junctions per arm:  i = I · tanh( u / (4·η·V_T) )
//! ```
//!
//! For `tanh(a·sinθ)` the third-harmonic ratio is `a²/12`, and doubling the
//! thermal scale halves `a`. So **two junctions per arm doubles the constant
//! and gives four times less third harmonic at equal drive**. A crate with
//! `2·η·V_T` baked into it, as this one has, is wrong for that element by a
//! factor of two in the argument and four in the third harmonic — and that
//! is the *best* case, because on the reading the drawing actually supports
//! there is no tanh to be wrong about.
//!
//! **So the TG12413's element is a second, separate component**, and it is
//! `noob-electrical-components-diode-arm-pair`. Its law carries
//! junctions-per-arm and a bulk resistance and so contains this ring as the
//! corner `n = 1, r_b = 0`; that containment is asserted there against this
//! crate's own numbers, to the last bits an `f32` has. Containing this law
//! is not being it. Widening *this* crate to reach the other case would
//! have produced a part that describes neither circuit, so there are two
//! crates and each says what the other is.
//!
//! **And the constants are the trap, not the safeguard.** Both crates rest
//! on the same published `η` and `V_T`, so [`THERMAL_SCALE`] here and a
//! two-junction arm's `v_n` there are the same number, about 90.7 mV, for
//! different reasons — two arms each of one junction against two junctions
//! inside one arm — and they sit in different places in the two formulas.
//! Matching them and concluding the parts are the same is exactly the error
//! this section exists to prevent.
//!
//! # What is estimated
//!
//! The topology and the law are derived and exact for ideal matched
//! diodes. The **numbers are not measured**. `η` and `I_S` come from
//! Pines' fit to a 1N4148; the bridges here use 1N4153-class parts and no
//! datasheet reachable for those publishes a saturation current or an
//! ideality factor at all. They enter only through [`THERMAL_SCALE`],
//! which is a single calibratable constant rather than a structural
//! assumption, so a machine that has a level annotation to calibrate
//! against should calibrate against it and treat the default as a
//! starting point.
//!
//! Temperature is not modelled: `k` is proportional to absolute
//! temperature, so a 20 °C swing moves the law by about 7 %. Neither is
//! junction capacitance, which rises with forward bias and lifts the top
//! end as gain falls, nor reverse recovery.
//!
//! # Sources
//!
//! - AMS Neve, *33609/J Limiter Compressor Technical Handbook*, the
//!   D14–D17 bridge and the resistors around it.
//! - Neve, drawing D/10,022/A, the 2254's B185 card: D1–D4, HBX 31.
//! - C. V. Pines, "Real-Time Virtual Analog Modelling of Diode-Based
//!   VCAs", DAFx-25, Ancona 2025, pages 63–70, for the diode parameters,
//!   the odd-symmetry result reached independently for a symmetric
//!   bridge, and the recommendation to block DC either side of a diode
//!   gain element.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Thermal voltage `V_T = kT/q` at 300 K, in volts.
///
/// From Pines, who states 25.85 mV at 300 K. Proportional to absolute
/// temperature, which is the whole of why a real bridge drifts with it.
pub const THERMAL_VOLTAGE: f32 = 0.025_85;

/// Diode ideality factor `η`, dimensionless.
///
/// **Estimate as applied here.** Pines fits 1.755 to a 1N4148. The bridges
/// this models use 1N4153-class parts, and no reachable 1N4153 datasheet
/// publishes an ideality factor, so this is the nearest usable figure
/// rather than the right one.
pub const IDEALITY: f32 = 1.755;

/// Diode saturation current `I_S`, in amps.
///
/// **Estimate as applied here**, for the same reason as [`IDEALITY`]:
/// Pines' 1N4148 fit standing in for a 1N4153. Present for completeness;
/// the bridge law does not use it, because the tail current sets the
/// operating point.
pub const SATURATION_CURRENT: f32 = 2.520e-9;

/// The bridge's thermal scale `k = 2·η·V_T`, in volts.
///
/// This is the one constant the law actually depends on. It is the voltage
/// that puts the tanh argument at unity, so it sets both how hard the
/// bridge has to be driven to distort and, through `r = k / I`, how much
/// control current a given resistance needs.
///
/// Computed from [`IDEALITY`] and [`THERMAL_VOLTAGE`] rather than written
/// out, so it stays consistent if either is recalibrated. It works out to
/// about 90.7 mV.
pub const THERMAL_SCALE: f32 = 2.0 * IDEALITY * THERMAL_VOLTAGE;

/// Smallest control current treated as real, in amps.
///
/// Below this the bridge is open and its resistance is taken as infinite.
/// A real bridge at zero control current is not infinitely resistive, but
/// it is far above any shunt a machine puts across it, so the difference
/// does not reach the output.
pub const CONTROL_FLOOR: f32 = 1e-12;

/// Signal current through the bridge, in amps.
///
/// `u` is the differential voltage across it in volts, `control` the DC
/// control current in amps, and `k` the thermal scale, normally
/// [`THERMAL_SCALE`]. The law is `I · tanh(u / k)`, odd in `u` and
/// saturating at `±control`.
///
/// Taking `k` as an argument rather than reading the constant is
/// deliberate: it is the calibratable one, and a machine with a level
/// annotation to fit against should be able to pass its own.
#[inline]
pub fn current(u: f32, control: f32, k: f32) -> f32 {
    control * (u / k).tanh()
}

/// Slope of [`current`] with respect to `u`, in amps per volt.
///
/// `I/k · sech²(u/k)`, which is the conductance the bridge presents at
/// that operating point. A caller solving its own node equation needs
/// this for the Newton step; at `u = 0` it is `control / k`, the
/// reciprocal of [`small_signal_resistance`].
#[inline]
pub fn slope(u: f32, control: f32, k: f32) -> f32 {
    let sech = 1.0 / (u / k).cosh();
    control / k * sech * sech
}

/// Antiderivative of [`current`] with respect to `u`, in amp-volts.
///
/// `I · k · ln(cosh(u / k))`, which is what first-order antiderivative
/// antialiasing needs. It belongs here rather than in whatever applies the
/// antialiasing, because it is a property of this law: a caller cannot
/// write it without knowing the law, and if the law is ever recalibrated
/// the two must move together.
///
/// Evaluated through a stable form of `ln cosh`, since `cosh` overflows
/// around `|u/k| = 89` while `ln cosh` is merely large.
#[inline]
pub fn current_antiderivative(u: f32, control: f32, k: f32) -> f32 {
    control * k * ln_cosh(u / k)
}

/// `ln(cosh(x))`, without overflowing for large `|x|`.
///
/// For large `|x|`, `cosh x → e^{|x|}/2`, so `ln cosh x → |x| − ln 2`. The
/// exact rearrangement `|x| + ln1p(e^{−2|x|}) − ln 2` holds everywhere and
/// stays finite, where the direct form overflows once `cosh` does.
#[inline]
pub fn ln_cosh(x: f32) -> f32 {
    let a = x.abs();
    a + (-2.0 * a).exp().ln_1p() - core::f32::consts::LN_2
}

/// The bridge's small-signal resistance `r = k / I`, in ohms.
///
/// This is the differential resistance at the origin, which is what a
/// machine puts in its divider. Control currents below [`CONTROL_FLOOR`]
/// return [`f32::INFINITY`], the open bridge.
#[inline]
pub fn small_signal_resistance(control: f32, k: f32) -> f32 {
    if control <= CONTROL_FLOOR {
        f32::INFINITY
    } else {
        k / control
    }
}

/// The control current that gives a wanted small-signal resistance, in
/// amps.
///
/// The inverse of [`small_signal_resistance`]. Closed form, because the
/// law is, which is one of the practical advantages of a bridge over a
/// single shunt diode: there is no gain parameterisation to invert
/// numerically.
#[inline]
pub fn control_for_resistance(r: f32, k: f32) -> f32 {
    if r <= 0.0 { f32::INFINITY } else { k / r }
}

#[cfg(test)]
mod tests;
