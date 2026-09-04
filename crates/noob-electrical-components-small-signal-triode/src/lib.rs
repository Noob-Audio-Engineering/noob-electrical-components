//! The small-signal triode gain stage: half a 12AX7-class double triode in
//! a class-A common-cathode connection, and the saturating law it obeys.
//!
//! This is the ordinary preamp valve. It has a sharp-cutoff grid, so it
//! amplifies by a fixed amount until it runs out of swing and then bends;
//! it is not a gain control. Two of these in series with variable negative
//! feedback between them are the whole audio path of a Universal Audio 610,
//! and the same stage is behind every valve preamp of that family.
//!
//! # This is not the remote-cutoff triode
//!
//! The other valve in this repository is
//! `noob-electrical-components-remote-cutoff-triode`, the gain element of
//! the variable-mu family, and the two are **different components with
//! different functional forms**. Neither can serve for the other, and the
//! reason is not that they want different numbers.
//!
//! A remote-cutoff valve is wound with a varying grid pitch, so that
//! different parts of its grid stop conducting at different bias voltages.
//! Its transconductance therefore falls away in a long shallow tail: tens
//! of decibels of gain over tens of volts of bias, with no bias at which it
//! simply switches off. That tail **is** the compressor. The control
//! voltage moves the grid bias, the bias moves the gain, and the valve is
//! the gain element.
//!
//! The law here does the opposite, by construction:
//!
//! - **Its gain does not depend on its bias at all.** [`transfer`] divides
//!   by the slope at the bias point, so the small-signal gain at the
//!   operating point is exactly one for every bias. Moving the bias changes
//!   the *asymmetry* of the curve and nothing else. There is no bias at
//!   which this stage is 20 dB down, so a control voltage applied to it
//!   would have nothing to do. The tests assert this over the whole usable
//!   bias range, and it is the sharpest single statement of the difference.
//! - **Its shape is fixed.** One exponent describes the entire curve, and
//!   the knee closes within a couple of units of drive. A remote-cutoff
//!   characteristic is the opposite shape: the knee never closes, because
//!   the tail is the point.
//!
//! So no choice of [`Triode::bias`] and [`Triode::knee`] turns this into
//! that, and no refit will, because the disagreement is in the functional
//! form. This repository's README once claimed that a variable-mu unit
//! would want the 610's tube stage. That claim was wrong, it has been
//! corrected there, and this paragraph is the correction restated where
//! somebody reaching for the wrong crate would read it.
//!
//! # The law
//!
//! The saturating shape is Yeh, Abel and Smith's tanh-like family,
//!
//! ```text
//! S(v) = v / (1 + |v|^n)^(1/n)
//! ```
//!
//! which is odd, bounded by ±1, and closes on that bound the harder the
//! larger `n` is. A valve stage is not centred on that curve's inflection:
//! it sits at a bias point somewhere up the bend, which is what makes a
//! single-ended triode asymmetric. So the stage is the family evaluated
//! about the bias, referred back to zero and normalised:
//!
//! ```text
//! T(v) = ( S(v + b) − S(b) ) / S'(b)
//! ```
//!
//! Three properties follow, and all three are what the part is for:
//!
//! 1. **It rests at zero.** Subtracting `S(b)` removes the operating-point
//!    offset, so silence in is silence out and no DC has to be blocked
//!    afterwards.
//! 2. **The bias sets the asymmetry, not the gain.** The curve bends more
//!    on one side of the operating point than the other, so a symmetric
//!    grid swing gives an asymmetric plate swing and the second harmonic
//!    dominates — the published character of a single-ended triode stage.
//!    Dividing by `S'(b)` is what keeps the level out of it, which matters
//!    to any machine that walks the bias about: a supply that sags under a
//!    loud passage should change the *colour* for a moment, not the volume.
//! 3. **The knee exponent is the only other freedom.** `n` near 2 gives a
//!    soft, tanh-ish bend; larger `n` closes the curve onto its asymptote
//!    more abruptly, which is how an output stage with less headroom than
//!    the one before it is voiced.
//!
//! # What is the part, and what is the machine
//!
//! The part is the curve: [`transfer`], and the [`Triode`] pair of numbers
//! that fixes it.
//!
//! The machine is everything that decides how hard the curve is driven and
//! how it is evaluated: the amplitude scale a stage saturates at, the
//! feedback a gain switch trades against attenuation, the supply sag that
//! walks the bias, the table that picks one revision's bias over another's,
//! the oversampling, and every filter around it. Those differ from unit to
//! unit while the valve does not.
//!
//! ## The antiderivative is deliberately not here
//!
//! `S` has an elementary antiderivative only for integer `n`, and the
//! exponents a real voicing uses are not integers, so a machine applying
//! first-order antiderivative anti-aliasing has to tabulate it. That table
//! is not in this crate. Anti-aliasing is a technique rather than a part,
//! which is this repository's standing line, and what would move is an
//! interpolation scheme and its error budget rather than a law.
//!
//! This is the opposite decision from the diode bridge, whose
//! antiderivative *does* live in its crate, and the difference is the
//! reason: there the antiderivative is a closed form of the law that a
//! caller could not write without knowing the law, so the two must move
//! together. Here the closed form does not exist. [`s_curve`] and
//! [`s_slope`] are public exactly so that a machine can integrate the law
//! numerically without keeping a second copy of it.
//!
//! # What is estimated
//!
//! **All of it, in the sense that matters.** This is a shape fitted to
//! published behaviour, not a device equation. There is no plate voltage
//! here, no amplification factor, no load line and no plate current: it is
//! not Koren's law, not Dempwolf and Zölzer's, not a Child-Langmuir fit.
//! Anything that needs a plate curve needs a different model, and this
//! crate will not become one by having parameters added to it.
//!
//! What it does reproduce is the character the sources agree on for a
//! single-ended triode gain stage: a decaying harmonic series dominated by
//! the second, distortion that grows with level, and a knee rather than a
//! clip. The tests assert those three and nothing stronger, because nothing
//! stronger is published: no independent measurement of a 610's own
//! transfer curve exists.
//!
//! **No bias or knee value ships here.** A voicing's numbers are the
//! machine's, and they are estimates in their own right wherever a plug-in
//! uses them.
//!
//! # Sources
//!
//! - Yeh, D. T., Abel, J. S., Smith, J. O., "Simplified, Physically-Informed
//!   Models of Distortion and Overdrive Guitar Effects Pedals", DAFx-07,
//!   Bordeaux, for the `x / (1 + |x|^n)^(1/n)` family.
//! - Blencowe, M., "Designing Valve Preamps for Guitar and Bass", chapter 1,
//!   for the common gain stage: unequally spaced grid curves, distortion
//!   dominated by the second harmonic, distortion proportional to level,
//!   and the self-rectification that walks the operating point.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// `S(v) = v / (1 + |v|^n)^(1/n)`: the saturating family the stage is built
/// from.
///
/// Odd in `v`, bounded by ±1, and unity slope at the origin. `knee` is the
/// `n` above: it does not change where the curve saturates, only how
/// abruptly it gets there.
///
/// This is public because a machine integrating the law for anti-aliasing
/// needs it; see the note in the crate documentation.
#[inline]
pub fn s_curve(v: f32, knee: f32) -> f32 {
    let a = v.abs().powf(knee);
    v / (1.0 + a).powf(1.0 / knee)
}

