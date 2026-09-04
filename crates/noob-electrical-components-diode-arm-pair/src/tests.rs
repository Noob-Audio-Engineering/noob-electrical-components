//! Tests for the diode arm pair.
//!
//! **Nothing about this part is published.** No factory handbook, no
//! specification and no measurement of the module it comes from has ever
//! been released, the diode in it has no reachable datasheet, and the only
//! primary evidence is one photographed drawing. So no test here asserts a
//! measured figure, and none pretends to: what they assert instead are the
//! **derived properties of the law** and the **arithmetic identities** that
//! follow from the topology, and each says which.
//!
//! The identity in the first test is the load-bearing one. It states the
//! exact relationship between this part and the diode bridge next door —
//! that the general law contains the bridge's special case, and that the
//! bridge's constant is therefore wrong here by a factor of two — and it
//! states it as arithmetic rather than as prose, so that anyone tempted to
//! merge the two crates has to break a test to do it.

use super::*;

/// A bias current in the middle of the working range, in amps.
const I: f32 = 40e-6;

/// The diode bridge's thermal scale `2·η·V_T`, written out rather than
/// imported.
///
/// This crate does not depend on that one and must not: the two parts are
/// different circuits and a dependency would assert a kinship that does
/// not exist. What the two genuinely share is the published ideality and
/// thermal voltage, cited from the same paper by both, so writing the
/// bridge's scale here from this crate's own constants is the honest form
/// of the comparison. It is about 90.7 mV, and note that it is **twice**
/// [`JUNCTION_SCALE`] and **equal to** a two-junction arm's `v_n`, which
/// is the coincidence the crate documentation warns about.
const BRIDGE_SCALE: f32 = 2.0 * JUNCTION_SCALE;

/// Discrete Fourier coefficient magnitude of a sampled period at harmonic
/// `h`.
fn harmonic(x: &[f64], h: usize) -> f64 {
    let n = x.len();
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for (k, y) in x.iter().enumerate() {
        let th = core::f64::consts::TAU * k as f64 / n as f64;
        re += y * (h as f64 * th).cos();
        im += y * (h as f64 * th).sin();
    }
    2.0 / n as f64 * (re * re + im * im).sqrt()
}

/// One period of the signal current for a sinusoidal differential voltage
/// of peak `u_pk`, on a forward pair where the inverse is closed form.
///
/// The inverse is written independently as `I·tanh(u / 2·V_n)` rather than
/// obtained from the crate, which is the point: the first test establishes
/// that this is exactly the inverse of [`DiodeArmPair::voltage`], and
/// every harmonic figure below then rests on a closed form somebody can
/// check by hand.
fn forward_current_period(e: &DiodeArmPair, u_pk: f32, n: usize) -> Vec<f64> {
    assert_eq!(e.r_b, 0.0, "the closed-form inverse needs no bulk term");
    (0..n)
        .map(|k| {
            let th = core::f64::consts::TAU * k as f64 / n as f64;
            let u = u_pk * th.sin() as f32;
            (I * (u / (2.0 * e.v_n)).tanh()) as f64
        })
        .collect()
}

