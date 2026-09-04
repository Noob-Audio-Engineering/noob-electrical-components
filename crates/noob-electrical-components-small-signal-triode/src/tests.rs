//! Tests for the small-signal triode stage.
//!
//! No independent measurement of a 610's own transfer curve exists in
//! public, and no manufacturer publishes a harmonic spectrum for one, so
//! nothing here can assert a measured figure and nothing here pretends to.
//! What these assert instead are the **derived properties of the law** and
//! the **character the sources agree on** for a single-ended triode gain
//! stage, and each test says which of the two it is.
//!
//! The character comes from Blencowe, chapter 1: the grid curves of a
//! triode are unequally spaced, so a symmetric grid swing gives an
//! asymmetric plate swing and a "decaying series of all harmonics,
//! dominated by the second"; and "in most triodes the harmonic distortion
//! is directly proportional to signal level".
//!
//! One test here exists for a different reason from all the others.
//! [`the_bias_never_changes_the_gain`] is the boundary against
//! `noob-electrical-components-remote-cutoff-triode`: it measures the one
//! property that makes these two valves different components rather than
//! one component with two parameter sets.

use super::*;

/// A knee in the middle of the range a preamp stage uses.
const KNEE: f32 = 2.5;
/// A bias in the middle of the range a preamp stage uses.
const BIAS: f32 = 0.12;

/// Magnitude of harmonic `h` of the stage's output for a sine of amplitude
/// `amp`, by discrete Fourier sum over one period.
fn harmonic(amp: f32, bias: f32, knee: f32, h: usize) -> f32 {
    const N: usize = 4096;
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for n in 0..N {
        let th = core::f64::consts::TAU * n as f64 / N as f64;
        let y = transfer(amp * th.sin() as f32, bias, knee) as f64;
        re += y * (h as f64 * th).cos();
        im += y * (h as f64 * th).sin();
    }
    (2.0 / N as f64 * (re * re + im * im).sqrt()) as f32
}

/// Incremental gain at the operating point, measured off the curve rather
/// than read from [`transfer_slope`], which is one there by construction
/// and so could not fail.
///
/// Measured as the fundamental of a small sine rather than by a difference
/// quotient. Deep in the bend the law is a difference of two nearly equal
/// values of `S`, and a quotient of that in single precision loses most of
/// its digits to the cancellation; a Fourier sum over a whole period
/// averages the same rounding away and measures the shape rather than the
/// arithmetic.
fn measured_gain(bias: f32, knee: f32) -> f32 {
    const AMP: f32 = 0.01;
    harmonic(AMP, bias, knee, 1) / AMP
}

/// Derived. The operating-point offset is subtracted out, so a stage at
/// rest passes nothing, and the normalisation puts the small-signal gain at
/// exactly one. Both matter to a machine: the first means no DC has to be
/// blocked after the stage, the second means the stage's bias can be voiced
/// without the gain structure moving underneath it.
#[test]
fn it_rests_at_zero_with_unity_gain() {
    for &bias in &[0.0f32, 0.08, 0.12, 0.2, 0.5] {
        for &knee in &[2.0f32, 2.5, 3.5, 4.0] {
            assert!(
                transfer(0.0, bias, knee).abs() < 1e-6,
                "bias {bias}, knee {knee}: not zero at rest"
            );
            let g = measured_gain(bias, knee);
            assert!(
                (g - 1.0).abs() < 1e-2,
                "bias {bias}, knee {knee}: small-signal gain {g}"
            );
        }
    }
}

