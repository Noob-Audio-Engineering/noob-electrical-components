//! The remote-cutoff triode: the gain element of a variable-mu limiter.
//!
//! One triode section, as a pure function of grid and anode voltage with no
//! state at all. Anode current, both its partial derivatives,
//! transconductance, plate resistance and amplification factor, all from one
//! fitted law with a parameter set per valve type.
//!
//! # It is not an ordinary triode model with different numbers
//!
//! A remote-cutoff valve has its grid wound with varying pitch, so it
//! switches off progressively over tens of volts rather than collapsing over
//! a few, and its amplification factor is a function of bias rather than a
//! number. The triode models a preamplifier uses were fitted for
//! 12AX7-class valves, which have no such characteristic — Raffensperger
//! says so in as many words, *"Existing triode models were designed for
//! tubes like the 12AX7 which do not have the remote cutoff characteristic
//! of the 6386"* — and **the difference is in the functional form, not the
//! parameters**. That is why this is its own component rather than a
//! parameter set of somebody else's.
//!
//! One of those preamplifier models is in this workspace, as
//! `noob-electrical-components-small-signal-triode` behind the
//! `small-signal-triode` feature, and the two crates state the boundary from
//! their own sides. Its law is a fixed-shape curve whose small-signal gain
//! is the same at every bias, so there is no bias at which it is twenty
//! decibels down and a control voltage applied to it would have nothing to
//! do. This one's whole purpose is that its gain *is* its bias, and
//! `transconductance_falls_monotonically_over_the_published_interval` asserts
//! the complement of that crate's assertion. Neither can serve the other, and
//! the pair of tests is what says so.
//!
//! # The law
//!
//! ```text
//!                  p1 · Vak^p2
//! Ia = ───────────────────────────────────────────
//!      (p3 − p4·Vgk)^p5 · [ p6 + exp(p7·Vak − p8·Vgk) ]
//! ```
//!
//! `Ia` in amperes, `Vgk` and `Vak` in volts. Grid current is assumed
//! negligible, which holds while the grid stays negative. The expression
//! diverges where `(p3 − p4·Vgk)` reaches zero, at +5 V of grid for the
//! parameter sets here, so the grid is clamped well below that
//! ([`ValveParams::vgk_clamp`], and [`ValveParams::grid_singularity`] for
//! where the pole actually is).
//!
//! # One parameter is refitted, and this is the reason
//!
//! **The published fit is to plate *current*, and its *slope* is a separate
//! matter that was never constrained.** For a variable-mu stage the audio
//! *is* the slope, because gain is transconductance. As published the law is
//! 42 % low in transconductance at the valve's own tabulated operating
//! point, and its rate of fall dips and climbs again in a way the maker's own
//! logarithmic plot does not. Read against General Electric's plate
//! characteristics at 250 V it is 9.1 dB low at −50 V of grid and 37.3 dB low
//! at −70, which is where a limiter spends its loudest moments: a
//! remote-cutoff valve still passing half a milliamp at −70 V *is* the point
//! of the type, and the published fit has it at one hundredth of that.
//!
//! So `p8`, the exponential cut-off rate, moves from 0.2 to 0.131 87 with the
//! scale `p1` renormalised, refitted against **General Electric's own plate
//! characteristics, across the working range and in the right topology** —
//! one published source to another, never to an invention. The least-squares
//! cost falls from 20.05 to 0.09. Letting `p4` and `p5` move as well buys
//! 0.03 more and was declined, because one changed parameter with a reason is
//! easier to defend than four. Both sets ship —
//! [`ValveParams::GE_6386`] and [`ValveParams::GE_6386_AS_PUBLISHED`] — so the
//! correction can be measured against the manufacturer's curve rather than
//! asserted in the abstract.
//!
//! **How the original check missed it, which is the lesson.** The published
//! fit was validated against three points on the *transfer* characteristics,
//! where the whole family of curves is crushed into the bottom few per cent
//! of a linear current axis below −30 V. Read there, −50 V looks like "half
//! to one milliamp" and the truth is 1.6. A check made on a plot that cannot
//! resolve the region it is checking can hardly fail. The plate
//! characteristics give every grid voltage its own line and so resolve the
//! deep end; the tests assert both ends. The rule that follows belongs beside
//! any procedure for fitting these valves: **a law must be validated against
//! a plot that resolves deep cutoff on its own terms before it is trusted in
//! the region a limiter actually works in.** A logarithmic transconductance
//! axis, constant-parameter plate characteristics or a plate-resistance curve
//! all do that; a linear-axis transfer family does not.
//!
//! # The parameters are per valve type, and a second type has three conditions
//!
//! [`ValveParams`] is a set per type because **an exponent read off a
//! datasheet is not a stable quantity.** Fitting a stretched exponential
//! `gm(w) = gm0 · exp(−(w/V0)^n)` to one valve's transconductance across the
//! four operating conditions its maker plots gives n = 1.00, 0.84, 0.71 and
//! 0.59, moving monotonically with the supply, every fit good to under half a
//! decibel: one valve, one page, a factor of 1.7. Another valve's exponent
//! moves from 2.16 to 1.71 on nothing more than a change of anchor points
//! within its own single curve. The published figures that circulated during
//! that argument — 1.01, 1.10, 1.58, 1.71, 1.98, 2.16 — were each somebody's
//! honest reading, and none of them should be reused.
//!
//! So a second valve type must be fitted **by one documented procedure, using
//! the same class of anchor points, on curves measured in the same
//! topology.** Each clause was learned by getting it wrong:
//!
//! - **one documented procedure**, because the procedure moves the answer;
//! - **the same class of anchor points**, because a fit to interior points is
//!   not a fit to endpoints;
//! - **the same topology**, because one valve's published plot is a cascode
//!   connection whose section's plate floats at the next valve's cathode
//!   while another's is a single-section characteristic at a fixed plate
//!   voltage — and neither is the grounded-cathode stage a limiter actually
//!   runs.
//!
//! Without all three, two implementations agree on a number while disagreeing
//! about the curve, which is worse than not sharing at all, because it looks
//! like corroboration.
//!
//! # The accuracy floor, stated rather than implied
//!
//! **There is none for this valve, and there cannot be one from published
//! data.** Exactly one datasheet for the 6386 exists, General Electric's
//! ET-T1113, so there is no second manufacturer's curve to cross-check
//! against and no inter-source spread to bound the model with. What is
//! recorded instead is [`ValveParams::fit_residual_db`], a **fit residual**:
//! 0.89 dB RMS over nine readings taken by one person off one 1953 graph. A
//! residual says how well a curve was fitted, not how right the curve is, and
//! the two are not the same claim. Where a law's sources do disagree — two
//! manufacturers' transconductance curves for the same valve differing by 1.3
//! to 1.5 dB, say — the fit cannot be more accurate than that, and the floor
//! is the spread rather than the residual.
//!
//! # What is not here
//!
//! The part, and nothing around it. Sections in parallel is a circuit choice
//! and belongs to the caller as a scale factor; push-pull is the machine; the
//! cathode resistor and its bypass capacitor differ from unit to unit; the
//! common-mode control injection and its resistors, the time-constant
//! network, the rectifier and its dead zone are all sidechain. The valve does
//! not change when any of them does.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// One valve type's parameters: the eight of the law, the grid-axis
/// transform that expresses a same-curve replacement, the clamp, the
/// interelectrode capacitances and the residual of the fit that produced
/// them.
///
/// A set per type, for the reason the crate documentation gives at length:
/// the shape of these curves is condition-dependent, anchor-dependent,
/// procedure-dependent and maker-dependent, so no valve inherits another's
/// numbers. Adding a type means fitting it — by one documented procedure, on
/// the same class of anchor points, on curves measured in the same
/// topology — not scaling these.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValveParams {
    /// The valve this set was fitted for, and which fit it is.
    pub name: &'static str,
    /// Overall scale of the anode-current law, in amps. Not an independent
    /// quantity: it is renormalised whenever anything else in the law moves.
    pub p1: f32,
    /// Exponent of the plate voltage in the numerator.
    pub p2: f32,
    /// Constant term of the grid-voltage factor. The law diverges where
    /// `p3 − p4·Vgk` reaches zero; see [`Self::grid_singularity`].
    pub p3: f32,
    /// Grid coefficient of that factor.
    pub p4: f32,
    /// Exponent of that factor.
    pub p5: f32,
    /// Constant term of the exponential cut-off factor.
    pub p6: f32,
    /// Plate coefficient inside the exponential.
    pub p7: f32,
    /// **Grid coefficient inside the exponential: the rate of the exponential
    /// cut-off, and the one parameter refitted here.** It is the only part of
    /// the expression that was wrong: shallower than about −30 V the term is
    /// negligible and the power-law factor carries the curve, which is why a
    /// fit checked only in the shallow end looked right.
    pub p8: f32,
    /// Grid-axis scale, for a replacement valve that carries the same anode
    /// current at the same bias and a different transconductance.
    ///
    /// **This is a same-curve transform, not a fit.** It expresses "three
    /// quarters of the slope at the same operating point" as the same curve
    /// stretched along the grid axis. A genuinely different valve type gets
    /// its own `p1` to `p8` instead.
    pub grid_scale: f32,
    /// Grid-axis offset that goes with [`Self::grid_scale`], chosen to leave
    /// the published operating point where it was.
    pub grid_offset: f32,
    /// Highest grid-to-cathode voltage the law is evaluated at.
    ///
    /// The fit is only meaningful for a negative grid, and the expression
    /// blows up at [`Self::grid_singularity`], so this sits well below it.
    /// Above the clamp the current is frozen and the grid slope is zero,
    /// which is what a clamp means.
    pub vgk_clamp: f32,
    /// Grid-to-plate capacitance of one section, pF (datasheet).
    pub c_grid_plate_pf: f32,
    /// Input capacitance of one section, pF (datasheet).
    pub c_input_pf: f32,
    /// Output capacitance of one section, pF (datasheet).
    pub c_output_pf: f32,
    /// RMS residual of this set against the readings it was fitted to, in dB.
    ///
    /// **A fit residual, not a measured accuracy.** It says how well a curve
    /// was fitted. Where a valve has only one datasheet there is nothing to
    /// bound how right the curve is, and this figure must not be quoted as
    /// though there were.
    pub fit_residual_db: f32,
}

