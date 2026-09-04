//! Tests for the Blackmer gain cell.
//!
//! Where a figure is published, the test asserts that figure and names its
//! source. Where it is not, the test says so and asserts the thing that
//! *is* established: a bound, an ordering, or an identity that follows
//! from the circuit. An audit of the plug-ins using this repository found
//! nine tests that asserted a model's own output where a published figure
//! existed, one of which compared an estimate with itself, so the rule
//! here is strict: never assert a value this crate produced, and never
//! widen an assertion until it passes.
//!
//! Sources referred to by name below:
//!
//! - **THAT 2180** — THAT Corporation, *THAT 2180 Series Blackmer
//!   Pre-Trimmed IC Voltage Controlled Amplifiers*, datasheet.
//! - **THAT 2150** — the same family's earlier part, whose datasheet
//!   carries the symmetry specification.
//! - **Blackmer** — David E. Blackmer, "Multiplier circuits",
//!   US 3,714,462, filed 14 June 1971, granted 30 January 1973.

use super::*;

/// THAT 2180: gain-control constant 6.1 mV/dB typical, 6.0 to 6.2 across
/// the A, B and C grades.
#[test]
fn control_constant_is_the_published_one() {
    assert_eq!(K_TYP_MV_PER_DB, 6.1);
    let (lo, hi) = K_RANGE_MV_PER_DB;
    assert_eq!((lo, hi), (6.0, 6.2));
    assert!(lo <= K_TYP_MV_PER_DB && K_TYP_MV_PER_DB <= hi);
}

/// THAT 2180: the negative port runs at −6.1 mV/dB and the positive at
/// +6.1 mV/dB. So 6.1 mV on either port is worth exactly one decibel, in
/// opposite directions.
#[test]
fn one_decibel_costs_the_published_millivolts() {
    let c = BlackmerCell::TYPICAL;
    assert!((c.gain_db(0.0, -6.1) - 1.0).abs() < 1e-4);
    assert!((c.gain_db(6.1, 0.0) - 1.0).abs() < 1e-4);
    assert!((c.gain_db(0.0, 6.1) + 1.0).abs() < 1e-4);
}

/// THAT 2180: gain at zero control voltage is 0.0 dB, within ±0.1 dB for
/// the A grade. A trimmed cell should sit exactly there.
#[test]
fn zero_control_voltage_is_unity() {
    assert_eq!(BlackmerCell::TYPICAL.gain_db(0.0, 0.0), 0.0);
    assert!(BlackmerCell::UNTRIMMED.gain_db(0.0, 0.0).abs() > 0.0);
}

/// Blackmer's stated objective: "a constant decibels per volt control
/// characteristic". The law must be a straight line, so equal voltage
/// steps must buy equal decibels anywhere in the range.
#[test]
fn the_law_is_constant_decibels_per_volt() {
    let c = BlackmerCell::TYPICAL;
    let step = |v: f32| c.gain_db(0.0, v + 10.0) - c.gain_db(0.0, v);
    let reference = step(-300.0);
    for v in [-200.0, -100.0, 0.0, 100.0, 200.0] {
        assert!((step(v) - reference).abs() < 1e-5, "not linear at {v} mV");
    }
}

/// THAT 2180: off isolation at 1 kHz is 110 dB minimum and 115 dB typical
/// with the positive port at −360 mV and the negative at +360 mV. The
/// control law alone must already reach that, since nothing else in the
/// part is doing it.
#[test]
fn off_isolation_meets_the_published_minimum() {
    let attenuation = -BlackmerCell::TYPICAL.gain_db(-360.0, 360.0);
    assert!(
        attenuation >= 110.0,
        "off isolation {attenuation:.1} dB, published minimum 110 dB"
    );
}

/// THAT 2180: gain range greater than 130 dB.
#[test]
fn gain_range_reaches_the_published_span() {
    let c = BlackmerCell::TYPICAL;
    let top = c.gain_db(0.0, -CONTROL_SPAN_DB.1 * K_TYP_MV_PER_DB);
    let bottom = c.gain_db(0.0, -CONTROL_SPAN_DB.0 * K_TYP_MV_PER_DB);
    assert!((top - bottom - 100.0).abs() < 1e-2);
    assert_eq!(GAIN_RANGE_DB, 130.0);
}

/// THAT 2180: the gain-control temperature coefficient is +0.33 %/°C,
/// referenced to a chip at 27 °C.
///
/// It is also checked against the physics, which is the stronger
/// statement. A junction's thermal voltage is proportional to absolute
/// temperature, so a control law built out of junctions scales the same
/// way, and the coefficient at the reference should be one over the
/// reference in kelvin. That is 1/300.15 K = 0.3332 %/°C, which is what
/// 0.33 %/°C is a two-figure rounding of. The datasheet number and the
/// derivation agree, and neither was fitted to the other.
#[test]
fn tempco_matches_the_datasheet_and_the_physics() {
    assert_eq!(TEMPCO_PER_C, 0.0033);
    let from_physics = 1.0 / (TEMPCO_REF_C + 273.15);
    assert!(
        (from_physics - TEMPCO_PER_C).abs() < 4e-5,
        "physics gives {from_physics:.6}/°C, datasheet {TEMPCO_PER_C}/°C"
    );
}

