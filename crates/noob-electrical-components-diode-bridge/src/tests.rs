//! Tests for the diode bridge.
//!
//! There is no published measurement of a Neve bridge, and no vendor
//! publishes a harmonic spectrum for one, so nothing here can assert a
//! measured figure. What these do instead is assert the **derived
//! properties of the law** and the one **arithmetic identity that a
//! manufacturer's own drawing corroborates**, and say which is which.
//!
//! The odd symmetry is the important one. It is derived here from the
//! topology, and Pines reaches it independently for a symmetric bridge:
//! "only odd harmonics are present, and the total distortion produced by
//! this model is significantly lower than by the other models for the same
//! α value". Two routes to one answer is the strongest confirmation
//! available.

use super::*;

const K: f32 = THERMAL_SCALE;

/// Discrete Fourier coefficient magnitude of `f` at harmonic `h`, over one
/// period of a sine of amplitude `amp`.
fn harmonic(amp: f32, h: usize) -> f32 {
    const N: usize = 4096;
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for n in 0..N {
        let th = core::f64::consts::TAU * n as f64 / N as f64;
        let y = current(amp * th.sin() as f32, 1.0, K) as f64;
        re += y * (h as f64 * th).cos();
        im += y * (h as f64 * th).sin();
    }
    (2.0 / N as f64 * (re * re + im * im).sqrt()) as f32
}

#[test]
fn the_law_is_odd_so_the_bridge_makes_no_even_harmonics() {
    // Derived: `I·tanh(u/k)` is odd in `u`, so a symmetric drive can
    // produce no even order at all. Corroborated independently by Pines
    // for the symmetric-bridge topology. This is the model's own
    // derivation, not a measurement: no manufacturer publishes a spectrum
    // for these units.
    for u in [0.001, 0.01, 0.05, 0.2, 1.0] {
        let p = current(u, 40e-6, K);
        let n = current(-u, 40e-6, K);
        assert!(
            (p + n).abs() <= p.abs() * 1e-6,
            "not odd at u={u}: f(u)={p}, f(-u)={n}"
        );
    }

    // And in the spectrum: at the drive level the 33609's own annotations
    // imply, even harmonics must sit far below the third.
    let a = 0.34 * K; // tanh argument 0.34, from the -31 dBu bridge level
    let (h2, h3) = (harmonic(a, 2), harmonic(a, 3));
    assert!(
        h3 > 0.0 && h2 <= h3 * 1e-4,
        "even order present: h2={h2}, h3={h3}"
    );
}

#[test]
fn the_third_harmonic_follows_the_tanh_expansion() {
    // Derived, not published. Expanding `tanh(a·sinθ) ≈ a·sinθ − (a·sinθ)³/3`
    // and using `sin³θ = (3sinθ − sin3θ)/4` gives a third harmonic of
    // `a³/12` against a fundamental of `a − a³/4`, so the ratio is
    // **`a²/12`** for small `a`.
    //
    // The dossier's section 4.5 states `a²/24` and therefore puts the
    // third harmonic at 0.48 % for the tanh argument of 0.34 that the
    // 33609's −31 dBu bridge level implies. That is out by a factor of
    // two: the correct figure is about 0.96 %. I checked it three ways,
    // by the algebra above, by this DFT, and by an independent one
    // outside the crate, and all three agree on `a²/12`.
    //
    // It does not weaken the dossier's argument, it sharpens it. Section
    // 4.5 flags that the bridge's own distortion at the annotated drive
    // level already exceeds the 0.075 % the handbook publishes for the
    // bypassed path, and concludes the real bridge must see less signal
    // than the annotation implies. With the correct figure that gap is
    // twice as wide, which is why the bridge drive level is a named
    // calibratable constant in the machine rather than a hard-coded one.
    //
    // This test pins the *shape* of the law. The drive level it gets
    // evaluated at is the machine's business.
    for a in [0.1f32, 0.2, 0.34] {
        let want = a * a / 12.0;
        let got = harmonic(a * K, 3) / harmonic(a * K, 1);
        assert!(
            (got - want).abs() <= want * 0.1,
            "a={a}: third harmonic {got} against the expansion's {want}"
        );
    }
}

#[test]
fn the_current_saturates_at_the_control_current() {
    // Derived: `tanh` is bounded by 1, so the bridge can never pass more
    // signal current than the control current biasing it. That is the
    // difference between a gain element and a multiplier.
    let i = 51.2e-6; // the current 12 dB of reduction needs, from the dossier
    for u in [0.0, 0.05, 0.5, 5.0, 500.0] {
        assert!(
            current(u, i, K).abs() <= i,
            "u={u} passed more than the control current"
        );
    }
    assert!(
        current(500.0, i, K) > i * 0.999,
        "should be hard against the rail"
    );
}