impl ValveParams {
    /// The General Electric 6386, **with the cut-off rate refitted against
    /// the datasheet the law was fitted to**.
    ///
    /// `p2` to `p7` are Raffensperger's published values. `p8` and the scale
    /// `p1` are not, and the crate documentation gives the argument. What the
    /// change buys, read off ET-T1113 page 5, lower figure, "AVERAGE PLATE
    /// CHARACTERISTICS, EACH SECTION", at 250 V of plate:
    ///
    /// | Vgk | GE | as published | here |
    /// |---|---|---|---|
    /// | −12 V | 18.26 mA | −1.23 dB | −0.08 dB |
    /// | −14 V | 14.30 | −0.85 | +0.30 |
    /// | −17 V | 11.55 | −1.30 | −0.14 |
    /// | −20 V | 8.85 | −1.01 | +0.17 |
    /// | −25 V | 7.10 | −2.04 | −0.78 |
    /// | −30 V | 5.14 | −1.88 | −0.41 |
    /// | −40 V | 3.61 | **−4.83** | −1.40 |
    /// | −50 V | 1.60 | **−9.14** | +2.04 |
    /// | −70 V | 0.60 | **−37.28** | −0.17 |
    ///
    /// A limiter's grids sit about 22 V down at rest and reach −70 V at the
    /// deepest limiting the published static curves show, so as published the
    /// model would spend its whole working range on the wrong part of its own
    /// valve law.
    ///
    /// **What the refit fixes and what it does not.** At the tabulated
    /// class-A₁ point — plate 100 V, cathode resistor 200 Ω, plate current
    /// 9.6 mA, so `Vgk` = −1.92 V — this set gives 9.78 mA against GE's
    /// 9.6 and 2606 µmho against GE's 4000, so the current is right and the
    /// transconductance is still 35 % low where the published fit was 42 %
    /// low. Over GE's own published interval, from that point to the −16 V
    /// where they tabulate 100 µmho, it falls 26.44 dB against a published
    /// 32.0. That gap is printed rather than closed: closing it would mean
    /// anchoring an exponential on two tabulated points at 100 V of plate and
    /// extrapolating it into a stage running at 216 to 230 V, which gives
    /// about 110 dB of control authority where the unit has 20 — not a small
    /// error but a model that could not work.
    ///
    /// The interelectrode capacitances are GE's own tabulated figures from the
    /// same sheet, carried as data because they are the valve's.
    pub const GE_6386: Self = Self {
        name: "GE 6386 (Raffensperger, p8 refitted to GE's plate characteristics)",
        p1: 4.539_9e-8,
        p2: 2.383,
        p3: 0.5,
        p4: 0.1,
        p5: 1.8,
        p6: 0.5,
        p7: -0.039_22,
        p8: 0.131_87,
        grid_scale: 1.0,
        grid_offset: 0.0,
        vgk_clamp: -0.5,
        c_grid_plate_pf: 1.2,
        c_input_pf: 2.0,
        c_output_pf: 1.1,
        fit_residual_db: 0.89,
    };

