//! The audio transformer's low end: the roll-off its magnetising inductance
//! puts under the band, and the flux its core can carry.
//!
//! A transformer works by not storing the signal. Current in the primary
//! magnetises the core, the changing flux induces a voltage in the
//! secondary, and what is wanted is for as little of the primary current as
//! possible to go into the magnetisation itself. Both behaviours modelled
//! here are that arrangement failing at the bottom of the band, in the two
//! different ways it can:
//!
//! - **It runs out of reactance.** The magnetising inductance sits across
//!   the source, and its impedance falls with frequency, so below some
//!   corner it shorts the signal out. That is [`Rolloff`], and it is
//!   linear: it happens at every level.
//! - **It runs out of core.** Flux is the integral of the applied voltage,
//!   so a low note puts far more flux through the core than a high one at
//!   the same level, and past some amount the core cannot carry any more.
//!   That is [`Core`], and it is a nonlinearity: it happens only when the
//!   signal is both loud and low.
//!
//! The second is why transformer distortion is a low-frequency
//! phenomenon — Paiva and colleagues measured it "at low frequencies only,
//! below about 100 Hz for the Fender and 30 Hz for the Hammond
//! transformer" — and why a model that ties transformer distortion to level
//! alone has it half right at best.
//!
//! # What is the part, and what is the machine
//!
//! The part is a corner, a Q, a flux limit and the law that limit obeys.
//!
//! The machine is which corner a given revision uses, and the filters it
//! builds from them. This crate designs no filter and runs no state: a
//! `Rolloff` is a description of an analogue response, and the sample rate
//! it has to be realised at, the topology that survives a 6 Hz corner at
//! 192 kHz, and the denormal handling are the caller's, because they are
//! properties of the arithmetic rather than of the part. Filters are
//! infrastructure in this repository, and that line is what keeps a
//! component crate from turning into a DSP library.
//!
//! # What is not here: the top end
//!
//! A real transformer rolls off at the top as well, from leakage
//! inductance and winding capacitance, and the units this part was drawn
//! from do model that. It has deliberately not been lifted, for two
//! reasons.
//!
//! Only one of the two units that share this component models a top-end
//! roll-off at all, so there is no second implementation to reconcile a
//! shape against; and the numbers that one uses were fitted to the response
//! of a whole chain, resamplers and anti-aliasing included, rather than
//! read off a transformer. They are a machine's calibration wearing a
//! part's name. A shape drawn from one implementation is usually the wrong
//! shape for the second, and while that argument no longer decides whether
//! a component exists, it still decides how one is drawn.
//!
//! The core is here on one user, which looks like the same situation and is
//! not. Its law is not a corner somebody tuned; it is the standard
//! flux-saturation approximation, and it is unambiguously this part rather
//! than the machine around it. Leaving it outside would also have had a
//! transformer's core borrowing a *valve's* saturating curve from
//! `noob-electrical-components-small-signal-triode`, because the two
//! happen to share an algebraic family, and a wrong dependency is worse
//! than a part drawn from one user.
//!
//! # What is estimated
//!
//! The roll-off's *form* is exact — a magnetising inductance across a
//! source is a single pole, and a pole pair is what a blocking network adds
//! to it — but **no corner ships here**, because a corner is a property of
//! one wound part and every unit's is different.
//!
//! The core's law is an approximation with a name: integrate, saturate,
//! subtract. It is **not** a hysteresis model. There are no minor loops, no
//! remanence, no coercivity and no eddy-current or hysteresis losses, so it
//! cannot show the memory a real core has: drive it hard and stop, and it
//! forgets immediately, where a real core does not. What it does get right
//! is the thing that is audible in a preamp — that saturation depends on
//! flux, so it arrives at low frequencies first. Anything needing the
//! memory wants Jiles-Atherton or a wave-digital core, not this.
//!
//! [`Core::KNEE`] is this crate's own number and is documented at its
//! definition.
//!
//! # Sources
//!
//! - Paiva, Pakarinen, Välimäki and Tikander, "Real-Time Audio Transformer
//!   Emulation for Virtual Tube Amplifiers", EURASIP Journal on Advances in
//!   Signal Processing, 2011: a wave-digital transformer on
//!   gyrator-capacitor theory, and the measurement that these distort at low
//!   frequencies only.
//! - Holters and Zölzer, DAFx-16: inductors and transformers with the
//!   magnetisation following a hysteresis curve, which is the model this
//!   crate deliberately is not.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::f32::consts::PI;

/// How many poles a [`Rolloff`] has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Poles {
    /// One pole: the magnetising inductance across the source, alone.
    /// 6 dB per octave, and −3.01 dB at the corner.
    One,
    /// A pole pair: the magnetising inductance with a second corner close
    /// enough to it to matter, from a blocking capacitor or the network on
    /// the far winding. 12 dB per octave, and how it approaches that
    /// depends on [`Rolloff::q`].
    Two,
}

/// A transformer winding's low-frequency roll-off.
///
/// Below the corner the magnetising inductance's impedance has fallen far
/// enough that it shunts the signal instead of transforming it. This type
/// is the *description* of that response; realising it at a sample rate is
/// the caller's job.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rolloff {
    /// The corner, Hz: where the winding's reactance has fallen to the
    /// resistance it works against.
    pub hz: f32,
    /// Quality factor of the pole pair.
    ///
    /// Below 0.707 the two poles pull apart and the response leans over
    /// early; at 0.707 it is maximally flat; above it there is a lift just
    /// above the corner before the fall, which is what a transformer that
    /// sounds as though it has a little weight at the bottom is doing.
    ///
    /// **Meaningless when [`Rolloff::poles`] is [`Poles::One`]**, where it
    /// reads 0.5 so that a caller who ignores `poles` and builds a pole
    /// pair anyway gets a critically damped one rather than a resonance.
    /// That is a guard, not a substitute: a critically damped pair is
    /// 12 dB per octave and −6 dB at the corner, and a single pole is
    /// neither.
    pub q: f32,
    /// One pole or two.
    pub poles: Poles,
}

