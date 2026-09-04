//! Blackmer's log-domain true-RMS detector: the part that reads the mean
//! of the square without ever squaring or taking a root.
//!
//! This crate is the **technique**, not "an RMS detector". The
//! distinction is the same one the Blackmer cell crate draws against
//! "VCA": a level detector that reads RMS could be a rectifier into a
//! squarer, a thermal converter, or this — a bilateral log converter
//! whose two junctions square the signal for free, a capacitor charged
//! through a junction and discharged by a constant current, and a square
//! root that is never computed because in the log domain it is a division
//! by two. Those share a specification, not an equation, and this crate
//! holds one of them.
//!
//! # The law
//!
//! The capacitor is charged through a junction whose current is the
//! antilogarithm of the difference between the log-domain signal and the
//! capacitor voltage, and discharged by a constant current. Writing the
//! stored level as `L` decibels and the instantaneous one as `L_inst`:
//!
//! ```text
//! dL/dt = (D/τ) · ( exp( (L_inst − L) / D ) − 1 )
//! ```
//!
//! That has an exact discrete solution for an input held over one sample
//! period, so [`LogRmsDetector::step`] costs one `exp` and one `ln` and is
//! unconditionally stable at any sample rate. There is no attack branch
//! and no release branch, **because the circuit has neither**.
//!
//! Three behaviours follow from that one equation, and all three are
//! published behaviour of the units built on it:
//!
//! - a falling signal is **rate-limited**, decaying a fixed number of
//!   decibels per second rather than exponentially;
//! - a rising one **attacks faster the bigger the step**, because a bigger
//!   step opens the charging junction harder;
//! - and the two **cannot be separated**. THAT Corporation, who make the
//!   descendant part, say in as many words that separate attack and
//!   release adjustments are not possible within the constraint of RMS
//!   response.
//!
//! # Crest factor, which is the point of the whole thing
//!
//! Because the averaging happens on the square, the level this detector
//! settles at depends on the *shape* of the waveform and not only on its
//! peak. A sine settles 3.01 dB below its peak, a square wave at its peak,
//! and anything peakier further down. That is what "true RMS" buys and it
//! is the reason a compressor built on this rides programme differently
//! from one built on a rectifier with a slow attack, whatever time
//! constants the rectifier is given.
//!
//! It is also why [`D_DB`] is exactly `10/ln 10` and not a measurement.
//! At any other value the averaging is an average of something that is not
//! the square, the detector reads a slightly different mean, and it is no
//! longer a true-RMS detector. See that constant for the derivation.
//!
//! # What this must not know, and why the line is there
//!
//! **No attack control, no release control, no threshold, no ratio and no
//! ballistics of any kind.** That boundary was not argued from first
//! principles; it was drawn by a real refusal, which is the strongest kind
//! this repository has.
//!
//! The dbx 160 has **no attack or release knobs at all**, because its
//! detector *is* its ballistics: the one time constant its two components
//! set produces its attack and its release together, and dbx's whole
//! argument for the box is that you cannot adjust them. The API 2500 has
//! **fourteen** ballistics positions, six attack and six release plus two
//! more, because on that unit the panel's ballistics are a separate stage
//! *after* the detector. Both contain this part. A component that carried
//! an attack control would be unusable by the first and redundant in the
//! second, so it carries none, and a caller that wants ballistics writes
//! them where they belong: outside.
//!
//! The time constant itself is a parameter of [`LogRmsDetector::set`]
//! rather than a constant here, and that is the same line seen from the
//! other side. The filter cannot run without one, but *which* one is a
//! capacitor and a current source on somebody's drawing — the dbx's are a
//! factory-matched pair marked as such — and those belong to the machine.
//!
//! # What is not modelled
//!
//! The detector's ripple. A real log converter's output carries the
//! excursion at every zero crossing, which is a real mechanism producing
//! real low-frequency third harmonic in the units built on it, but the
//! ripple that reaches the control port depends on what the machine does
//! between the two, so it emerges from a caller running this at audio rate
//! rather than being modelled here.
//!
//! # Who contains one
//!
//! Two units, on very different evidence, and the difference is recorded
//! rather than smoothed over:
//!
//! - the **dbx 160**, on dbx's own schematic, whose detector's time
//!   constant is set by R35 and C15, a factory-matched pair the drawing
//!   marks as one;
//! - the **API 2500**, whose detector is reported to be true RMS by
//!   reviewers and by API's own copy, but for which no schematic exists
//!   publicly and nothing below block level comes from API.
//!
//! So one drawing and one report. The report was not allowed to shape
//! anything here: what it contributed is the refusal above, which is a
//! statement about where the boundary is rather than about what is inside
//! it.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// The **thermal decibel**: how many decibels one junction voltage is
/// worth in the log domain, and the natural unit of this filter.
///
/// It sets both the release rate, `D/τ` decibels per second, and how much
/// faster a big step attacks than a small one.
///
/// # Why this is exactly `10/ln 10`
///
/// It is tempting to read it off two datasheet numbers — a thermal
/// voltage of 25.9 mV divided by a log constant of 6.1 mV/dB gives 4.246
/// — and that is wrong, because the two figures do not correspond. The
/// 6.1 is a measured typical that carries the junctions' ideality factor
/// with it; the 25.9 is bare `kT/q`. The quotient is about 2 % small.
///
/// Doing the algebra instead: the log converter puts `2·n·V_T·ln(I/I_S)`
/// on the charging junction, and that junction's own current is
/// `exp((v_in − v_C)/(n·V_T))`. The capacitor settles where the mean of
/// that current equals the constant discharge current, which is where
/// `⟨(I/I_S)²⟩ = exp(v_C/(n·V_T))` — **the true mean of the square** —
/// with the ideality and the temperature both cancelling, because the same
/// kind of junction does the logarithm and the averaging. The filter's
/// decibel unit is then `n·V_T / (n·V_T·ln10/10) = 10/ln 10`, exactly,
/// whatever the ideality and whatever the temperature.
///
/// That cancellation is why this crate has no junction constants in it at
/// all. A detector whose scale depended on the ideality would need them;
/// this one does not, and finding that out is most of the reason the
/// technique is worth having as a component rather than as a filter
/// somebody tunes.
///
/// So this is not a measurement to be rounded. It is the number that makes
/// the averaging an average of the square, and at any other value a sine
/// no longer settles 3.01 dB below its peak.
pub const D_DB: f32 = 10.0 / core::f32::consts::LN_10;