/// A ten degree rise makes every decibel of the control law 3.3 % bigger,
/// so a fixed control voltage buys 3.3 % *less* gain. The sign is the
/// point of this test: the constant is millivolts per decibel, and it is
/// easy to apply it upside down.
#[test]
fn ten_degrees_moves_the_law_by_three_point_three_percent() {
    let cold = BlackmerCell::TYPICAL;
    let warm = BlackmerCell {
        temp_c: TEMPCO_REF_C + 10.0,
        ..cold
    };
    let ratio = warm.k_at_temp() / cold.k_at_temp();
    assert!((ratio - 1.033).abs() < 1e-4, "law moved by {ratio}");

    // A fixed control voltage buys less *change* when the cell is warm,
    // and the sign of the change is set by which port is driven. So the
    // invariant is on the magnitude: a warm cell moves the gain less far
    // from unity, whichever way it is going.
    for ec_minus in [100.0, -100.0] {
        let (c_g, w_g) = (cold.gain_db(0.0, ec_minus), warm.gain_db(0.0, ec_minus));
        assert!(
            w_g.abs() < c_g.abs(),
            "warm cell moved further at {ec_minus} mV"
        );
        assert!((c_g / w_g - 1.033).abs() < 1e-3);
    }
}

/// THAT 2180: control-law linearity 0.5 % typical, 2 % maximum, over the
/// 100 dB span. The **magnitude** is published; the shape of the bow is
/// this crate's estimate and is documented as one, so this test asserts
/// only the published bound and never the curve.
#[test]
fn linearity_error_stays_inside_the_published_bound() {
    let (lo, hi) = CONTROL_SPAN_DB;
    let span = hi - lo;
    for (pct, bound) in [(LINEARITY_TYP_PCT, 0.5), (LINEARITY_MAX_PCT, 2.0)] {
        let c = BlackmerCell {
            linearity_pct: pct,
            ..BlackmerCell::TYPICAL
        };
        let mut worst = 0.0f32;
        let mut g = lo;
        while g <= hi {
            worst = worst.max(c.linearity_error_db(g).abs());
            g += 0.5;
        }
        let allowed = bound / 100.0 * span;
        assert!(worst <= allowed + 1e-4, "{worst} dB exceeds {allowed} dB");
    }
    // The ideal cell is exactly ideal, so a caller who wants no invention
    // gets none.
    assert_eq!(BlackmerCell::TYPICAL.linearity_error_db(-30.0), 0.0);
}

/// THAT 2150: an untrimmed A-grade cell's symmetry control voltage lies
/// within ±1.6 mV. Through the published 6.1 mV/dB that is 0.26 dB of
/// offset, which is arithmetic from two published figures rather than a
/// third measurement, and is asserted as such.
#[test]
fn symmetry_offset_follows_from_the_published_window() {
    let c = BlackmerCell {
        symmetry_mv: SYMMETRY_WINDOW_MV,
        ..BlackmerCell::TYPICAL
    };
    let offset = c.gain_db(0.0, 0.0);
    let expected = -SYMMETRY_WINDOW_MV / K_TYP_MV_PER_DB;
    assert!((offset - expected).abs() < 1e-5);
    assert!((offset + 0.262).abs() < 1e-3, "offset {offset:.4} dB");
}

/// The inverse must actually invert, for a cell with no bow.
#[test]
fn control_voltage_round_trips() {
    let c = BlackmerCell {
        symmetry_mv: 0.9,
        ..BlackmerCell::TYPICAL
    };
    for want in [-40.0, -12.0, 0.0, 6.0, 20.0] {
        let v = c.control_mv_for_gain(want);
        assert!(
            (c.gain_db(0.0, v) - want).abs() < 1e-3,
            "{want} dB round trip"
        );
    }
}

/// Second-harmonic distortion of a sine, measured rather than asserted
/// from the coefficient.
///
/// Coherently sampled over exactly one period, so the correlation is
/// exact and no window is needed.
fn measured_thd2(cell: &BlackmerCell, peak: f32) -> f32 {
    const N: usize = 4096;
    let (mut h1, mut h2) = ((0.0f64, 0.0f64), (0.0f64, 0.0f64));
    for n in 0..N {
        let phase = core::f64::consts::TAU * n as f64 / N as f64;
        let y = f64::from(cell.process(peak * (phase.sin() as f32)));
        h1.0 += y * phase.sin();
        h1.1 += y * phase.cos();
        h2.0 += y * (2.0 * phase).sin();
        h2.1 += y * (2.0 * phase).cos();
    }
    let mag = |c: (f64, f64)| c.0.hypot(c.1);
    (mag(h2) / mag(h1)) as f32
}

