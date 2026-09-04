//! Tests for the transformer's low end.
//!
//! No independent measurement of any of the transformers these units use is
//! published — the parts are custom-wound, and where a type is named the
//! manufacturer is long gone — so nothing here asserts a measured corner or
//! a measured saturation level, and no corner ships in this crate to assert
//! one against. What these assert are the **exact properties of the
//! response** a pole or a pole pair has, which are arithmetic, and the
//! **shape** of the core's behaviour, which is where the published record
//! reaches: Paiva and colleagues measured transformer distortion "at low
//! frequencies only, below about 100 Hz for the Fender and 30 Hz for the
//! Hammond transformer", and that is a statement about frequency dependence
//! rather than about a number.

use super::*;

/// Derived, and exact: a single pole is 3.01 dB down where its reactance
/// equals the resistance it works against. This is the definition of the
/// corner, so a failure here means the response formula is wrong rather
/// than that a transformer is unusual.
#[test]
fn a_single_pole_is_three_decibels_down_at_its_corner() {
    for &hz in &[5.0f32, 12.0, 60.0] {
        let db = Rolloff::one_pole(hz).response_db(hz);
        assert!((db + 3.0103).abs() < 1e-3, "{hz} Hz gave {db:.4} dB");
    }
}

/// Derived. A maximally flat pair is also 3.01 dB down at its corner, which
/// is what makes 0.707 the value a designer reaches for; the two-pole
/// response is a different curve either side of that point, not a different
/// point.
#[test]
fn a_maximally_flat_pair_is_three_decibels_down_at_its_corner() {
    let db = Rolloff::two_pole(12.0, core::f32::consts::FRAC_1_SQRT_2).response_db(12.0);
    assert!((db + 3.0103).abs() < 1e-3, "{db:.4} dB");
}

/// Derived, and the reason [`Rolloff::poles`] exists.
///
/// A caller who ignores the pole count and builds a pair from a
/// single-pole roll-off gets the critically damped pair the `q` guard
/// leaves behind: twice the slope and twice the loss at the corner. The
/// guard stops it resonating; it cannot stop it being the wrong filter, and
/// this test is here to say how wrong.
#[test]
fn a_single_pole_and_a_critically_damped_pair_are_not_the_same_filter() {
    let single = Rolloff::one_pole(12.0);
    let pair = Rolloff::two_pole(12.0, single.q);
    assert!((pair.response_db(12.0) + 6.0206).abs() < 1e-3);
    assert!((single.response_db(12.0) + 3.0103).abs() < 1e-3);
    // An octave below the corner the gap has widened to another 6 dB.
    let gap = single.response_db(6.0) - pair.response_db(6.0);
    assert!(gap > 5.5, "only {gap:.2} dB apart an octave down");
}

/// Derived. Far below the corner a pole gives 6 dB per octave and a pair
/// gives 12, which is what a machine is choosing between when it picks an
/// order.
#[test]
fn the_orders_give_six_and_twelve_decibels_per_octave() {
    let single = Rolloff::one_pole(12.0);
    let pair = Rolloff::two_pole(12.0, 0.6);
    // Two decades below the corner, where the asymptote has arrived. Nearer
    // the corner both are still turning and neither has reached its slope.
    let slope = |r: &Rolloff| r.response_db(0.12) - r.response_db(0.06);
    assert!((slope(&single) - 6.0206).abs() < 0.02, "{}", slope(&single));
    assert!((slope(&pair) - 12.0412).abs() < 0.02, "{}", slope(&pair));
}

/// Derived. Above 0.707 the pair lifts before it falls, which is a
/// transformer with a little weight just above its corner; at or below it
/// the response only ever falls. A machine choosing a Q is choosing between
/// those two behaviours and should know which side of the line it is on.
#[test]
fn a_high_q_lifts_before_it_falls_and_a_low_one_never_does() {
    let peaked = Rolloff::two_pole(12.0, 1.4);
    let damped = Rolloff::two_pole(12.0, 0.6);
    let mut peak = 0.0f32;
    let mut damped_peak = 0.0f32;
    let mut hz = 1.0f32;
    while hz < 2000.0 {
        peak = peak.max(peaked.magnitude(hz));
        damped_peak = damped_peak.max(damped.magnitude(hz));
        hz *= 1.01;
    }
    assert!(peak > 1.05, "a Q of 1.4 should lift; peak was {peak:.4}");
    assert!(
        damped_peak <= 1.0 + 1e-4,
        "a Q of 0.6 lifted to {damped_peak:.4}"
    );
}

/// Derived. A roll-off is a roll-off: two decades above the corner the
/// transformer is out of the way, which is why one of these can sit in the
/// path at all times.
#[test]
fn well_above_the_corner_it_is_transparent() {
    for r in [Rolloff::one_pole(10.0), Rolloff::two_pole(10.0, 0.6)] {
        assert!(r.response_db(1000.0).abs() < 0.02);
    }
    // A corner of zero means no roll-off at all rather than one at DC.
    assert_eq!(Rolloff::one_pole(0.0).magnitude(20.0), 1.0);
}

/// Derived. The corner is `R / (2πL)`, so it belongs to the winding and to
/// what drives it together: more turns or a bigger core lower it, a heavier
/// load raises it. This is why the same transformer measures differently in
/// two machines, and why a corner is not shipped in this crate.
#[test]
fn the_corner_comes_from_the_winding_and_the_load() {
    let r = Rolloff::from_winding(10.0, 600.0);
    assert!((r.hz - 9.5493).abs() < 1e-3, "{} Hz", r.hz);
    assert_eq!(r.poles, Poles::One);
    // Twice the inductance, half the corner. Twice the load, twice the
    // corner.
    assert!((Rolloff::from_winding(20.0, 600.0).hz - r.hz / 2.0).abs() < 1e-4);
    assert!((Rolloff::from_winding(10.0, 1200.0).hz - r.hz * 2.0).abs() < 1e-3);
}

