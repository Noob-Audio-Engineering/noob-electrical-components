//! Tests for the FET used as a variable resistor.
//!
//! One published figure exists for this part and it is a limit rather than
//! a coefficient: distortion under 3 % within ±250 mV of drain-source
//! swing, from EDN's guide to using FETs for voltage controlled circuits.
//! It is asserted at [`the_reference_swing_is_the_published_figure`]. No
//! manufacturer publishes a second- or third-order coefficient for a JFET
//! in this service, and no measurement of an 1176's gain FET exists in
//! public, so nothing else here can assert a magnitude.
//!
//! What the rest assert instead are the **shapes and orderings the sources
//! do establish** — that the even term is the dominant one, that halving
//! the swing costs the two orders different amounts, that the control law
//! saturates — and the **arithmetic identities of the laws themselves**,
//! which are exact and would catch an algebra error. Each test says which
//! of those it is.

use super::*;

/// Magnitude of harmonic `h` of a signal driven through a divider this
/// channel shunts, over one period of a sine of amplitude `amp`.
///
/// The divider is the caller's, not the part's, but a harmonic only exists
/// once the modulated conductance has been turned into a gain, so the
/// simplest one a machine could build stands in here: a fixed normalised
/// conductance `w`, modulated by the swing, closed as `1 / (1 + w·m)`.
fn harmonic(amp: f32, scale: f32, shape: Nonlinearity, w: f32, h: usize) -> f32 {
    const N: usize = 4096;
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for n in 0..N {
        let th = core::f64::consts::TAU * n as f64 / N as f64;
        let x = amp * th.sin() as f32;
        let y = (x / (1.0 + w * conductance_modulation(x * scale, shape))) as f64;
        re += y * (h as f64 * th).cos();
        im += y * (h as f64 * th).sin();
    }
    (2.0 / N as f64 * (re * re + im * im).sqrt()) as f32
}

#[test]
fn the_reference_swing_is_the_published_figure() {
    // Published. EDN, "A guide to using FETs for voltage controlled
    // circuits": distortion below 3 % within plus or minus 250 mV, and
    // "reasonably" low below about 500 mV peak to peak. This is the only
    // number in the crate that anybody published, and it is a threshold
    // rather than a coefficient, which is why the coefficients are the
    // caller's.
    assert_eq!(REFERENCE_SWING_VOLTS, 0.25);
    assert_eq!(2.0 * REFERENCE_SWING_VOLTS, 0.5, "500 mV peak to peak");
}

#[test]
fn the_control_law_starts_at_nothing_and_approaches_its_plateau_without_reaching_it() {
    // Derived from the law, not measured. The shape is what the 1176
    // research asks for: a near-constant number of decibels per volt while
    // the channel is near pinch-off, flattening where the on-resistance
    // stops falling against the series resistance.
    let (s, max) = (40.0f32, 40.0f32);
    assert_eq!(
        attenuation_db(0.0, s, max),
        0.0,
        "no control, no gain change"
    );

    let mut previous = 0.0f32;
    for step in 1..=200 {
        let v = step as f32 * 0.05;
        let db = attenuation_db(v, s, max);
        assert!(db < previous, "not monotone at {v} V: {db} then {previous}");
        assert!(db > -max, "law reached its plateau at {v} V: {db} dB");
        previous = db;
    }
    // 10 V is 10 times the volt at which the law is `1 − 1/e` of the way
    // down, so it must be within a hair of the plateau.
    assert!(
        attenuation_db(10.0, s, max) < -max + 0.01,
        "plateau not approached: {} dB",
        attenuation_db(10.0, s, max)
    );
}