/// `S'(v) = (1 + |v|^n)^(−(n+1)/n)`: the slope of [`s_curve`].
///
/// One at the origin and falling monotonically with `|v|`, which is the
/// stage running out of swing.
#[inline]
pub fn s_slope(v: f32, knee: f32) -> f32 {
    let a = v.abs().powf(knee);
    (1.0 + a).powf(-(knee + 1.0) / knee)
}

/// The stage: `T(v) = (S(v + b) − S(b)) / S'(b)`.
///
/// `v` is the grid swing about the operating point, `bias` how far up the
/// bend that point sits, and `knee` the exponent of [`s_curve`].
///
/// Zero at rest and unity small-signal gain at the operating point **for
/// every bias**, which is the property that separates this valve from a
/// remote-cutoff one; see the crate documentation.
#[inline]
pub fn transfer(v: f32, bias: f32, knee: f32) -> f32 {
    (s_curve(v + bias, knee) - s_curve(bias, knee)) / s_slope(bias, knee)
}

/// The slope of [`transfer`] with respect to `v`: `S'(v + b) / S'(b)`.
///
/// The stage's incremental gain at a drive of `v`, which is one at the
/// operating point and falls as the stage bends. A caller solving a
/// feedback loop around the stage needs it for the Newton step; it is also
/// what makes the "bias does not move the gain" property directly
/// measurable rather than argued.
#[inline]
pub fn transfer_slope(v: f32, bias: f32, knee: f32) -> f32 {
    s_slope(v + bias, knee) / s_slope(bias, knee)
}

/// One triode gain stage: where it is biased and how hard its knee is.
///
/// These are the two numbers that fix the curve, and they are the whole of
/// the part. What amplitude the machine drives the curve at, and which
/// revision of a unit uses which pair, belong to the machine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triode {
    /// How far up the bend the operating point sits.
    ///
    /// Zero is the symmetric case, which makes no even harmonics at all.
    /// Raising it tips the curve so the second harmonic dominates, which is
    /// the audible signature of a single-ended stage. A machine may walk
    /// this about with a sagging supply without changing the stage's gain.
    pub bias: f32,
    /// The knee exponent `n` of [`s_curve`].
    ///
    /// Around 2 is a soft, tanh-like bend; larger values close onto the
    /// asymptote more abruptly. It sets how the stage overloads, not where.
    pub knee: f32,
}

impl Triode {
    /// A stage biased at `bias` with knee exponent `knee`.
    pub const fn new(bias: f32, knee: f32) -> Self {
        Triode { bias, knee }
    }

    /// [`transfer`] at this stage's bias and knee.
    #[inline]
    pub fn shape(&self, v: f32) -> f32 {
        transfer(v, self.bias, self.knee)
    }

    /// [`transfer`] with the bias moved, for a machine whose supply sags
    /// under load and walks the operating point about.
    ///
    /// Separate from [`Triode::shape`] because the moved bias is the
    /// machine's, sample by sample, while [`Triode::bias`] is the stage's
    /// resting one.
    #[inline]
    pub fn shape_at(&self, v: f32, bias: f32) -> f32 {
        transfer(v, bias, self.knee)
    }

    /// [`transfer_slope`] at this stage's bias and knee.
    #[inline]
    pub fn slope(&self, v: f32) -> f32 {
        transfer_slope(v, self.bias, self.knee)
    }
}

#[cfg(test)]
mod tests;