    /// Raffensperger's eight parameters exactly as published, kept so the
    /// correction can be measured rather than asserted.
    ///
    /// It is the only published fit of this valve that exists, and it
    /// reproduces the datasheet's *transfer* characteristics to within the
    /// width of the printed curve at three points across two decades of
    /// current. What it does not reproduce is the deep end or the slope, and
    /// the tests here assert both failures against GE's own figures, so
    /// anybody with the datasheet can check them.
    ///
    /// Its residual is recorded as what it measures: 12.95 dB RMS over the
    /// nine plate-characteristic readings, which is a statement about the
    /// published fit read against a curve it was not fitted to, not a
    /// judgement of the paper it comes from.
    pub const GE_6386_AS_PUBLISHED: Self = Self {
        name: "GE 6386 (Raffensperger as published)",
        p1: 3.981e-8,
        p8: 0.2,
        fit_residual_db: 12.95,
        ..Self::GE_6386
    };

    /// The JJ Electronic 6386 LGP, the modern replacement.
    ///
    /// JJ publish typical characteristics at the same operating point GE use
    /// — `Ua = 100 V, Rk = 200 Ω, Ia = 9.6 mA` — with `S = 3 mA/V` against
    /// GE's 4 mA/V and `μ = 18` against 17. So the two parts carry the **same
    /// plate current at the same bias** and differ in the **slope** by a
    /// factor of 0.75, which is 2.5 dB. A valve with the same current and
    /// three quarters of the transconductance is the same curve stretched
    /// along the grid axis by 0.75, with the offset chosen to leave the
    /// operating point where it was:
    ///
    /// ```text
    /// 0.75 · (−1.92 V) + offset = −1.92 V   →   offset = −0.48 V
    /// ```
    ///
    /// **That is an assumption about the shape away from the one published
    /// point, and it is stated rather than measured.** What it reproduces
    /// exactly is the published transconductance ratio and, by construction,
    /// whatever anode current the GE set gives at the bias where both
    /// datasheets quote the same figure. It is not a second fit, and it is not
    /// evidence about a second valve *type*: two datasheets quoting one
    /// operating point cannot fix a curve, which is the whole finding behind
    /// the per-type rule above.
    pub const JJ_6386_LGP: Self = Self {
        name: "JJ 6386 LGP (GE curve, grid axis stretched to JJ's published slope)",
        grid_scale: 0.75,
        grid_offset: -0.48,
        ..Self::GE_6386
    };