#[test]
fn the_initial_slope_is_the_slope_it_was_given_whatever_the_plateau() {
    // Arithmetic identity of the law: `G'(0) = −slope` for every plateau,
    // because the plateau divides out of the derivative at the origin. It
    // matters because a machine that changes its plateau — the 1176's
    // all-buttons mode drops it — must not find its threshold moving as a
    // side effect.
    for max in [16.0f32, 32.0, 40.0, 48.0] {
        let s = 40.0f32;
        let h = 1e-4;
        let slope = -(attenuation_db(h, s, max) - attenuation_db(0.0, s, max)) / h;
        assert!(
            (slope - s).abs() < 0.05,
            "initial slope {slope} dB/V against {s} at a plateau of {max} dB"
        );
    }
}

#[test]
fn the_law_bends_away_from_that_slope_which_is_what_makes_a_ratio_stop_climbing() {
    // Derived. A limiter whose control law were a straight line would keep
    // increasing its ratio as it went deeper; the flattening is what stops
    // it. So the decibels bought by the next volt must fall with every
    // volt already spent.
    let (s, max) = (40.0f32, 40.0f32);
    let per_volt = |v: f32| attenuation_db(v, s, max) - attenuation_db(v + 0.1, s, max);
    let mut previous = f32::INFINITY;
    for step in 0..40 {
        let gained = per_volt(step as f32 * 0.1);
        assert!(gained < previous, "not saturating at step {step}");
        previous = gained;
    }
}

#[test]
fn an_attenuation_of_six_decibels_is_a_channel_matching_its_series_resistor() {
    // Arithmetic identity of a divider, and the check that
    // `conductance_ratio` is the right way up. Half the voltage means the
    // shunt equals the series element, so the normalised conductance is 1.
    let half = -20.0 * 2f32.log10();
    assert!(
        (conductance_ratio(half) - 1.0).abs() < 1e-3,
        "{} at {half} dB",
        conductance_ratio(half)
    );
    assert_eq!(
        conductance_ratio(0.0),
        0.0,
        "no attenuation is an open channel"
    );

    // And it rises without bound as the attenuation deepens, monotonically.
    let mut previous = 0.0f32;
    for db in [-1.0f32, -3.0, -6.0, -12.0, -24.0, -40.0] {
        let w = conductance_ratio(db);
        assert!(w > previous, "not monotone at {db} dB");
        previous = w;
    }
}

#[test]
fn conductance_composes_the_two_laws_in_the_order_a_divider_needs() {
    // Arithmetic identity: the convenience form must be the two steps.
    for v in [0.0f32, 0.1, 0.5, 1.0, 3.0] {
        let (s, max) = (48.0f32, 48.0f32);
        assert_eq!(
            conductance(v, s, max),
            conductance_ratio(attenuation_db(v, s, max))
        );
    }
}

#[test]
fn a_linear_channel_is_a_plain_divider_at_any_drive() {
    // Arithmetic identity, and the reference the coloured channels are
    // heard against.
    for u in [-4.0f32, -1.0, 0.0, 0.3, 1.0, 9.0] {
        assert_eq!(conductance_modulation(u, Nonlinearity::LINEAR), 1.0);
    }
}

#[test]
fn the_even_term_makes_a_second_harmonic_and_the_odd_term_a_third() {
    // Derived from the law. The modulation multiplies a signal that is
    // itself the drive, so each term appears one order higher than it
    // looks: the even coefficient is first order in the drive and lands on
    // the second harmonic, the odd one is second order and lands on the
    // third. An implementation that had them the other way round would
    // pass a total-distortion test and fail this.
    let (scale, w, amp) = (1.0f32, 0.5f32, 0.3f32);

    let even = Nonlinearity::new(0.15, 0.0);
    let (h2, h3) = (
        harmonic(amp, scale, even, w, 2),
        harmonic(amp, scale, even, w, 3),
    );
    assert!(h2 > 1e-4, "even term made no second harmonic: {h2}");
    assert!(
        h3 < h2 * 0.05,
        "even term made a third harmonic: {h3} of {h2}"
    );

    let odd = Nonlinearity::new(0.0, 0.15);
    let (h2, h3) = (
        harmonic(amp, scale, odd, w, 2),
        harmonic(amp, scale, odd, w, 3),
    );
    assert!(h3 > 1e-5, "odd term made no third harmonic: {h3}");
    assert!(
        h2 < h3 * 0.05,
        "odd term made a second harmonic: {h2} of {h3}"
    );
}

