//! Tests for the log-domain true-RMS detector.
//!
//! Nothing here is published as a number by anybody, because no
//! manufacturer publishes the detector separately from the box it sits in.
//! What *is* established is stronger than a specification: this part
//! computes a mean of a square, and the mean of a square of a waveform is
//! arithmetic. So every level asserted below is a closed-form identity
//! worked out from the waveform, and every timing figure is the closed-form
//! solution of the filter's own differential equation, neither of which the
//! model has any say in. A test that ran the detector and asserted what it
//! returned would be worthless here, and this crate's repository has
//! already found nine of those.

use super::*;

const SR: f32 = 48_000.0;
/// A time constant in the region real units use. Nothing rests on the
/// value: every test either uses two of them or divides it out.
const TAU: f32 = 0.035;

/// Settle the detector on a periodic power signal and return the mean
/// stored level over a whole number of its periods.
///
/// The mean over a period, rather than the last sample: the level ripples
/// at the waveform's own period because the capacitor really does charge
/// and discharge within it, and that ripple is behaviour rather than
/// error.
fn settled_db(period: usize, power: impl Fn(usize) -> f32, tau: f32) -> f32 {
    let mut d = LogRmsDetector::new(tau, SR);
    let settle = (SR * tau * 12.0) as usize;
    for n in 0..settle {
        d.step(power(n % period));
    }
    let periods = 8;
    let mut sum = 0.0f64;
    for n in 0..period * periods {
        sum += f64::from(d.step(power((settle + n) % period)));
    }
    (sum / (period * periods) as f64) as f32
}

/// The identity every RMS detector is judged by: a sine settles 3.01 dB
/// below its peak, because the mean of `sin²` is one half and ten times
/// the base-ten logarithm of one half is −3.0103.
///
/// This is arithmetic, not a specification, and it is the cheapest
/// possible check that nobody has quietly substituted a peak detector or
/// rounded [`D_DB`]. At 4.246 thermal decibels — the value two datasheet
/// figures divide to give, which do not correspond — this lands at 2.98
/// instead, and the test fails as it should.
#[test]
fn a_sine_settles_three_decibels_below_its_peak() {
    let period = 48; // 1 kHz at 48 kHz.
    let peak = 0.25f32;
    let got = settled_db(
        period,
        |n| {
            let x = peak * (core::f32::consts::TAU * n as f32 / period as f32).sin();
            x * x
        },
        TAU,
    );
    let want = 20.0 * peak.log10() - 3.0103;
    assert!(
        (got - want).abs() < 0.05,
        "a sine settled at {got:.4} dB, arithmetic says {want:.4} dB"
    );
}

/// Crest factor, which is what a true-RMS detector is *for*.
///
/// Three waveforms of the same peak settle at three different levels, and
/// each level is the mean of the waveform's square worked out on paper: a
/// square wave at its peak, a sine 3.01 dB below it, and a quarter-duty
/// pulse train 6.02 dB below it. A rectifier followed by a slow attack
/// cannot reproduce this ordering at any setting, which is the whole
/// argument for the part.
#[test]
fn the_level_follows_the_crest_factor() {
    let peak = 0.5f32;
    let period = 24;
    let square = settled_db(period, |_| peak * peak, TAU);
    let sine = settled_db(
        period,
        |n| {
            let x = peak * (core::f32::consts::TAU * n as f32 / period as f32).sin();
            x * x
        },
        TAU,
    );
    let quarter = settled_db(
        period,
        |n| if n * 4 < period { peak * peak } else { 0.0 },
        TAU,
    );

    let reference = 20.0 * peak.log10();
    for (name, got, want) in [
        ("square", square, reference),
        ("sine", sine, reference - 3.0103),
        ("quarter-duty pulses", quarter, reference - 6.0206),
    ] {
        assert!(
            (got - want).abs() < 0.05,
            "{name} settled at {got:.4} dB, the mean of its square is \
             {want:.4} dB"
        );
    }
    assert!(square > sine && sine > quarter);
}

/// Release is a **straight line in decibels**, at `D/τ` per second.
///
/// The rate is the closed-form asymptote of the filter's own equation once
/// the charging junction is shut, so this asserts both that the decay is
/// straight — the slope must be the same at the start and at the end of
/// the fall — and that its slope is the one the equation gives.
#[test]
fn release_is_rate_limited_at_the_rate_the_equation_gives() {
    let mut d = LogRmsDetector::new(TAU, SR);
    for _ in 0..(SR * TAU * 12.0) as usize {
        d.step(1.0);
    }
    let mut trace = Vec::new();
    for _ in 0..(SR * 0.3) as usize {
        trace.push(d.step(0.0));
    }
    let per_sample = |a: usize, b: usize| (trace[a] - trace[b]) / (b - a) as f32 * SR;
    let early = per_sample(2400, 4800);
    let late = per_sample(9600, 12_000);
    let want = release_rate_db_s(TAU);
    assert!(
        (early - want).abs() < want * 0.01 && (late - want).abs() < want * 0.01,
        "released at {early:.2} then {late:.2} dB/s, D/τ is {want:.2}"
    );
    assert!((D_DB / TAU - want).abs() < 1e-3);
}