#[test]
fn the_law_contains_the_diode_bridges_tanh_and_only_as_a_special_case() {
    // **Identity, derived.** With one junction per arm and no bulk term,
    // (G1) is `u = 2·η·V_T·artanh(i/I)`, whose inverse is exactly the
    // diode bridge's `i = I·tanh(u / 2ηV_T)`. That is the whole of the
    // overlap between the two parts, and it holds at a corner this part is
    // never in: as drawn it has two junctions per arm and a bulk term.
    let ring = DiodeArmPair::ring();
    let mut worst = 0.0f32;
    for decade in 0..4 {
        let i_bias = 1e-6 * 10f32.powi(decade);
        for step in 1..=60 {
            let u = BRIDGE_SCALE * 3.0 * step as f32 / 60.0;
            let i = i_bias * (u / BRIDGE_SCALE).tanh();
            let back = ring.voltage(i, i_bias);
            worst = worst.max(((back - u) / u).abs());
        }
    }
    // Near its asymptote the logarithm's argument is a difference of two
    // nearly equal currents, so a relative error is amplified by
    // 1/(1 − (i/I)²). At `u/k = 3` that is about a hundredfold, and f32's
    // epsilon is 1.19e-7. The bound is that conditioning, not slack.
    let bound = f32::EPSILON / (1.0 - 0.995f32 * 0.995);
    assert!(
        worst <= bound,
        "the bridge's tanh and (G1) disagree by {worst:.3e} relative, past the {bound:.3e} f32 allows"
    );

    // And the small-signal resistances the two laws imply are one law:
    // the bridge's `r = k / I`.
    for decade in 0..4 {
        let i_bias = 1e-6 * 10f32.powi(decade);
        let theirs = BRIDGE_SCALE / i_bias;
        let mine = ring.resistance(i_bias);
        assert!(
            ((mine - theirs) / theirs).abs() <= 1e-6,
            "at {i_bias} A this law gives {mine} Ω and the bridge's {theirs} Ω"
        );
    }

    // The special case is special. The part as drawn is not the bridge,
    // and asking it to be gives an answer that is wrong by the factor the
    // documentation names.
    let drawn = DiodeArmPair::forward(N_JUNCTIONS);
    let u = 0.34 * BRIDGE_SCALE;
    let i_ring = I * (u / BRIDGE_SCALE).tanh();
    let i_drawn = I * (u / (2.0 * drawn.v_n)).tanh();
    assert!(
        i_ring > i_drawn * 1.9,
        "at {u} V the ring passes {i_ring:.4e} A and the drawn pair {i_drawn:.4e} A; two junctions \
         per arm must roughly halve the tanh argument"
    );
}

#[test]
fn the_junction_scale_is_half_the_bridges_and_that_is_the_trap() {
    // **Identity, arithmetic.** Both crates are built on the same
    // published η and V_T, so the numbers line up and the reasons do not.
    // A two-junction arm's `v_n` equals the bridge's whole thermal scale
    // to the bit, and anyone matching constants will read that as
    // agreement. It is not: `v_n` is half the denominator of this law's
    // tanh argument and the bridge's scale is all of it.
    assert!(
        (JUNCTION_SCALE - 0.045_4).abs() <= 0.000_1,
        "one junction's scale is {JUNCTION_SCALE} V; η·V_T is about 45.4 mV"
    );
    let drawn = DiodeArmPair::forward(2);
    assert_eq!(
        drawn.v_n, BRIDGE_SCALE,
        "a two-junction arm's v_n and the bridge's thermal scale must be the same number, because \
         the coincidence is the thing being warned about"
    );
    // The consequence, which is what stops the two being interchangeable:
    // the drawn pair's own tanh scale is twice the bridge's.
    assert_eq!(2.0 * drawn.v_n, 2.0 * BRIDGE_SCALE);
}

#[test]
fn two_junctions_per_arm_quarter_the_third_harmonic_at_equal_drive() {
    // **Derived, not measured.** For `tanh(a·sinθ)` the third-harmonic
    // ratio is `a²/12`, and `a = û / (2·V_n)`, so doubling the junctions
    // per arm halves `a` and quarters the ratio. "At equal drive" means at
    // equal voltage across the part, which is the only comparison the part
    // alone can make; holding the current equal instead would compare two
    // different working points.
    const N: usize = 4096;
    const U_PK: f32 = 0.02;
    let mut ratios = [0.0f64; 2];
    for (slot, n) in [1u32, 2].iter().enumerate() {
        let e = DiodeArmPair::forward(*n);
        let x = forward_current_period(&e, U_PK, N);
        ratios[slot] = harmonic(&x, 3) / harmonic(&x, 1);
        // And each one against the expansion it comes from.
        let a = (U_PK / (2.0 * e.v_n)) as f64;
        let want = a * a / 12.0;
        assert!(
            (ratios[slot] / want - 1.0).abs() <= 0.02,
            "at {n} junctions per arm the third harmonic is {:.4e} against the a²/12 expansion's \
             {want:.4e}",
            ratios[slot]
        );
    }
    let got = ratios[0] / ratios[1];
    assert!(
        (got - 4.0).abs() <= 0.05,
        "one junction per arm distorts {got:.4} times as much as two; the expansion requires 4.00"
    );
}