#[test]
fn the_second_harmonic_dominates_which_is_the_ordering_the_sources_publish() {
    // Published as an ordering, not a magnitude. EDN states that a JFET's
    // ohmic-region distortion is predominantly second harmonic, and a
    // GroupDIY analysis of the 1178 describes the same character. A fitted
    // pair whose even term is the larger must therefore come out with the
    // second harmonic on top; if it did not, the sign or the order of the
    // polynomial would be wrong.
    let shape = Nonlinearity::new(0.15, 0.05);
    let (h2, h3) = (
        harmonic(0.3, 1.0, shape, 0.5, 2),
        harmonic(0.3, 1.0, shape, 0.5, 3),
    );
    assert!(
        h2 > h3,
        "second harmonic {h2} did not dominate the third {h3}"
    );
}

#[test]
fn halving_the_swing_costs_the_second_order_half_and_the_third_order_three_quarters() {
    // Derived, and the whole arithmetic point of a reduced-drive circuit
    // such as the 1176's low-noise one. The even product is first order in
    // the drive and the odd second, so halving the drive divides them by
    // two and by four. The tolerance is loose because the divider mixes
    // the orders slightly; the ratios themselves are exact in the law.
    let shape = Nonlinearity::new(0.15, 0.05);
    let (full, half) = (swing_scale(1.0, false), swing_scale(1.0, true));
    assert_eq!(half, full * HALF_SWING);

    let (amp, w) = (0.3f32, 0.5f32);
    let second = harmonic(amp, full, shape, w, 2) / harmonic(amp, half, shape, w, 2);
    let third = harmonic(amp, full, shape, w, 3) / harmonic(amp, half, shape, w, 3);
    assert!(
        (second - 2.0).abs() < 0.1,
        "second harmonic fell by {second}, not 2"
    );
    assert!(
        (third - 4.0).abs() < 0.4,
        "third harmonic fell by {third}, not 4"
    );
}

#[test]
fn a_machine_in_its_own_units_gets_the_same_drive_as_one_in_volts() {
    // Arithmetic identity, and the reason `swing_scale` takes a reference
    // rather than reading `REFERENCE_SWING_VOLTS`: a machine whose full
    // scale is not one volt says where the reference lands for it, and the
    // drive that reaches the law is the same number either way.
    let volts = swing_scale(REFERENCE_SWING_VOLTS, false);
    let quarter_of_full_scale = swing_scale(1.0, false);
    assert_eq!(0.25 * volts, 1.0 * quarter_of_full_scale);
}

#[test]
fn the_modulation_stays_within_its_bounds_however_hard_it_is_driven() {
    // Not a physical claim, and the crate says so: the polynomial is a
    // local fit and a large enough drive would send it negative, inverting
    // whatever divider a caller built on it. The bound is a guard rail
    // standing in for the channel leaving its ohmic region, and this
    // asserts it holds at drives no machine should ever reach.
    let shape = Nonlinearity::new(1.0, 0.5);
    for u in [-1e6f32, -100.0, -3.0, 0.0, 3.0, 100.0, 1e6] {
        let m = conductance_modulation(u, shape);
        assert!(
            (MODULATION_FLOOR..=MODULATION_CEILING).contains(&m),
            "modulation {m} out of bounds at a drive of {u}"
        );
    }
    // And a divider closed on the bounded modulation can never invert or
    // blow up, which is the property the bound exists to buy.
    for u in [-1e6f32, 0.0, 1e6] {
        let g = 1.0 / (1.0 + 20.0 * conductance_modulation(u, shape));
        assert!(g > 0.0 && g < 1.0, "divider gain {g} at a drive of {u}");
    }
}
