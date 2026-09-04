//! The diode arm pair used as a gain element: two arms of series
//! junctions hanging from one common rail, opposed across the audio.
//!
//! Each arm is *n* diode junctions in series with a bulk resistance. Both
//! arms hang the same way up from a node that is a **supply rail** rather
//! than a floating one, a control source sinks a bias current down each of
//! them, and the audio appears across the two arms differentially, so the
//! signal current transfers from one arm to the other. The bias current
//! therefore sets a resistance, and whatever divider the machine puts
//! around it sets the gain.
//!
//! EMI used exactly this as the gain element of the TG12413, the dynamics
//! module of the Abbey Road TG consoles: four HS2051 diodes in two
//! branches of two, sharing the +20 V rail. As drawn all four have their
//! cathodes toward the rail and their anodes toward the transistors that
//! sink current through them, which means they conduct in **reverse
//! breakdown** rather than forward. EMI's own limiter lineage runs through
//! a product called the RS168 *Zener* Limiter, and the two companies that
//! have built recreations with access to the hardware both call this
//! element a zener limiter, so the drawing is probably right.
//!
//! # This is not the diode bridge, and the two must not be merged
//!
//! The neighbouring crate, `noob-electrical-components-diode-bridge`,
//! models a different part that also has four diodes in it: **a closed
//! ring**, two opposed pairs with **two floating common nodes**, **one
//! junction per arm**, **forward-biased** by a control current injected
//! into those floating nodes. That is Neve's attenuator in the 2254 and
//! the 33609, and because both its common nodes float, each pair is a
//! long-tailed pair and its law is a hyperbolic tangent.
//!
//! This part is none of those things. Four diodes is the whole of the
//! resemblance, and a side-by-side comparison of the two elements gives
//! thirteen rows of which six are structural rather than differences of
//! value: the arrangement, the junctions per arm, the common node, the
//! operating region, the balance mechanism and where the control enters.
//!
//! The bridge crate was written expecting this element to be its second
//! user. It is not one, and that is why there are two crates.
//!
//! ## What the two genuinely share
//!
//! Two numbers and a numerical convention, and all three are cited from
//! outside both crates rather than taken from one another:
//!
//! - [`THERMAL_VOLTAGE`], `kT/q` at 300 K. That is physics, true of every
//!   junction ever made, and it belongs to neither part.
//! - [`IDEALITY`], which both crates take from the same published fit to a
//!   1N4148 because neither models a diode with a reachable datasheet —
//!   1N4153-class parts there, the HS2051 here. Two independent borrowings
//!   of one estimate, each admitting the same weakness.
//! - Treating a bias current below a picoamp as an open circuit
//!   ([`CURRENT_FLOOR`]), which is a guard against dividing by nothing and
//!   not a property of any component.
//!
//! **A shared citation is not a shared component**, so this crate has no
//! dependency on that one and neither is built from the other.
//!
//! ## What only looks shared, which is the dangerous half
//!
//! - **The law.** Equation (G1) below and the bridge's tanh coincide only
//!   in the corner `n = 1, r_b = 0` — which *is* the bridge, so the
//!   coincidence says the general law contains the special one and nothing
//!   more. As drawn this part runs in breakdown with `r_b > 0`, where
//!   breakdown is tunnelling below about 5 V and avalanche above about
//!   6 V, neither is the diode exponential, and there is no tanh to share.
//!   [`DiodeArmPair::ring`] exists so that the containment is a testable
//!   claim rather than a paragraph.
//! - **The scale constants, which is the trap.** The bridge's thermal
//!   scale is `2·η·V_T`, about 90.7 mV, and this part's forward `v_n` at
//!   two junctions per arm is *also* `2·η·V_T`, about 90.7 mV. **Same
//!   number, different reason**: the bridge's factor of two is two arms in
//!   opposition each contributing one junction, and this part's is two
//!   junctions in series inside one arm. They also sit in different places
//!   in the two formulas — the bridge's scale is the whole denominator of
//!   its tanh argument, while `v_n` here is half of it. Anyone who matches
//!   the two constants and concludes the parts are the same is wrong by a
//!   factor of two in the argument and, since the third-harmonic ratio of
//!   `tanh(a·sinθ)` is `a²/12`, a factor of four in the third harmonic.
//! - **The Newton solve.** Both parts leave their machine an implicit node
//!   equation, and both machines solve it with a linear seed and one or
//!   two Newton corrections. But the bridge's law is explicit in voltage
//!   and is solved in `u`, while this one is explicit in current and is
//!   solved in `i`. That is a code pattern with the variable reversed, it
//!   belongs to whatever machine owns the divider, and it is in neither
//!   crate.
//!
//! # The law
//!
//! Model each arm as *n* junctions in series with a bulk resistance, two
//! arms in opposition across the differential audio, biased at *I* each
//! and so carrying `I + i` and `I − i`. Taking the junctions as ideal and
//! matched, each arm's voltage relative to the common rail is *n* times a
//! single junction's, and subtracting the two gives
//!
//! ```text
//! u(i) = 2·r_b·i + 2·V_n·artanh( i / I )        (G1)
//! ```
//!
//! Three circuits fall out of it by choosing two constants:
//!
//! | circuit | n | V_n | r_b | reduces to |
//! |---|---|---|---|---|
//! | the bridge crate's ring, forward | 1 | η·V_T | 0 | `i = I·tanh(u / 2ηV_T)` |
//! | this part, forward | 2 | 2·η·V_T | ≈0 | `i = I·tanh(u / 4ηV_T)` |
//! | this part, breakdown | 2 | knee scale, **estimate** | **> 0** | a soft knee onto a resistive floor |
//!
//! The operating region is a **choice the caller makes**, not a default
//! this crate hides, because the drawing that is the only primary evidence
//! for the part supports two readings of it and the reading changes the
//! sound. [`DiodeArmPair::breakdown`] is what the drawing shows and is the
//! [`Default`]; [`DiodeArmPair::forward`] is the generous reading, under
//! which the law is a tanh with twice the bridge's thermal scale.
//!
//! # Mismatch, and why this is written as a logarithm rather than an artanh
//!
//! EMI specify D1/D3 and D2/D4 as matched pairs on two separate drawings
//! and provide two adjust-on-test resistors to trim what is left, so the
//! balance between the two arms is something a factory adjusted by hand
//! and something that can be out. Writing the law as
//!
//! ```text
//! u(i) = 2·r_b·i + V_n·ln( (I_a + i) / (I_b − i) )
//! ```
//!
//! carries the two arm currents separately, becomes (G1) exactly when
//! `I_a == I_b`, and gives the even harmonics an unbalanced pair really
//! does make. The `artanh` form cannot express it at all. The bridge crate
//! does not model mismatch, which is a seventh difference between them.
//!
//! # What is the part and what is the machine
//!
//! This crate is the pair of arms alone: the voltage across it for a
//! signal current at a bias current, that law's slope, the small-signal
//! resistance it presents and the inverse of that resistance.
//!
//! The series resistance the source drives it through, the divider that
//! resistance forms with it, the sidechain that produces the bias current,
//! the coupling capacitors either side and the output ladder after it are
//! the machine. They differ from unit to unit while the part does not, so
//! solving a particular divider's node equation is the caller's job and
//! [`DiodeArmPair::voltage`] and [`DiodeArmPair::slope`] are what that
//! solve needs.
//!
//! # What is estimated
//!
//! **The topology is read off a drawing; the numbers are not measured, and
//! for this part none of them are published anywhere at all.** No factory
//! handbook, no specification and no measurement of the module this comes
//! from has ever been published, and the HS2051 has no reachable
//! datasheet, so:
//!
//! - [`IDEALITY`] is a fit to a different diode, as it is in the bridge
//!   crate, and it enters only through [`JUNCTION_SCALE`].
//! - [`V_N_BREAKDOWN`] has no source at all. Breakdown is not the diode
//!   exponential, so the forward figure does not carry over, and this is a
//!   calibration knob with a plausible starting value.
//! - [`BULK_RESISTANCE`] is an order of magnitude inferred from the
//!   drawing rather than a measurement: a fixed 24 Ω sits on one branch
//!   against two adjust-on-test resistors in parallel on the other, which
//!   is what a designer builds to trim the balance between two branches
//!   that must carry the same current, and you trim against ohms because
//!   ohms is what a device in breakdown presents.
//!
//! A machine with a level annotation to calibrate against should calibrate
//! against these and treat the defaults as starting points. This part's
//! module has none, which is the difference between it and the bridge.
//!
//! # What is not modelled
//!
//! Temperature, junction capacitance, reverse recovery and the part's own
//! noise. Temperature is the interesting exclusion: a forward junction, a
//! zener below 5 V and an avalanche device above 6 V have three different
//! signs of coefficient, and since the device is unidentified the sign is
//! unknown. Modelling it would mean inventing it.
//!
//! # Sources
//!
//! - EMI, drawing TG12413-D101: D1–D4 HS2051 in two series branches on the
//!   +20 V rail, the matched-pair callouts, R14's 20 kΩ series arm and
//!   R16's 24 Ω against the two adjust-on-test resistors.
//! - Chandler Limited, who build this circuit under licence from EMI, on
//!   the RS168 Zener Limiter lineage and on "a rarely seen diode network".
//! - Waves, who modelled the module jointly with Abbey Road Studios and
//!   had the console, naming the element a "Zener diode limiter" three
//!   times in one user guide.
//! - C. V. Pines, "Real-Time Virtual Analog Modelling of Diode-Based
//!   VCAs", DAFx-25, Ancona 2025, pages 63–70, for [`IDEALITY`] and
//!   [`THERMAL_VOLTAGE`] and for the odd-symmetry result reached
//!   independently for a symmetric element.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Thermal voltage `V_T = kT/q` at 300 K, in volts.
///
/// From Pines, who states 25.85 mV at 300 K. This is physics rather than a
/// property of any part, which is why it is written here instead of taken
/// from the diode-bridge crate that quotes the same figure from the same
/// paper. It is proportional to absolute temperature, which is the whole
/// of why a real element drifts with it.
pub const THERMAL_VOLTAGE: f32 = 0.025_85;