    /// Grid voltage at which the law's pole sits, `p3 / p4`.
    ///
    /// `(p3 − p4·Vgk)` reaches zero there and the expression blows up, so
    /// [`Self::vgk_clamp`] must stay well below it. +5 V for the sets here,
    /// which is five and a half volts above the clamp and far above anything
    /// a limiter's grids reach.
    pub fn grid_singularity(&self) -> f32 {
        self.p3 / self.p4
    }
}

/// A remote-cutoff triode section: a pure function of grid and plate
/// voltage, with no state at all.
///
/// Constructed from a [`ValveParams`] — one of the named sets, or one fitted
/// for a valve this crate does not carry yet under the three conditions the
/// crate documentation sets out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoteCutoffTriode {
    params: ValveParams,
}

impl Default for RemoteCutoffTriode {
    fn default() -> Self {
        Self::ge_6386()
    }
}

impl RemoteCutoffTriode {
    /// A section with an arbitrary parameter set.
    pub const fn new(params: ValveParams) -> Self {
        RemoteCutoffTriode { params }
    }

    /// The General Electric 6386, on the refitted set
    /// ([`ValveParams::GE_6386`]).
    pub const fn ge_6386() -> Self {
        Self::new(ValveParams::GE_6386)
    }

    /// The JJ Electronic 6386 LGP, the modern replacement
    /// ([`ValveParams::JJ_6386_LGP`]).
    pub const fn jj_6386_lgp() -> Self {
        Self::new(ValveParams::JJ_6386_LGP)
    }