/// The level, in decibels relative to unit power, below which the
/// detector's input is floored.
///
/// This is a numerical floor and not a gate. It must sit far enough down
/// to be inaudible in the ripple the excursion at every zero crossing
/// produces, because that ripple is a real mechanism in the units built on
/// this part and a floor placed to tidy it away would be removing
/// behaviour rather than protecting arithmetic.
pub const FLOOR_DB: f64 = -200.0;

/// How far below the stored level, in thermal decibels, the instantaneous
/// one has to fall before the general update is replaced by its exact
/// asymptote.
///
/// Below this `exp(-q)` would overflow, and the asymptote is a straight
/// line in decibels, which is the release. Using it is not an
/// approximation: it is the limit of the same expression.
pub const RATE_LIMIT_Q: f64 = -40.0;

/// The release rate a time constant gives, in decibels per second.
///
/// `D/τ`, and it is a straight line rather than an exponential, which is
/// the signature of this detector and the thing a rectifier cannot
/// imitate at any setting.
#[inline]
#[must_use]
pub fn release_rate_db_s(tau_s: f32) -> f32 {
    D_DB / tau_s.max(1e-4)
}

/// Blackmer's true-RMS detector as a log-domain filter.
///
/// Feed it **power** — the square of the signal, or the sum of the squares
/// of several channels — one sample at a time, and read back the stored
/// level in decibels. See the crate documentation for the law, and for
/// what deliberately is not here.
#[derive(Clone, Copy, Debug)]
pub struct LogRmsDetector {
    /// Stored level in dB, kept in `f64` because it can sit a hundred
    /// decibels above the instantaneous one and the difference matters.
    level_db: f64,
    /// `exp(-h/τ)` for the sample period in force.
    a: f64,
    /// `(D/τ)·h`, the decibels one sample of rate-limited release costs.
    rate_step_db: f64,
    d: f64,
}

impl LogRmsDetector {
    /// A detector with a time constant, at a sample rate.
    ///
    /// The time constant is the caller's: it is a capacitor and a current
    /// source on the caller's drawing, not a property of the technique.
    #[must_use]
    pub fn new(tau_s: f32, sample_rate: f32) -> Self {
        let mut d = LogRmsDetector {
            level_db: FLOOR_DB,
            a: 0.0,
            rate_step_db: 0.0,
            d: D_DB as f64,
        };
        d.set(tau_s, sample_rate);
        d
    }

    /// Retune to a time constant and a sample rate.
    ///
    /// This is the only rate-dependent coefficient in the whole detector,
    /// which is the pleasant consequence of solving the filter exactly
    /// rather than discretising it by hand.
    pub fn set(&mut self, tau_s: f32, sample_rate: f32) {
        let h = 1.0 / sample_rate.max(1.0) as f64;
        let tau = tau_s.max(1e-4) as f64;
        self.a = (-h / tau).exp();
        self.rate_step_db = self.d / tau * h;
    }

    /// Forget the stored level.
    pub fn reset(&mut self) {
        self.level_db = FLOOR_DB;
    }

    /// The stored level, in decibels relative to unit power.
    #[inline]
    #[must_use]
    pub fn level_db(&self) -> f32 {
        self.level_db as f32
    }

    /// One sample of the detector's power input.
    ///
    /// `power` is the square of the signal, not the signal. Passing an
    /// amplitude would halve every decibel this returns and would still
    /// look plausible, so it is worth being sure.
    #[inline]
    pub fn step(&mut self, power: f32) -> f32 {
        let inst = if power > 1e-20 {
            10.0 * (power as f64).log10()
        } else {
            FLOOR_DB
        };
        let q0 = (inst - self.level_db) / self.d;
        if q0 < RATE_LIMIT_Q {
            // The charging junction is shut; the capacitor is discharged
            // by the constant current alone and the level falls along a
            // straight line. This is the exact asymptote of the line
            // below, not an approximation of it, and it is also what keeps
            // `exp(-q0)` from overflowing after loud material stops.
            self.level_db -= self.rate_step_db;
        } else {
            let m = 1.0 - (1.0 - (-q0).exp()) * self.a;
            self.level_db = inst + self.d * m.max(1e-300).ln();
        }
        self.level_db as f32
    }
}

#[cfg(test)]
mod tests;