/// **The boundary against the remote-cutoff triode**, and the reason these
/// are two components rather than one.
///
/// A remote-cutoff valve is a gain element: its grid is wound with a
/// varying pitch so that its transconductance falls away in a long shallow
/// tail, and a variable-mu compressor gets the whole of its gain reduction
/// by moving that grid bias. This valve has no such tail. Its bias is a
/// voicing control, not a gain control, and the whole usable bias range —
/// including everything a sagging supply can walk it to — moves the gain by
/// essentially nothing.
///
/// Derived, and it is a structural property rather than a tolerance: the
/// law divides by the slope at the bias point precisely so that the bias
/// cannot reach the gain. The measurement is taken off the curve so that
/// the assertion is about the shape rather than about the arithmetic.
#[test]
fn the_bias_never_changes_the_gain() {
    for &knee in &[2.0f32, 2.5, 3.5, 4.0] {
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        // From no bias at all to ten times what a supply sag walks a 610's
        // stage to, which is about 0.36 at worst.
        for i in 0..=40 {
            let bias = i as f32 * 0.1;
            let g = measured_gain(bias, knee);
            lo = lo.min(g);
            hi = hi.max(g);
        }
        let spread_db = 20.0 * (hi / lo).log10();
        assert!(
            spread_db < 0.01,
            "knee {knee}: the whole bias range moved the gain by {spread_db:.4} dB; \
             this law has no bias-controlled gain and a remote-cutoff valve is all \
             bias-controlled gain, which is why neither crate can serve for the other"
        );
    }
}

/// Derived. The stage is a bend, not a clip: it stays single-valued and
/// increasing everywhere, so no drive can fold it back on itself.
#[test]
fn it_is_monotonic_and_bounded() {
    let mut last = f32::NEG_INFINITY;
    let mut v = -12.0f32;
    while v <= 12.0 {
        let y = transfer(v, BIAS, KNEE);
        assert!(y.is_finite(), "{v} gave {y}");
        assert!(y > last, "not monotonic at {v}");
        last = y;
        v += 0.01;
    }
    // `S` is bounded by ±1, so the stage is bounded by ±(1 + S(b)) / S'(b).
    let ceiling = (1.0 + s_curve(BIAS, KNEE)) / s_slope(BIAS, KNEE);
    assert!(transfer(1e6, BIAS, KNEE) <= ceiling * 1.000_01);
}

/// Derived. With no bias the law is odd, so a symmetric drive can produce
/// no even order at all. This is the negative half of the next test: the
/// second harmonic of a real stage comes from where it sits on the curve,
/// not from the curve.
#[test]
fn an_unbiased_stage_makes_no_even_harmonics() {
    let h2 = harmonic(0.6, 0.0, KNEE, 2);
    let h1 = harmonic(0.6, 0.0, KNEE, 1);
    assert!(
        h2 / h1 < 1e-4,
        "an odd law made a second harmonic of {:.2} %",
        100.0 * h2 / h1
    );
}

/// The published character. Blencowe describes a triode gain stage's
/// distortion as a decaying series dominated by the second harmonic, which
/// is a shape rather than a magnitude; no measurement of one of these
/// stages is published, so a shape is what this asserts.
#[test]
fn a_biased_stage_is_dominated_by_its_second_harmonic() {
    for &amp in &[0.2f32, 0.4, 0.7] {
        let h1 = harmonic(amp, BIAS, KNEE, 1);
        let h2 = harmonic(amp, BIAS, KNEE, 2);
        let h3 = harmonic(amp, BIAS, KNEE, 3);
        let h4 = harmonic(amp, BIAS, KNEE, 4);
        assert!(h2 > h3, "amp {amp}: h2 {h2:.5} did not exceed h3 {h3:.5}");
        assert!(
            h3 > h4,
            "amp {amp}: the series did not decay past the third"
        );
        assert!(
            h2 / h1 > 1e-3,
            "amp {amp}: a single-ended stage with no audible second harmonic"
        );
    }
}