/// THAT 2180: total harmonic distortion with no external trim, at 0 dBV
/// in, 0 dB gain, 1 kHz: 0.005 %, 0.010 % and 0.030 % typical for the A,
/// B and C grades.
///
/// This runs a real sine through the cell and measures the second
/// harmonic, rather than reading back the coefficient that produced it,
/// which is the only version of this test worth having.
///
/// **Both residual shapes are checked against the same published figure.**
/// That is the point of carrying two: the magnitude is published, so
/// either shape must reproduce it at the published condition, and what
/// separates them is what happens away from that condition.
#[test]
fn distortion_matches_the_published_figure() {
    for residual in [EvenResidual::HalfPathMismatch, EvenResidual::Squarer] {
        for grade in [GRADE_A, GRADE_B, GRADE_C] {
            let published = THD_UNTRIMMED.unity_gain_0dbv[grade];
            let cell = BlackmerCell {
                thd_unity: published,
                residual,
                ..BlackmerCell::TYPICAL
            };
            let measured = measured_thd2(&cell, DBV_PEAK_VOLTS);
            assert!(
                (measured - published).abs() < published * 0.01,
                "{residual:?}, grade {grade}: measured {measured:.6}, \
                 published {published:.6}"
            );
        }
    }
}

/// The coefficient and the published figure are inverses of each other,
/// for both shapes and at a reference amplitude that is not one.
///
/// A round trip rather than a value: the arithmetic relating a
/// second-harmonic ratio to a polynomial coefficient is the part of this
/// crate a caller is most likely to reimplement by hand and get wrong by a
/// factor of two or of `4/(3π)`.
#[test]
fn the_coefficient_and_the_published_figure_invert() {
    for residual in [EvenResidual::HalfPathMismatch, EvenResidual::Squarer] {
        for peak in [0.1024f32, DBV_PEAK_VOLTS, 10.0] {
            for thd in [0.00005f32, 0.00075, 0.03] {
                let c = residual.coefficient_for_thd(thd, peak);
                let back = residual.thd_for_coefficient(c, peak);
                assert!(
                    (back - thd).abs() < thd * 1e-5,
                    "{residual:?} at peak {peak}: {thd} went round to {back}"
                );
            }
        }
    }
}

/// The one behaviour that separates the two shapes: halving the level
/// halves the squarer's relative second harmonic and leaves the
/// mismatch's alone.
///
/// This is the difference a caller is choosing between, so it is asserted
/// rather than described. Neither branch is a published figure and neither
/// is claimed to be: what is published is the magnitude at one condition,
/// which the test above checks, and the slope away from it is exactly the
/// thing no datasheet settles.
#[test]
fn the_two_shapes_differ_only_in_how_they_follow_level() {
    let at = |residual: EvenResidual, peak: f32| {
        measured_thd2(
            &BlackmerCell {
                thd_unity: 0.001,
                residual,
                ..BlackmerCell::TYPICAL
            },
            peak,
        )
    };
    for residual in [EvenResidual::HalfPathMismatch, EvenResidual::Squarer] {
        let loud = at(residual, DBV_PEAK_VOLTS);
        let quiet = at(residual, DBV_PEAK_VOLTS * 0.5);
        let want = if residual.thd_varies_with_level() {
            0.5
        } else {
            1.0
        };
        assert!(
            (quiet / loud - want).abs() < 1e-3,
            "{residual:?}: 6 dB down moved the second harmonic by \
             {:.4}, wanted {want}",
            quiet / loud
        );
    }
}

/// Both shapes emit a direct-current term, and
/// [`BlackmerCell::process_coupled`] is where a caller takes it out.
///
/// This is the crate's boundary in one test. The offset is real, the part
/// produces it, and the coupling capacitor that removes it is downstream
/// and belongs to the machine — so the term is emitted here and the seam
/// to remove it is offered here, and the filter is not.
#[test]
fn the_residual_emits_direct_current_and_the_seam_removes_it() {
    const N: usize = 4096;
    for residual in [EvenResidual::HalfPathMismatch, EvenResidual::Squarer] {
        let cell = BlackmerCell {
            thd_unity: 0.01,
            residual,
            ..BlackmerCell::TYPICAL
        };
        let peak = DBV_PEAK_VOLTS;
        let sample =
            |n: usize| peak * ((core::f64::consts::TAU * n as f64 / N as f64).sin() as f32);
        let raw: f64 = (0..N).map(|n| f64::from(cell.process(sample(n)))).sum();
        assert!(
            raw / N as f64 > 1e-4,
            "{residual:?} emitted no offset: {:.3e}",
            raw / N as f64
        );
        // The caller's coupling: the mean of the shape, which is what its
        // capacitor is holding off.
        let dc = (0..N)
            .map(|n| f64::from(residual.shape(sample(n))))
            .sum::<f64>()
            / N as f64;
        let coupled: f64 = (0..N)
            .map(|n| f64::from(cell.process_coupled(sample(n), dc as f32)))
            .sum();
        assert!(
            (coupled / N as f64).abs() < 1e-5,
            "{residual:?} kept an offset after coupling: {:.3e}",
            coupled / N as f64
        );
    }
    // And a zero mean is the same as no coupling at all.
    let c = BlackmerCell::UNTRIMMED;
    assert_eq!(c.process_coupled(0.3, 0.0), c.process(0.3));
}