/// Diode ideality factor `η`, dimensionless.
///
/// **Estimate as applied here.** Pines fits 1.755 to a 1N4148. This part
/// is built from HS2051s, for which no datasheet is reachable and which
/// publish neither an ideality factor nor a saturation current, so this is
/// the nearest usable figure rather than the right one. The diode-bridge
/// crate borrows the same figure for a 1N4153 and admits the same
/// weakness; the two borrowings are independent and neither is evidence
/// for the other.
pub const IDEALITY: f32 = 1.755;

/// `η·V_T` for **one** junction, in volts: about 45.4 mV.
///
/// The scale of a single junction, which is what makes junctions-per-arm a
/// parameter rather than a constant. Note that it is *half* the
/// diode-bridge crate's thermal scale, and that the bridge's `2·η·V_T`
/// counts two arms while [`DiodeArmPair::forward`] at `n = 2` counts two
/// junctions inside one arm. The numbers coincide; the reasons do not.
pub const JUNCTION_SCALE: f32 = IDEALITY * THERMAL_VOLTAGE;

/// Junctions per arm as EMI drew it: four diodes in two branches.
pub const N_JUNCTIONS: u32 = 2;

/// The knee scale of one arm in reverse breakdown, in volts.
///
/// **Estimate with no source at all.** The forward figure follows from a
/// published ideality and thermal voltage; this one follows from nothing,
/// because breakdown is tunnelling below about 5 V and avalanche above
/// about 6 V and neither is the diode exponential. 120 mV is a plausible
/// starting value and a calibration knob.
pub const V_N_BREAKDOWN: f32 = 0.120;