#[test]
fn the_law_is_odd_when_the_arms_match() {
    // **Derived**, and corroborated independently: Pines reaches the same
    // conclusion for a symmetric diode gain element, that it "is an odd
    // function … therefore, only odd harmonics are present". So a matched
    // pair makes no even order at all, and every even harmonic a real
    // module produces comes from its amplifiers, its coupling or from the
    // mismatch the next test covers.
    for e in [
        DiodeArmPair::breakdown(),
        DiodeArmPair::forward(2),
        DiodeArmPair::ring(),
    ] {
        for frac in [0.001f32, 0.01, 0.1, 0.5, 0.9, 0.999] {
            let i = frac * I;
            let p = e.voltage(i, I);
            let n = e.voltage(-i, I);
            // The bound carries a 1/frac factor because that is what the
            // arithmetic does, not because the law is less odd there: at a
            // small signal current the logarithm is taken of a ratio close
            // to one, and its relative precision falls off in proportion
            // to how close. That is conditioning, and stating it is
            // cheaper than a tolerance chosen to fit.
            let bound = 4.0 * f32::EPSILON / frac;
            assert!(
                (p + n).abs() <= p.abs() * bound,
                "not odd at i/I={frac}: u(i)={p}, u(-i)={n}, past the {bound:.3e} f32 allows"
            );
        }
    }
}

#[test]
fn mismatch_reintroduces_even_order_and_does_so_monotonically() {
    // **Direction and ordering, not a level.** EMI specify two matched
    // pairs on two separate drawings and fit two adjust-on-test resistors
    // to trim what is left, which is evidence they knew it mattered, but
    // **no figure is published** for how far out a real pair sits. So this
    // asserts that imbalance raises the second harmonic, that it does so
    // without going backwards, and that at full imbalance the second leads
    // the third — never how much.
    const N: usize = 2048;
    /// Peak signal current as a fraction of the bias current. Small enough
    /// that the second harmonic imbalance makes, which is first order in
    /// the imbalance and second order in the drive, still leads the third
    /// the law makes on its own, which is second order in the drive. Drive
    /// a matched pair hard enough and its own third harmonic buries the
    /// asymmetry, which is a fact about the comparison rather than about
    /// the imbalance.
    const DRIVE: f32 = 0.1;
    let mut last = 0.0f64;
    let mut top = (0.0f64, 0.0f64);
    for step in 0..=10 {
        let mut e = DiodeArmPair::breakdown();
        e.mismatch = 0.05 * step as f32 / 10.0;
        let x: Vec<f64> = (0..N)
            .map(|k| {
                let th = core::f64::consts::TAU * k as f64 / N as f64;
                e.voltage(DRIVE * I * th.sin() as f32, I) as f64
            })
            .collect();
        let h1 = harmonic(&x, 1);
        let h2 = harmonic(&x, 2) / h1;
        let h3 = harmonic(&x, 3) / h1;
        assert!(
            h2 >= last * 0.999,
            "at {:.1} % imbalance the second harmonic fell to {h2:.3e} from {last:.3e}",
            100.0 * e.mismatch
        );
        last = h2;
        top = (h2, h3);
    }
    assert!(
        top.0 > top.1,
        "at full imbalance the second harmonic is {:.3e} and the third {:.3e}; the second must lead",
        top.0,
        top.1
    );
}

#[test]
fn breakdown_bottoms_out_on_its_bulk_resistance_and_forward_does_not() {
    // **Derived from (G1)**, and it is the structural difference between
    // the two readings of the drawing rather than a figure about the
    // hardware: `r = 2·r_b + 2·V_n/I` cannot fall below `2·r_b`, so a
    // divider built on the breakdown reading has a bounded loss while one
    // built on the forward reading has none. **Where** the floor sits
    // depends on a bulk resistance inferred from a drawing and is not
    // asserted; that a floor exists is the finding.
    let b = DiodeArmPair::breakdown();
    let floor = 2.0 * BULK_RESISTANCE;
    assert!(
        (b.resistance(1.0) - floor).abs() <= floor * 0.01
            && (b.resistance(1000.0) - floor).abs() <= floor * 1e-4,
        "breakdown gave {} Ω at 1 A and {} Ω at 1000 A against a {floor} Ω floor",
        b.resistance(1.0),
        b.resistance(1000.0)
    );
    let f = DiodeArmPair::forward(2);
    assert!(
        f.resistance(1000.0) < f.resistance(1.0) * 1e-2,
        "forward gave {} Ω at 1 A and {} Ω at 1000 A; with no bulk term it must keep falling",
        f.resistance(1.0),
        f.resistance(1000.0)
    );
}