/// The residual is even-order, because the two halves of the signal go
/// through different transistors and an asymmetric transfer curve is an
/// even-order one. So it must be the *second* harmonic that appears, and
/// the curve must not be odd-symmetric.
#[test]
fn the_residual_is_even_order() {
    let c = BlackmerCell::UNTRIMMED;
    let (up, down) = (c.process(0.5), c.process(-0.5));
    assert!(
        (up + down).abs() > 1e-6,
        "an odd-symmetric residual is the wrong kind of distortion"
    );
    // A trimmed cell is transparent, which is what a null is for.
    assert_eq!(BlackmerCell::TYPICAL.process(0.5), 0.5);
}

/// The squarer's distortion is quoted at an amplitude, so a caller working
/// in something other than volts must say so, and a caller that does say
/// so gets the published figure back.
///
/// This is the field that exists because the two real users work in two
/// different units and neither is volts. A cell told that its published
/// 0.005 % was measured at a peak of 0.1025 in the caller's own scale
/// produces 0.005 % there — not at √2, which is where a crate that assumed
/// volts would have put it, and which is a factor of fourteen away.
#[test]
fn the_published_figure_holds_in_the_callers_own_units() {
    let published = THD_UNTRIMMED.unity_gain_0dbv[GRADE_A];
    // A console whose unit sample amplitude is 13.794 V: √2 volts of
    // datasheet condition is 0.10252 of full scale.
    let peak = DBV_PEAK_VOLTS / 13.794;
    let cell = BlackmerCell {
        thd_unity: published,
        thd_peak: peak,
        residual: EvenResidual::Squarer,
        ..BlackmerCell::TYPICAL
    };
    let measured = measured_thd2(&cell, peak);
    assert!(
        (measured - published).abs() < published * 0.01,
        "measured {measured:.7} in the caller's units, published {published:.7}"
    );
    // And the mismatch shape needs no such declaration, because its
    // relative second harmonic is the same everywhere.
    let flat = BlackmerCell {
        residual: EvenResidual::HalfPathMismatch,
        ..cell
    };
    for p in [peak, DBV_PEAK_VOLTS, 100.0] {
        let m = measured_thd2(&flat, p);
        assert!(
            (m - published).abs() < published * 0.01,
            "at peak {p} the mismatch shape gave {m:.7}"
        );
    }
}

/// A miss, recorded rather than hidden.
///
/// THAT publish three distortion conditions and the two away from unity
/// gain are worse than the one at it: 0.030 % against 0.010 % for the B
/// grade. This crate models only the unity-gain figure, because three
/// points do not establish a surface and fitting one through them would
/// replace a published measurement with an invention. So a caller running
/// the cell at high gain reduction gets less distortion than the part
/// really produces. The published table is carried in full so that
/// whoever closes this gap has the numbers to hand.
#[test]
fn gain_dependent_distortion_is_a_known_gap() {
    let t = THD_UNTRIMMED;
    for g in [GRADE_A, GRADE_B, GRADE_C] {
        assert!(t.minus_15db_gain_plus_10dbv[g] >= t.unity_gain_0dbv[g]);
        assert!(t.plus_15db_gain_minus_5dbv[g] >= t.unity_gain_0dbv[g]);
    }
    assert_eq!(t.unity_gain_0dbv[GRADE_B], 0.00010);
    assert_eq!(t.minus_15db_gain_plus_10dbv[GRADE_B], 0.00030);

    // And the shape of the table, which is the part [`EvenResidual`] cites.
    // THAT's two conditions away from unity gain are published with the
    // *same* distortion although one raises the input 10 dB above the
    // reference and the other lowers it 5 dB below: the table is symmetric
    // in gain deviation and says nothing consistent about input level.
    for g in [GRADE_A, GRADE_B, GRADE_C] {
        assert_eq!(
            t.minus_15db_gain_plus_10dbv[g], t.plus_15db_gain_minus_5dbv[g],
            "grade {g}: the two off-unity rows are published equal"
        );
    }
}