/// The bulk resistance of one arm in breakdown, in ohms.
///
/// **A hint, not a measurement.** A device in breakdown presents a bulk
/// resistance in the ohms to tens of ohms, and this 24 Ω is the fixed
/// balance resistor EMI put on one branch against two adjust-on-test
/// resistors in parallel on the other. You trim against ohms because ohms
/// is what the element presents, so the drawing gives the order of
/// magnitude and nothing finer.
///
/// It is named for what it is to the part rather than for the resistor it
/// was inferred from: that resistor is the machine, and the machine does
/// not live in this crate.
pub const BULK_RESISTANCE: f32 = 24.0;

/// Below this bias current the pair is treated as an open circuit, in
/// amps.
///
/// At 1 pA a pair in breakdown presents about 2.4 × 10¹¹ Ω, which is far
/// above any divider a machine will put around it, so the difference
/// cannot reach an output and a linear seed computed from it would only
/// lose precision. The diode-bridge crate makes the same choice at the
/// same value; it is a numerical convention rather than a shared property.
pub const CURRENT_FLOOR: f32 = 1e-12;

/// How close to an arm's bias current the signal current may come.
///
/// The logarithm form needs this guard on each arm separately, because
/// with mismatch the two ends are not at the same place.
const HEADROOM: f32 = 1e-6;

/// Two arms of series diode junctions on a common rail, opposed across the
/// audio.
///
/// The bias current sets the resistance the pair presents, so **no bias
/// current means an open circuit**. Note what that makes of the
/// distortion: a pair carrying no current cannot bend a waveform, so this
/// part is transparent when it is idle and dirtiest when it is working
/// hardest. That is the opposite of the diode bridge, whose forward-biased
/// ring distorts *less* as its control current rises, and it is the most
/// audible consequence of the two parts being different.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiodeArmPair {
    /// `V_n` of (G1): the knee scale of one arm, in volts.
    pub v_n: f32,
    /// `r_b` of (G1): the bulk resistance of one arm, in ohms.
    pub r_b: f32,
    /// Arm imbalance as a fraction of the bias current, 0 to 1: the arms
    /// carry `I·(1 + m)` and `I·(1 − m)`.
    pub mismatch: f32,
}

impl Default for DiodeArmPair {
    /// The pair as EMI drew it: two junctions per arm, in breakdown.
    fn default() -> Self {
        DiodeArmPair::breakdown()
    }
}