#[test]
fn the_small_signal_resistance_is_k_over_i() {
    // Derived by differentiating the law at the origin. This is the
    // relation the whole gain mechanism rests on: control current sets
    // resistance, resistance sets the machine's divider.
    for i in [1e-6f32, 17.1e-6, 51.2e-6, 154e-6] {
        let r = small_signal_resistance(i, K);
        assert!((r - K / i).abs() <= r * 1e-6, "resistance at {i} A");

        // The slope at the origin is its reciprocal.
        assert!(
            (slope(0.0, i, K) - 1.0 / r).abs() <= 1e-9,
            "slope at the origin disagrees with the resistance at {i} A"
        );

        // And the inverse round-trips.
        assert!(
            (control_for_resistance(r, K) - i).abs() <= i * 1e-6,
            "resistance inverse at {i} A"
        );
    }

    // An unbiased bridge is open.
    assert!(small_signal_resistance(0.0, K).is_infinite());
}

#[test]
fn the_slope_matches_a_numerical_derivative() {
    // Guards the Newton step a caller uses to solve its own node
    // equation: if `slope` and `current` ever disagree, that solve stops
    // converging quadratically and nothing else here would notice.
    let i = 51.2e-6;
    // A wide step on purpose. The currents here are tens of microamps, so
    // in f32 a central difference over a narrow step is mostly
    // cancellation: at h = 1e-5 the two samples differ in their last few
    // bits and the "numeric" derivative is really a measurement of
    // rounding. At 1e-3 the truncation error is still about 1e-4
    // relative, which leaves the tolerance below able to catch any real
    // disagreement between the law and its slope.
    let h = 1e-3;
    for u in [-0.2, -0.05, 0.0, 0.03, 0.09, 0.4] {
        let numeric = (current(u + h, i, K) - current(u - h, i, K)) / (2.0 * h);
        let exact = slope(u, i, K);
        assert!(
            (numeric - exact).abs() <= exact.abs() * 1e-3,
            "u={u}: numeric {numeric} against exact {exact}"
        );
    }
}

#[test]
fn the_antiderivative_differentiates_back_to_the_law() {
    // The reason this lives in the crate rather than in whatever applies
    // the antialiasing: it is a property of the law, and the two have to
    // move together if the law is recalibrated.
    let i = 51.2e-6;
    let h = 1e-4;
    for u in [-0.5, -0.09, -0.01, 0.01, 0.09, 0.5] {
        let numeric =
            (current_antiderivative(u + h, i, K) - current_antiderivative(u - h, i, K)) / (2.0 * h);
        let exact = current(u, i, K);
        assert!(
            (numeric - exact).abs() <= exact.abs() * 1e-3 + 1e-12,
            "u={u}: d/du of the antiderivative gave {numeric}, law says {exact}"
        );
    }
}

#[test]
fn the_antiderivative_stays_finite_where_cosh_overflows() {
    // `cosh` overflows in f32 near an argument of 89, and a bridge driven
    // hard by a caller's `drive` control can reach that. The stable form
    // must not.
    assert!((14.0f32).cosh().is_finite(), "sanity: small argument");
    assert!(
        (200.0f32).cosh().is_infinite(),
        "sanity: cosh really does overflow here"
    );
    for u in [10.0, 100.0, 1000.0, 1e6] {
        let v = current_antiderivative(u, 51.2e-6, K);
        assert!(v.is_finite(), "antiderivative blew up at u={u}: {v}");
        assert!(
            current_antiderivative(-u, 51.2e-6, K).is_finite(),
            "and at -{u}"
        );
    }

    // Against its own asymptote: ln cosh x → |x| − ln 2.
    for x in [20.0f32, 100.0, 5000.0] {
        let want = x - core::f32::consts::LN_2;
        assert!((ln_cosh(x) - want).abs() <= want * 1e-6, "asymptote at {x}");
    }
}

#[test]
fn the_thermal_scale_follows_from_its_own_constants() {
    // It is `2·η·V_T` and nothing else, so recalibrating either constant
    // moves it. The dossier quotes 90.75 mV; the constants give 90.73, and
    // the difference is the dossier's rounding rather than a disagreement.
    assert!(
        (THERMAL_SCALE - 2.0 * IDEALITY * THERMAL_VOLTAGE).abs() < 1e-9,
        "the scale stopped following its constants"
    );
    assert!(
        (THERMAL_SCALE - 0.0907).abs() < 0.0002,
        "expected about 90.7 mV, got {THERMAL_SCALE}"
    );
}

#[test]
fn nothing_produces_a_nan_at_the_extremes() {
    for u in [0.0f32, -0.0, 1e-30, -1e-30, 1e6, -1e6] {
        for i in [0.0f32, CONTROL_FLOOR, 1e-6, 1.0] {
            assert!(current(u, i, K).is_finite(), "current at u={u}, i={i}");
            assert!(slope(u, i, K).is_finite(), "slope at u={u}, i={i}");
            assert!(
                current_antiderivative(u, i, K).is_finite(),
                "antiderivative at u={u}, i={i}"
            );
        }
    }
}