impl Rolloff {
    /// A single-pole roll-off at `hz`.
    pub const fn one_pole(hz: f32) -> Self {
        Rolloff {
            hz,
            q: 0.5,
            poles: Poles::One,
        }
    }

    /// A two-pole roll-off at `hz` with quality factor `q`.
    pub const fn two_pole(hz: f32, q: f32) -> Self {
        Rolloff {
            hz,
            q,
            poles: Poles::Two,
        }
    }

    /// The corner a winding of `henries` working against `ohms` puts the
    /// roll-off at: `f = R / (2πL)`.
    ///
    /// This is where the part's corner actually comes from, and it is worth
    /// having even though no unit modelled here is specified this way,
    /// because it says what moves the corner: a bigger core or more turns
    /// lower it, and a heavier load raises it. A transformer's low end is a
    /// property of the winding *and* of what it is driven by, which is why
    /// the same transformer measures differently in two machines.
    pub fn from_winding(henries: f32, ohms: f32) -> Self {
        Rolloff::one_pole(ohms / (2.0 * PI * henries))
    }

    /// Magnitude of the analogue response at `hz`, as a linear ratio.
    ///
    /// One well above the corner, falling to zero at DC.
    pub fn magnitude(&self, hz: f32) -> f32 {
        if self.hz <= 0.0 {
            return 1.0;
        }
        let r = hz / self.hz;
        match self.poles {
            Poles::One => r / (1.0 + r * r).sqrt(),
            Poles::Two => {
                let rr = r * r;
                let real = 1.0 - rr;
                let imag = r / self.q.max(1e-4);
                rr / (real * real + imag * imag).sqrt()
            }
        }
    }

    /// [`Rolloff::magnitude`] in decibels.
    pub fn response_db(&self, hz: f32) -> f32 {
        20.0 * self.magnitude(hz).max(1e-20).log10()
    }
}

/// The transformer core: how much flux it can carry, and what happens to
/// the rest.
///
/// The machine hands this the flux, which it obtains by integrating the
/// signal; the core says how much of that never reaches the secondary. In
/// the linear region the block is transparent, so a machine can leave it in
/// the path at all times.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Core {
    /// The corner above which the machine's integrator is an integrator,
    /// Hz.
    ///
    /// The flux a caller hands to [`Core::excess`] comes from a leaky
    /// integrator — a one-pole low-pass — rather than a perfect one, because
    /// a perfect integrator has infinite gain at DC and a transformer does
    /// not. Above this corner it integrates and the flux falls as `1/f`,
    /// which is the whole of why saturation arrives at low frequencies
    /// first; below it the core simply follows the signal.
    ///
    /// It is carried here rather than in the machine because it is a
    /// property of the wound part, but the filter itself is the machine's:
    /// see the note on infrastructure in the crate documentation.
    pub integrator_hz: f32,
    /// The flux the core can carry, in whatever units the machine's
    /// integrator produces.
    ///
    /// This crate cannot give it webers, because the machine works in
    /// signal units rather than volts and turns. What it means is fixed by
    /// the law: at a flux of exactly this, the core is one decibel and a
    /// half into its knee.
    pub flux_limit: f32,
}

impl Core {
    /// The sharpness of the core's knee.
    ///
    /// The saturating shape is `S(x) = x / (1 + |x|^n)^(1/n)` with `n` =
    /// 4, which closes onto the limit appreciably harder than the
    /// hyperbolic tangent usually written for this approximation. It is
    /// this crate's own number and no source fixes it: a real core's knee
    /// depends on its material and its gap, and nobody publishes a B-H
    /// curve for the parts these units use. Four was chosen because the
    /// alternative — a tanh — leaves the core softening the signal well
    /// before it is in trouble, which spends the transformer's character on
    /// material that is not loud enough to have any.
    pub const KNEE: f32 = 4.0;

    /// A core that integrates above `integrator_hz` and carries
    /// `flux_limit` of flux.
    pub const fn new(integrator_hz: f32, flux_limit: f32) -> Self {
        Core {
            integrator_hz,
            flux_limit,
        }
    }

    /// The part of the flux the core cannot carry.
    ///
    /// Odd in `flux`, essentially zero while the core is inside its limit,
    /// and approaching `flux` less the limit once it is far outside it: past
    /// the knee the core carries the limit and no more, so everything
    /// further is excess.
    #[inline]
    pub fn excess(&self, flux: f32) -> f32 {
        let sat = self.flux_limit;
        let x = flux / sat;
        let a = x.abs().powf(Self::KNEE);
        flux - sat * (x / (1.0 + a).powf(1.0 / Self::KNEE))
    }

    /// What reaches the secondary: the signal, less the part of the flux
    /// the core could not carry.
    ///
    /// `signal` is the voltage across the primary and `flux` its integral,
    /// as the machine computed it. Keeping the two separate is the whole
    /// point of the approximation: the saturation is a function of the
    /// flux, which is why it depends on frequency, but what is subtracted
    /// is taken off the signal.
    #[inline]
    pub fn through(&self, signal: f32, flux: f32) -> f32 {
        signal - self.excess(flux)
    }
}

#[cfg(test)]
mod tests;