    /// The parameter set this section runs on.
    pub const fn params(&self) -> &ValveParams {
        &self.params
    }

    /// Anode current of one section, in amps.
    #[inline]
    pub fn anode_current(&self, vgk: f32, vak: f32) -> f32 {
        self.slopes(vgk, vak).0
    }

    /// Anode current and both its partial derivatives:
    /// `(Ia, ∂Ia/∂Vgk, ∂Ia/∂Vak)`.
    ///
    /// One evaluation gives all three, because the derivatives share every
    /// expensive term with the current. A cathode solve wants both slopes at
    /// the same point, so returning them together halves the transcendental
    /// count of an inner loop.
    ///
    /// Above [`ValveParams::vgk_clamp`] the current is frozen and the grid
    /// slope is zero, which is what a clamp means.
    #[inline]
    pub fn slopes(&self, vgk: f32, vak: f32) -> (f32, f32, f32) {
        let p = &self.params;
        let vak = vak.max(1.0);
        let raw = p.grid_scale * vgk + p.grid_offset;
        let g = raw.min(p.vgk_clamp);
        let c = p.p6 + (p.p7 * vak - p.p8 * g).exp();
        let ia = p.p1 * vak.powf(p.p2) / ((p.p3 - p.p4 * g).powf(p.p5) * c);
        let d_vak = ia * (p.p2 / vak - p.p7 * (c - p.p6) / c);
        if raw > p.vgk_clamp {
            return (ia, 0.0, d_vak);
        }
        // d(ln Ia)/dg, times the chain rule for the stretched grid axis.
        let d_vgk = ia * (p.p4 * p.p5 / (p.p3 - p.p4 * g) + p.p8 * (c - p.p6) / c) * p.grid_scale;
        (ia, d_vgk, d_vak)
    }

    /// Transconductance `∂Ia/∂Vgk` of one section, in siemens.
    ///
    /// This is what a variable-mu stage's gain actually is, and it is the
    /// quantity the refit exists for: a law fitted to current alone gets it
    /// wrong without ever looking wrong.
    #[inline]
    pub fn transconductance(&self, vgk: f32, vak: f32) -> f32 {
        self.slopes(vgk, vak).1
    }

    /// Plate resistance `∂Vak/∂Ia` of one section, in ohms.
    ///
    /// 3974 Ω at the class-A₁ point on [`ValveParams::GE_6386`] against GE's
    /// tabulated 4250, which is the closest of the three tabulated
    /// quantities.
    pub fn plate_resistance(&self, vgk: f32, vak: f32) -> f32 {
        1.0 / self.slopes(vgk, vak).2
    }

    /// Amplification factor at a point: `gm · rp`, which for a remote-cutoff
    /// valve is a function of bias and not a number.
    ///
    /// **This is the one quantity the functional form cannot reproduce**, and
    /// it is recorded rather than hidden. Measured off GE's plate
    /// characteristics as the horizontal spacing of the grid curves at a fixed
    /// current — which is what an amplification factor *is*, and a far easier
    /// reading than a current near the baseline — the real valve runs 16.5
    /// near zero bias down to 5.8 at −30 V. That closes against GE's tabulated
    /// pair: 16.5 over a tabulated 4250 Ω is 3880 µmho against a tabulated
    /// 4000. This law gives 10.4 at the same point, because its `Vak^p2`
    /// numerator over a grid-only denominator forces `μ ∝ Vak` while the real
    /// valve's falls as the plate rises. No choice of the eight parameters can
    /// do both.
    ///
    /// A caller whose audio path is a difference of plate currents into a
    /// fixed plate voltage never divides a load against a plate resistance and
    /// so never reads this. One that does should know the figure first.
    pub fn mu(&self, vgk: f32, vak: f32) -> f32 {
        let (_, dg, da) = self.slopes(vgk, vak);
        dg / da
    }
}

#[cfg(test)]
mod tests;