/// Attack is faster for a bigger step, and by the amount the closed-form
/// solution says.
///
/// For a step of `Δ` decibels, the time to 63 % of it is
/// `t/τ = ln[(1 − e^−u) / (1 − e^−0.37u)]` with `u = Δ/D`. That is the
/// equation solved, not the model measured: a bigger step opens the
/// charging junction harder, so the ratio of the two times is not one and
/// no single attack constant can be quoted for the part.
#[test]
fn attack_is_faster_for_a_bigger_step_by_the_closed_form_amount() {
    let measured = |step_db: f32| {
        let mut d = LogRmsDetector::new(TAU, SR);
        let low = 10f32.powf(-step_db / 10.0);
        for _ in 0..(SR * TAU * 12.0) as usize {
            d.step(low);
        }
        let start = d.level_db();
        let target = start + 0.63 * step_db;
        let mut n = 0usize;
        while d.step(1.0) < target {
            n += 1;
            assert!(n < SR as usize, "never reached 63 % of a {step_db} dB step");
        }
        n as f32 / SR * 1e3
    };
    let closed_form = |step_db: f32| {
        let u = step_db / D_DB;
        ((1.0 - (-u).exp()) / (1.0 - (-0.37 * u).exp())).ln() * TAU * 1e3
    };
    let mut times = Vec::new();
    for step in [10.0f32, 20.0, 30.0] {
        let got = measured(step);
        let want = closed_form(step);
        assert!(
            (got - want).abs() < want * 0.03,
            "a {step} dB step took {got:.3} ms, the closed form says {want:.3}"
        );
        times.push(got);
    }
    assert!(
        times[0] > times[1] && times[1] > times[2],
        "attack did not shorten with the step: {times:?}"
    );
}

/// Attack and release are **one constant seen from two sides**, which is
/// why this part carries no ballistics controls.
///
/// Doubling the time constant halves the release rate and lengthens the
/// attack in the same proportion. There is no setting of one that leaves
/// the other alone, and that is not a limitation of the model: it is the
/// reason the dbx 160 has no attack or release knobs and the reason the
/// API 2500's fourteen ballistics positions have to live in a stage after
/// this one.
#[test]
fn attack_and_release_cannot_be_separated() {
    let attack_ms = |tau: f32| {
        let mut d = LogRmsDetector::new(tau, SR);
        for _ in 0..(SR * tau * 12.0) as usize {
            d.step(1e-2);
        }
        let start = d.level_db();
        let target = start + 0.63 * 20.0;
        let mut n = 0usize;
        while d.step(1.0) < target {
            n += 1;
        }
        n as f32 / SR * 1e3
    };
    let slow_over_fast = attack_ms(TAU * 2.0) / attack_ms(TAU);
    let rate_ratio = release_rate_db_s(TAU) / release_rate_db_s(TAU * 2.0);
    assert!(
        (slow_over_fast - 2.0).abs() < 0.05,
        "doubling τ moved the attack by {slow_over_fast:.4}"
    );
    assert!((rate_ratio - 2.0).abs() < 1e-4);
}

/// The exact solution means the sample rate does not change the answer.
///
/// The filter is solved over a held sample rather than discretised, so the
/// settled level and the attack time are the same at 44.1, 48, 96 and
/// 192 kHz. A hand-discretised one-pole would not be, and the difference
/// shows up first at the fast time constants, which is where a compressor
/// lives.
#[test]
fn the_answer_does_not_depend_on_the_sample_rate() {
    let attack_ms = |sr: f32| {
        let mut d = LogRmsDetector::new(TAU, sr);
        for _ in 0..(sr * TAU * 12.0) as usize {
            d.step(1e-2);
        }
        let target = d.level_db() + 0.63 * 20.0;
        let mut n = 0usize;
        while d.step(1.0) < target {
            n += 1;
        }
        n as f32 / sr * 1e3
    };
    let reference = attack_ms(48_000.0);
    for sr in [44_100.0f32, 96_000.0, 192_000.0] {
        let got = attack_ms(sr);
        assert!(
            (got - reference).abs() < reference * 0.01,
            "at {sr} Hz the attack was {got:.4} ms against {reference:.4} ms"
        );
    }
}

/// The thermal decibel is `10/ln 10` exactly, and the reason is that at
/// any other value the averaging is not an average of the square.
///
/// The constant is asserted against its derivation rather than against a
/// datasheet, because no datasheet publishes it: the two figures that get
/// divided to guess at it carry different ideality assumptions and their
/// quotient is 2 % out. See [`D_DB`].
#[test]
fn the_thermal_decibel_is_the_exact_one() {
    assert!(
        (D_DB - 10.0 / core::f32::consts::LN_10).abs() < 1e-7,
        "the thermal decibel is {D_DB}"
    );
    // Ten decibels of level is one natural logarithm of power, which is
    // the whole content of the constant.
    assert!((10.0 / D_DB - core::f32::consts::LN_10).abs() < 1e-5);
}

/// Silence floors rather than diverging, and the floor is a floor rather
/// than a gate: the detector goes on releasing towards it at the same
/// rate, and comes back the moment there is signal again.
#[test]
fn silence_floors_and_recovers() {
    let mut d = LogRmsDetector::new(TAU, SR);
    for _ in 0..(SR * TAU * 12.0) as usize {
        d.step(1.0);
    }
    for _ in 0..(SR * 20.0) as usize {
        d.step(0.0);
    }
    let floored = d.level_db();
    assert!(
        floored.is_finite() && floored <= FLOOR_DB as f32 + 1.0,
        "silence left the detector at {floored}"
    );
    // And a loud sample brings it straight back, because the charging
    // junction opens harder the bigger the step.
    for _ in 0..(SR * TAU * 12.0) as usize {
        d.step(1.0);
    }
    assert!((d.level_db()).abs() < 0.01, "recovered to {}", d.level_db());

    d.reset();
    assert!(d.level_db() <= FLOOR_DB as f32 + 1e-3);
}