/// Derived, and the arithmetic a response budget lives or dies by: two
/// roll-offs in a chain add their decibels. A machine with a transformer at
/// each end that checks only one of them against a published +0 / −1 dB
/// figure will pass its own test and miss the specification, which is a
/// mistake that has actually been made in a unit this part came from.
#[test]
fn roll_offs_in_a_chain_add_their_decibels() {
    let input = Rolloff::two_pole(7.0, 0.6);
    let output = Rolloff::one_pole(6.0);
    let together = input.magnitude(20.0) * output.magnitude(20.0);
    let sum_db = input.response_db(20.0) + output.response_db(20.0);
    assert!((20.0 * together.log10() - sum_db).abs() < 1e-3);
    // Each alone is comfortably inside a 1 dB budget; the pair of them is
    // most of the way through it.
    assert!(input.response_db(20.0) > -0.6);
    assert!(output.response_db(20.0) > -0.6);
    assert!(sum_db < -0.7, "the pair only cost {sum_db:.2} dB");
}

/// Derived. Inside its limit the core is not doing anything, so a machine
/// can leave it in the signal path at all times and pay nothing for it
/// until the signal is loud and low.
#[test]
fn a_core_inside_its_limit_is_transparent() {
    let core = Core::new(10.0, 0.085);
    for &fraction in &[0.01f32, 0.05, 0.1] {
        let flux = fraction * core.flux_limit;
        let ratio = core.excess(flux).abs() / flux;
        assert!(
            ratio < 1e-3,
            "at {fraction} of the limit the core lost {:.4} %",
            100.0 * ratio
        );
    }
}

/// Derived. [`Core::flux_limit`] has to mean something definite, and what
/// it means is fixed by [`Core::KNEE`]: at exactly the limit the core is
/// 1.5 dB into its knee. A crate that carried a limit without saying where
/// on the curve it sat would be handing every caller a different number.
#[test]
fn the_limit_is_a_decibel_and_a_half_into_the_knee() {
    let core = Core::new(10.0, 0.085);
    let passed = core.flux_limit - core.excess(core.flux_limit);
    let db = 20.0 * (passed / core.flux_limit).log10();
    assert!((db + 1.5052).abs() < 1e-3, "{db:.4} dB into the knee");
}

/// Derived. The law is odd, so a core makes no even harmonics by itself.
/// Any even order a transformer-coupled stage shows comes from an
/// asymmetry elsewhere — a valve's operating point, or a DC offset through
/// the winding — and not from this.
#[test]
fn the_core_saturates_symmetrically() {
    let core = Core::new(10.0, 0.085);
    for &flux in &[0.01f32, 0.085, 0.4, 3.0] {
        assert!((core.excess(flux) + core.excess(-flux)).abs() < 1e-9);
    }
}

/// The published behaviour. Paiva and colleagues measured transformer
/// distortion at low frequencies only; the mechanism is that flux is the
/// integral of the signal, so the amplitude needed to reach the core's knee
/// rises **in proportion to frequency**.
///
/// This drives the core through the flux a machine's leaky integrator would
/// hand it and searches for the amplitude that puts the core a fixed way
/// into its knee. Well above the integrator's corner the ratio is the
/// frequency ratio exactly, which is the published statement; nearer the
/// corner the leak takes a little off it, and both are asserted so that
/// neither is mistaken for the other.
#[test]
fn the_core_saturates_at_low_frequencies_first() {
    let core = Core::new(10.0, 0.085);

    // The flux a one-pole integrator at `integrator_hz` produces for a sine
    // of amplitude `amp` at `hz`.
    let flux_amplitude = |amp: f32, hz: f32| {
        let r = hz / core.integrator_hz;
        amp / (1.0 + r * r).sqrt()
    };
    // The amplitude at which the core loses 5 % of the flux handed to it.
    let knee_amplitude = |hz: f32| {
        let (mut lo, mut hi) = (1e-6f32, 1e6f32);
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            let flux = flux_amplitude(mid, hz);
            if core.excess(flux) / flux < 0.05 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };

    // Well above the integrator's corner, where the flux really is the
    // integral: a fifth of the frequency needs a fifth of the amplitude.
    let ratio = knee_amplitude(1500.0) / knee_amplitude(300.0);
    assert!(
        (ratio - 5.0).abs() < 0.05,
        "five times the frequency needed {ratio:.3} times the amplitude"
    );

    // In the band where a preamp actually runs into this, the leak takes a
    // little off the ratio. The direction is the published one; the
    // magnitude is the model's own and is stated rather than asserted
    // tightly.
    let near = knee_amplitude(150.0) / knee_amplitude(30.0);
    assert!(
        (4.0..5.0).contains(&near),
        "30 Hz against 150 Hz gave {near:.3}"
    );
}

/// The core has to stay finite for anything a machine can hand it,
/// including the first sample after a reset and a signal that has run away.
#[test]
fn the_core_survives_extremes() {
    let core = Core::new(10.0, 0.085);
    for &flux in &[0.0f32, 1e-30, -1e-30, 1e6, -1e6] {
        let y = core.through(1.0, flux);
        assert!(y.is_finite(), "flux {flux} gave {y}");
    }
    assert_eq!(core.excess(0.0), 0.0);
    assert_eq!(core.through(0.5, 0.0), 0.5);
}