#[test]
fn the_resistance_inverts_and_refuses_what_the_floor_forbids() {
    // **Identity.** `current_for_resistance` is the algebraic inverse of
    // `resistance`, so a round trip must return the current it started
    // with, including with the arms out of balance. Below the floor there
    // is no such current, and the `None` is the answer rather than an
    // infinity — the difference from the bridge crate, whose `k / I`
    // reaches every resistance above zero.
    for e in [
        DiodeArmPair::breakdown(),
        DiodeArmPair::forward(2),
        DiodeArmPair {
            mismatch: 0.05,
            ..DiodeArmPair::breakdown()
        },
    ] {
        for decade in 0..5 {
            let i_bias = 1e-6 * 10f32.powi(decade);
            let r = e.resistance(i_bias);
            let back = e
                .current_for_resistance(r)
                .expect("a resistance the part reached must be reachable");
            assert!(
                ((back - i_bias) / i_bias).abs() <= 1e-4,
                "{i_bias} A gave {r} Ω and came back as {back} A"
            );
        }
    }
    let b = DiodeArmPair::breakdown();
    assert!(
        b.current_for_resistance(2.0 * BULK_RESISTANCE).is_none(),
        "the floor itself must be unreachable, not reached at an infinite current"
    );
    assert!(
        b.current_for_resistance(BULK_RESISTANCE).is_none(),
        "a resistance below the bulk floor must be refused"
    );
}

#[test]
fn the_slope_matches_a_numerical_derivative() {
    // **Consistency**, and it earns its place because a caller's Newton
    // step uses `slope` to solve an equation written with `voltage`: if
    // the two ever disagree the solve converges to the wrong answer
    // quietly.
    for e in [DiodeArmPair::breakdown(), DiodeArmPair::forward(2)] {
        for frac in [-0.8f32, -0.3, 0.0, 0.3, 0.8] {
            let i = frac * I;
            let h = I * 1e-3;
            let num = (e.voltage(i + h, I) - e.voltage(i - h, I)) / (2.0 * h);
            let ana = e.slope(i, I);
            assert!(
                ((num - ana) / ana).abs() <= 1e-3,
                "at i/I={frac} the slope is {ana} Ω against a numerical {num} Ω"
            );
        }
    }
}

#[test]
fn no_bias_current_is_an_open_circuit_and_so_is_a_broken_one() {
    // A pair carrying nothing is an open circuit, so a machine that puts
    // it in a divider sees unity. The NaN case is asserted because
    // `NaN <= x` is false, so an ordinary floor test lets a broken control
    // value fall through into a logarithm and poison the audio.
    let e = DiodeArmPair::breakdown();
    assert_eq!(e.resistance(0.0), f32::INFINITY);
    assert_eq!(e.resistance(CURRENT_FLOOR), f32::INFINITY);
    assert_eq!(e.resistance(f32::NAN), f32::INFINITY);
    assert!(e.resistance(CURRENT_FLOOR * 10.0).is_finite());
}

#[test]
fn nothing_produces_a_nan_at_the_extremes() {
    // The clamps in `voltage` and `slope` exist so that a signal current
    // driven onto or past an arm's bias current stays finite rather than
    // taking the logarithm of zero.
    for e in [
        DiodeArmPair::breakdown(),
        DiodeArmPair::forward(2),
        DiodeArmPair::ring(),
        DiodeArmPair {
            mismatch: 0.95,
            ..DiodeArmPair::breakdown()
        },
    ] {
        for i_bias in [CURRENT_FLOOR * 10.0, 1e-9, 1e-6, 1e-3, 1.0] {
            for frac in [-2.0f32, -1.0, -0.999, 0.0, 0.999, 1.0, 2.0] {
                let i = frac * i_bias;
                assert!(
                    e.voltage(i, i_bias).is_finite(),
                    "voltage went non-finite at i/I={frac}, I={i_bias}"
                );
                assert!(
                    e.slope(i, i_bias).is_finite() && e.slope(i, i_bias) > 0.0,
                    "slope went non-finite or non-positive at i/I={frac}, I={i_bias}"
                );
            }
        }
    }
}