impl DiodeArmPair {
    /// One junction per arm, forward, with no bulk term — **which is not
    /// this part**.
    ///
    /// It is the arrangement `noob-electrical-components-diode-bridge`
    /// models, expressed in this crate's law, and (G1) reduces to that
    /// crate's `i = I·tanh(u / 2ηV_T)` exactly here. It exists so a caller
    /// can assert that containment against the shipped bridge crate and
    /// find it holds to the last bits an `f32` has, which turns "the
    /// general law contains the special one, and a crate with the special
    /// constant baked in cannot serve this part" into a test rather than
    /// an argument.
    ///
    /// Do not reach for it to model a bridge. Use the bridge crate, which
    /// says what a bridge is and carries a bridge's own sources.
    pub fn ring() -> Self {
        DiodeArmPair {
            v_n: JUNCTION_SCALE,
            r_b: 0.0,
            mismatch: 0.0,
        }
    }

    /// The forward reading: `n` junctions per arm, no bulk term.
    ///
    /// `n = 2` is the pair as drawn on the generous reading of the
    /// operating region, giving `i = I·tanh(u / 4ηV_T)` — the same
    /// function as the bridge's with the thermal scale doubled, and
    /// therefore four times less third harmonic at equal drive.
    pub fn forward(n: u32) -> Self {
        DiodeArmPair {
            v_n: n as f32 * JUNCTION_SCALE,
            r_b: 0.0,
            mismatch: 0.0,
        }
    }

    /// The breakdown reading, which is what the drawing shows.
    pub fn breakdown() -> Self {
        DiodeArmPair {
            v_n: V_N_BREAKDOWN,
            r_b: BULK_RESISTANCE,
            mismatch: 0.0,
        }
    }

    /// The two arm currents for a bias current, largest first.
    #[inline]
    fn arms(&self, i_bias: f32) -> (f32, f32) {
        let m = self.mismatch.clamp(0.0, 0.95);
        (i_bias * (1.0 + m), i_bias * (1.0 - m))
    }

    /// (G1): the differential voltage across the pair, in volts, for a
    /// signal current `i` at bias current `i_bias`, both in amps.
    ///
    /// With `mismatch == 0` this is `2·r_b·i + 2·V_n·artanh(i/I)` to the
    /// last bit, since `ln((I+i)/(I−i)) == 2·artanh(i/I)`.
    #[inline]
    pub fn voltage(&self, i: f32, i_bias: f32) -> f32 {
        let (a, b) = self.arms(i_bias);
        let num = (a + i).max(a * HEADROOM);
        let den = (b - i).max(b * HEADROOM);
        2.0 * self.r_b * i + self.v_n * (num / den).ln()
    }

    /// `du/di` in ohms, which a caller's Newton step needs.
    #[inline]
    pub fn slope(&self, i: f32, i_bias: f32) -> f32 {
        let (a, b) = self.arms(i_bias);
        let num = (a + i).max(a * HEADROOM);
        let den = (b - i).max(b * HEADROOM);
        2.0 * self.r_b + self.v_n * (1.0 / num + 1.0 / den)
    }

    /// The small-signal resistance the pair presents, in ohms.
    ///
    /// `2·r_b + 2·V_n / I` when the arms are matched. **The `2·r_b` term
    /// is a floor**, so a divider built around this part has a bounded
    /// loss and its gain reduction stops deepening however hard the bias
    /// current is driven. That is a property of breakdown operation, it is
    /// what the forward reading with `r_b = 0` does not have, and it is
    /// the mechanism behind a limiter that lets transients past.
    #[inline]
    pub fn resistance(&self, i_bias: f32) -> f32 {
        // The NaN test is not decoration: a bias current that has gone
        // non-finite must leave the pair open rather than fall through to
        // a logarithm, because `NaN <= x` is false on its own.
        if i_bias.is_nan() || i_bias <= CURRENT_FLOOR {
            return f32::INFINITY;
        }
        self.slope(0.0, i_bias)
    }

    /// The bias current that gives a wanted small-signal resistance, in
    /// amps, or `None` when the bulk floor puts that resistance out of
    /// reach.
    ///
    /// The inverse of [`resistance`](Self::resistance), and closed form,
    /// because that law is. The `None` has no counterpart in the bridge
    /// crate, whose `r = k / I` reaches every resistance above zero; here
    /// no bias current whatever brings the pair below `2·r_b`, and a
    /// caller asking for a gain reduction deeper than the floor allows
    /// needs an answer rather than an infinity.
    #[inline]
    pub fn current_for_resistance(&self, r: f32) -> Option<f32> {
        let m = self.mismatch.clamp(0.0, 0.95);
        let top = 2.0 * self.v_n / (1.0 - m * m);
        let bottom = r - 2.0 * self.r_b;
        if bottom <= 0.0 {
            None
        } else {
            Some(top / bottom)
        }
    }
}

#[cfg(test)]
mod tests;