/// The published character again: "in most triodes the harmonic distortion
/// is directly proportional to signal level" (Blencowe, chapter 1). For a
/// curve whose leading error term is quadratic that is exactly what the law
/// gives — the second harmonic grows with the square of the drive, so the
/// *proportion* of it grows with the drive — and this asserts the
/// proportionality rather than any particular distortion figure.
///
/// It is a small-signal statement and the test stays inside where it holds.
/// By a fifth of the saturation scale the higher orders have arrived and
/// doubling the level multiplies the second harmonic by rather more than
/// two, which is the stage overloading rather than the source being wrong.
#[test]
fn distortion_grows_in_proportion_to_level() {
    let ratio = |amp: f32| harmonic(amp, BIAS, KNEE, 2) / harmonic(amp, BIAS, KNEE, 1);
    let mut previous = ratio(0.025);
    for &amp in &[0.05f32, 0.1] {
        let next = ratio(amp);
        assert!(
            (next / previous - 2.0).abs() < 0.08,
            "doubling the level to {amp} multiplied the second harmonic ratio by {:.3}",
            next / previous
        );
        previous = next;
    }
}

/// Derived. The knee exponent decides how abruptly the stage closes onto
/// its asymptote and nothing else: every exponent saturates at the same
/// place. This is the ordering an output stage with less headroom than the
/// stage before it is voiced by.
#[test]
fn the_knee_exponent_orders_how_abruptly_it_closes() {
    let mut previous = 0.0f32;
    for &knee in &[2.0f32, 2.5, 3.5, 4.0] {
        let y = s_curve(1.5, knee);
        assert!(
            y > previous,
            "knee {knee} was no closer to the asymptote than the softer one"
        );
        previous = y;
        assert!(y < 1.0, "knee {knee} passed the asymptote");
    }
    // Every exponent has the same asymptote; only the approach differs.
    for &knee in &[2.0f32, 2.5, 3.5, 4.0] {
        assert!((s_curve(1e6, knee) - 1.0).abs() < 1e-3);
    }
}

/// Derived consistency: [`s_slope`] must be the derivative of [`s_curve`],
/// and [`transfer_slope`] the derivative of [`transfer`]. A machine
/// integrating the law for anti-aliasing uses all four, and a typo in any
/// one of them would be a silent mis-shaping rather than a failure.
#[test]
fn the_slopes_are_the_derivatives_of_the_curves() {
    let h = 1e-3f32;
    for &knee in &[2.0f32, 2.5, 3.5, 4.0] {
        for &v in &[-2.0f32, -0.5, 0.0, 0.3, 1.0, 3.0] {
            let numeric = (s_curve(v + h, knee) - s_curve(v - h, knee)) / (2.0 * h);
            let analytic = s_slope(v, knee);
            assert!(
                (numeric - analytic).abs() < 5e-3,
                "S' at {v}, knee {knee}: {analytic} against {numeric}"
            );
            let numeric = (transfer(v + h, BIAS, knee) - transfer(v - h, BIAS, knee)) / (2.0 * h);
            let analytic = transfer_slope(v, BIAS, knee);
            assert!(
                (numeric - analytic).abs() < 5e-3,
                "T' at {v}, knee {knee}: {analytic} against {numeric}"
            );
        }
    }
}

/// The struct is only the two numbers, and has to agree with the free
/// functions it stands for.
#[test]
fn the_stage_struct_is_the_law_with_its_numbers_attached() {
    let t = Triode::new(BIAS, KNEE);
    for &v in &[-3.0f32, -0.4, 0.0, 0.4, 3.0] {
        assert_eq!(t.shape(v), transfer(v, BIAS, KNEE));
        assert_eq!(t.slope(v), transfer_slope(v, BIAS, KNEE));
        assert_eq!(t.shape_at(v, 0.3), transfer(v, 0.3, KNEE));
    }
}

/// Nothing a machine can hand it produces a NaN or an infinity: a stage
/// that folded here would take the whole audio path with it.
#[test]
fn it_survives_extremes() {
    for &v in &[0.0f32, 1e-30, -1e-30, 1e6, -1e6, f32::MAX, f32::MIN] {
        for &bias in &[0.0f32, 0.12, 4.0] {
            let y = transfer(v, bias, KNEE);
            assert!(y.is_finite(), "v {v}, bias {bias} gave {y}");
        }
    }
}
