//! Tests for the remote-cutoff triode.
//!
//! Where a figure is published, the test asserts that figure and names its
//! source. Where it is not, the test says so rather than dressing an estimate
//! up as a measurement, and asserts the thing that *is* established: a
//! direction, an ordering or a shape. That distinction carries most of the
//! weight here, because the two failures this component exists to remember —
//! a law fitted to current whose slope was never checked, and an exponent
//! that moves by a factor of 1.7 across one valve's own operating
//! conditions — were both invisible to a test that compares a model with
//! itself.

use super::*;

/// General Electric ET-T1113 page 5, "AVERAGE PLATE CHARACTERISTICS, EACH
/// SECTION", read at 250 V of plate off a 400 dpi render by locating the dark
/// runs in that column. Nine curves are cleanly separated below the label
/// clutter and there are exactly nine grid values between −12 and −70, so the
/// assignment is forced rather than judged. The deepest two sit within fifty
/// pixels of the baseline and are the softest readings here, ±20 % at −50 V
/// and worse at −70.
const GE_PLATE_250V: [(f32, f32); 9] = [
    (-12.0, 18.26),
    (-14.0, 14.30),
    (-17.0, 11.55),
    (-20.0, 8.85),
    (-25.0, 7.10),
    (-30.0, 5.14),
    (-40.0, 3.61),
    (-50.0, 1.60),
    (-70.0, 0.60),
];

/// GE's Class A₁ operating point, ET-T1113 "Characteristics and Typical
/// Operation, Class A₁ Amplifier, Each Section": plate 100 V, cathode
/// resistor 200 Ω, plate current 9.6 mA, so `Vgk` = −1.92 V.
const CLASS_A1: (f32, f32, f32) = (-1.92, 100.0, 9.6);

/// The least-squares cost the refit was done under: the sum of squared
/// natural-log residuals over the nine plate readings plus the tabulated
/// class-A₁ current. In f64 because a sum of squares in f32 is not what was
/// minimised.
fn cost(p: &ValveParams) -> f64 {
    let t = RemoteCutoffTriode::new(*p);
    let mut s = 0.0;
    for (vgk, want) in GE_PLATE_250V {
        let e = (t.anode_current(vgk, 250.0) as f64 * 1e3 / want as f64).ln();
        s += e * e;
    }
    let (vgk, vak, want) = CLASS_A1;
    let e = (t.anode_current(vgk, vak) as f64 * 1e3 / want as f64).ln();
    s + e * e
}

/// **A fit residual and not an independent check, which is the point of
/// saying so.**
///
/// *Published:* GE's plate characteristics at 250 V, the nine readings above.
/// `p1` and `p8` were fitted to exactly these, so what this measures is how
/// well that fit closed, not whether the curve is right. It earns its place
/// anyway: it fails if a constant is edited, if the clamp moves or if the
/// evaluation drifts.
#[test]
fn the_law_reproduces_the_curve_it_was_fitted_to() {
    let t = RemoteCutoffTriode::ge_6386();
    let mut sq = 0.0f32;
    for (vgk, want) in GE_PLATE_250V {
        let got = t.anode_current(vgk, 250.0) * 1e3;
        let err = 20.0 * (got / want).log10();
        sq += err * err;
        // A guard rather than a measurement: no single reading may be a
        // factor of 1.4 out, which would mean a constant had been edited or
        // the clamp had moved rather than that the fit had drifted.
        assert!(
            err.abs() < 3.0,
            "Ia({vgk} V, 250 V) = {got:.3} mA against GE's {want} mA, {err:+.2} dB"
        );
    }
    let rms = (sq / GE_PLATE_250V.len() as f32).sqrt();
    // The fit was done in double precision and callers run in single, so the
    // allowance is a quarter of a decibel over the recorded residual.
    let recorded = t.params().fit_residual_db;
    assert!(
        rms <= recorded + 0.25,
        "the fit residual is {rms:.2} dB against the {recorded:.2} dB recorded at the constants"
    );
}

/// *Published:* the same nine readings, against **Raffensperger's equation as
/// he published it**.
///
/// This is the measurement that says why one of his parameters was changed.
/// It is an assertion about a published equation rather than about this
/// model, so anybody with the datasheet can check it.
///
/// | Vgk | GE | as published |
/// |---|---|---|
/// | −20 V | 8.85 mA | −1.01 dB |
/// | −40 V | 3.61 mA | **−4.83 dB** |
/// | −50 V | 1.60 mA | **−9.14 dB** |
/// | −70 V | 0.60 mA | **−37.28 dB** |
///
/// A remote-cutoff valve still passing half a milliamp at −70 V is the point
/// of the type. A variable-mu limiter's grids reach that voltage at the
/// deepest limiting its own published static curves show, so this is the
/// working range and not a tail.
#[test]
fn the_published_fit_cuts_the_valve_off_far_too_early() {
    let t = RemoteCutoffTriode::new(ValveParams::GE_6386_AS_PUBLISHED);
    for (vgk, want, least_db) in [
        (-40.0f32, 3.61f32, 3.0f32),
        (-50.0, 1.60, 6.0),
        (-70.0, 0.60, 20.0),
    ] {
        let got = t.anode_current(vgk, 250.0) * 1e3;
        let err = 20.0 * (got / want).log10();
        assert!(
            err < -least_db,
            "as published the equation gives {got:.4} mA at {vgk} V against GE's {want} mA, \
             which is {err:+.1} dB; it has to be at least {least_db} dB low for the correction \
             to `p8` to be justified"
        );
    }
    // And it is not low where it was checked: shallower than about −30 V the
    // exponential term is negligible and the published fit is good, which is
    // exactly why three points on a linear plot did not catch this.
    let got = t.anode_current(-20.0, 250.0) * 1e3;
    assert!(
        (20.0 * (got / 8.85f32).log10()).abs() < 2.0,
        "as published the equation gives {got:.3} mA at −20 V against GE's 8.85; the error is \
         supposed to be confined to the deep end"
    );
}

/// *Published:* GE's nine plate readings and their tabulated class-A₁
/// current, which are what both sets are scored against.
///
/// The refit was recorded as taking the least-squares cost from **20.05 to
/// 0.09**. This recomputes it, so the recorded figure is a measurement in the
/// repository rather than a number in a paragraph.
#[test]
fn the_refit_costs_two_orders_of_magnitude_less_against_ges_own_curve() {
    let published = cost(&ValveParams::GE_6386_AS_PUBLISHED);
    let corrected = cost(&ValveParams::GE_6386);
    assert!(
        (published - 20.05).abs() < 0.5,
        "the published set scores {published:.3} against the 20.05 recorded"
    );
    assert!(
        (corrected - 0.09).abs() < 0.05,
        "the corrected set scores {corrected:.4} against the 0.09 recorded"
    );
    assert!(
        published / corrected > 100.0,
        "the refit is supposed to be worth two orders of magnitude: {published:.3} to \
         {corrected:.4}"
    );
}

/// **A recorded miss.**
///
/// *Published:* GE's ET-T1113 Class A₁ block: transconductance **4000 µmhos**
/// at the operating point and "Grid Voltage, approximate, Gm = 100 Micromhos,
/// **−16 Volts**". That is a fall of forty times, **32.0 dB**, over 14.08 V of
/// grid, and it is the figure that would catch a wrong valve model before
/// anything else did.
///
/// | | published | this law |
/// |---|---|---|
/// | gm at the class-A₁ point | 4000 µmho | **2606 µmho** |
/// | gm at −16 V | 100 µmho | **124 µmho** |
/// | range | **32.0 dB ± 3** | **26.44 dB** |
///
/// The gap is printed rather than closed, and the crate documentation says
/// why: the two tabulated points sit at 100 V of plate, a limiter's stage runs
/// at 216 to 230 V, and extrapolating an exponential anchored on them gives
/// about 110 dB of control authority where the unit has 20.
///
/// What is asserted instead is what both readings agree on, and what makes
/// this a remote-cutoff valve at all: the transconductance falls
/// **monotonically** across the published interval, and by more than a decade
/// over it. An ordinary triode would have cut off entirely inside it.
#[test]
fn transconductance_falls_monotonically_over_the_published_interval() {
    let t = RemoteCutoffTriode::ge_6386();
    let (vgk0, vak, _) = CLASS_A1;
    let mut last = f32::INFINITY;
    for k in 0..=32 {
        let vgk = vgk0 - 14.08 * k as f32 / 32.0;
        let gm = t.transconductance(vgk, vak);
        assert!(
            gm < last,
            "transconductance rose at Vgk = {vgk:.2} V; a remote-cutoff valve's falls all the \
             way down"
        );
        last = gm;
    }
    let range = 20.0 * (t.transconductance(vgk0, vak) / t.transconductance(-16.0, vak)).log10();
    assert!(
        range >= 20.0,
        "the control range is {range:.2} dB over the datasheet's own interval; GE publish 32.0 \
         and this law reaches 26.4, so what is asserted here is a decade of transconductance \
         rather than the published figure"
    );
}

/// *Published:* GE tabulate, for the Class A₁ amplifier, each section:
/// amplification factor **17**, plate resistance **4250 Ω**, transconductance
/// **4000 µmhos**. Those three are not independent — `μ = gm · rp` — so the
/// block can be checked against itself, and it closes: 17 / 4250 is 4000 µmho
/// on the nose.
///
/// It also closes against the *curves*. The amplification factor is `dVp/dVg`
/// at constant plate current, which is the horizontal spacing of the grid
/// curves on page 5 — a far easier reading than a current near the baseline.
/// Measured off the 400 dpi render at 10 mA, the 0 V curve crosses at 75 V and
/// the −2 V curve at 108 V, so μ = 16.5, and 16.5 / 4250 is 3880 µmho against
/// a tabulated 4000. Two independent readings of one document agreeing is the
/// strongest statement available about this valve, because there is no second
/// manufacturer to disagree.
///
/// **This law gives μ = 10.4 at that point, and that is recorded rather than
/// fixed**, for the reason at [`RemoteCutoffTriode::mu`]. Its plate resistance
/// is the closest of the three, 3974 Ω against 4250.
#[test]
fn the_datasheet_closes_on_itself_and_the_law_does_not_reach_it() {
    let (mu, rp, gm) = (17.0f32, 4250.0f32, 4000e-6f32);
    assert!(
        ((mu / rp) / gm - 1.0).abs() < 0.02,
        "GE's tabulated block does not close: 17 / 4250 = {:.0} µmho against a tabulated 4000",
        mu / rp * 1e6
    );
    // The same figure read off the curve spacing at 10 mA: 0 V at 75 V of
    // plate, −2 V at 108 V.
    let measured = (108.0f32 - 75.0) / 2.0;
    assert!(
        (measured / mu - 1.0).abs() < 0.10,
        "the curve spacing gives μ = {measured:.1} against a tabulated 17"
    );
    let t = RemoteCutoffTriode::ge_6386();
    let (vgk, vak, _) = CLASS_A1;
    let model_mu = t.mu(vgk, vak);
    assert!(
        model_mu > 5.0 && model_mu < mu,
        "the law's amplification factor is {model_mu:.2} at the class-A₁ point; GE tabulate 17 \
         and the curve spacing measures 16.5. The functional form cannot reach it — but if this \
         ever exceeds the published figure, something has changed that a caller dividing a load \
         against a plate resistance would care about"
    );
    let model_rp = t.plate_resistance(vgk, vak);
    assert!(
        (model_rp / rp - 1.0).abs() < 0.15,
        "the law's plate resistance is {model_rp:.0} Ω against GE's tabulated 4250"
    );
}

/// *Published:* JJ Electronic's typical characteristics for the 6386 LGP at
/// the same operating point GE use — `Ua = 100 V, Rk = 200 Ω, Ia = 9.6 mA` —
/// with `S = 3 mA/V` against GE's 4 mA/V. A ratio of 0.75, which is
/// **−2.50 dB**.
///
/// The replacement is carried as the GE curve stretched along the grid axis,
/// so what this asserts is what the two datasheets actually agree and
/// disagree on at the one point where both quote a figure: the same anode
/// current, and three quarters of the slope. **Everywhere else the JJ's shape
/// is an assumption**, and there is no published curve to test it against.
#[test]
fn the_replacement_valve_differs_by_the_published_slope_ratio() {
    let ge = RemoteCutoffTriode::ge_6386();
    let jj = RemoteCutoffTriode::jj_6386_lgp();
    let (vgk, vak, _) = CLASS_A1;
    assert_eq!(
        jj.anode_current(vgk, vak),
        ge.anode_current(vgk, vak),
        "both datasheets quote 9.6 mA at this point, so the two parts must carry the same \
         current here"
    );
    let db = 20.0 * (jj.transconductance(vgk, vak) / ge.transconductance(vgk, vak)).log10();
    assert!(
        (db + 2.499).abs() < 0.05,
        "the JJ's transconductance is {db:+.3} dB against the GE's; the two datasheets publish \
         3 mA/V against 4 mA/V at this point, which is −2.50 dB"
    );
}

/// *No published figure:* the clamp is a property of the fitted law rather
/// than of any valve, so what is asserted is the shape it has to have.
///
/// `(p3 − p4·Vgk)` reaches zero at +5 V for every set here and the expression
/// blows up there, so the clamp must sit well below it; above the clamp the
/// current has to freeze and the grid slope has to be zero, which is what a
/// clamp means.
#[test]
fn the_clamp_sits_below_the_pole_and_freezes_the_current() {
    for p in [
        ValveParams::GE_6386,
        ValveParams::GE_6386_AS_PUBLISHED,
        ValveParams::JJ_6386_LGP,
    ] {
        let name = p.name;
        let pole = p.grid_singularity();
        assert!(
            (pole - 5.0).abs() < 1e-4,
            "{name}: the pole is at {pole} V, not the +5 V the documentation states"
        );
        assert!(
            p.vgk_clamp < pole - 5.0,
            "{name}: the clamp at {} V is not well below the pole at {pole} V",
            p.vgk_clamp
        );
        let t = RemoteCutoffTriode::new(p);
        // The clamp is on the *stretched* grid axis, so the raw voltage that
        // reaches it differs between sets.
        let at_clamp = (p.vgk_clamp - p.grid_offset) / p.grid_scale;
        let (ia_at, dg_at, _) = t.slopes(at_clamp, 250.0);
        let (ia_above, dg_above, _) = t.slopes(at_clamp + 3.0, 250.0);
        assert_eq!(
            ia_above, ia_at,
            "{name}: the current is supposed to freeze above the clamp"
        );
        assert_eq!(
            dg_above, 0.0,
            "{name}: the grid slope above the clamp is not zero"
        );
        assert!(
            dg_at > 0.0,
            "{name}: the grid slope at the clamp should still be live"
        );
    }
}

/// *No published figure:* this checks the hand-derived derivatives against
/// finite differences of the current law, which is a check of the arithmetic
/// rather than of the valve. It is worth having because `slopes` returns all
/// three quantities from one evaluation for speed, and that is exactly the
/// kind of code where a shared subexpression drifts silently.
///
/// The tolerance is a per cent, which is loose because the difference itself
/// is: subtracting two single-precision currents a few milliamps apart over a
/// hundredth of a volt keeps only three or four significant digits. A wrong
/// derivative would be out by far more than that — the two terms of the grid
/// slope differ by an order of magnitude across the working range, so
/// dropping either shows up immediately.
#[test]
fn the_analytic_slopes_agree_with_finite_differences() {
    let t = RemoteCutoffTriode::ge_6386();
    for (vgk, vak) in [
        (-5.0f32, 200.0f32),
        (-22.0, 216.0),
        (-50.0, 230.0),
        (-70.0, 250.0),
    ] {
        let h = 1e-2;
        let (_, dg, da) = t.slopes(vgk, vak);
        let ng = (t.anode_current(vgk + h, vak) - t.anode_current(vgk - h, vak)) / (2.0 * h);
        let na = (t.anode_current(vgk, vak + h) - t.anode_current(vgk, vak - h)) / (2.0 * h);
        assert!(
            (dg / ng - 1.0).abs() < 1e-2,
            "∂Ia/∂Vgk at ({vgk} V, {vak} V) is {dg:.6e} analytically against {ng:.6e} numerically"
        );
        assert!(
            (da / na - 1.0).abs() < 1e-2,
            "∂Ia/∂Vak at ({vgk} V, {vak} V) is {da:.6e} analytically against {na:.6e} numerically"
        );
    }
}

/// *Published:* GE's page 3 transconductance plot is a **straight line on a
/// logarithmic axis** over this span, so the real valve's rate of fall does
/// not turn.
///
/// The published fit's does: it dips to 0.12 dB per volt near −39 V and climbs
/// back to 1.9 by −70. **The refit shrinks that turn and pushes it deeper —
/// 0.10 dB per volt near −59 V, up to 0.5 by −70 — but it does not remove
/// it**, because the wobble comes from a power law multiplying an exponential
/// and is a property of the functional form. That is asserted here rather than
/// left in a paragraph, because it is the honest limit of what one refitted
/// parameter bought.
///
/// No figure is published for the size of the residual turn, so what is
/// asserted is the ordering: the corrected law's is smaller, and it sits
/// deeper.
#[test]
fn the_refit_shrinks_the_slopes_wobble_without_removing_it() {
    let rate_turn = |p: ValveParams| {
        let t = RemoteCutoffTriode::new(p);
        let rate = |v: f32| {
            20.0 * (t.transconductance(v, 250.0) / t.transconductance(v - 0.5, 250.0)).log10() * 2.0
        };
        let mut min = (f32::INFINITY, 0.0f32);
        let mut max_after = f32::NEG_INFINITY;
        for k in 0..=1400 {
            let v = -1.0 - k as f32 * 0.05;
            let r = rate(v);
            if r < min.0 {
                min = (r, v);
                max_after = f32::NEG_INFINITY;
            } else if r > max_after {
                max_after = r;
            }
        }
        (min.0, min.1, max_after)
    };
    let (pub_min, pub_at, pub_after) = rate_turn(ValveParams::GE_6386_AS_PUBLISHED);
    let (cor_min, cor_at, cor_after) = rate_turn(ValveParams::GE_6386);
    assert!(
        pub_after > pub_min + 1.0,
        "the published fit is supposed to turn hard: {pub_min:.2} dB/V at {pub_at:.1} V then \
         back up to {pub_after:.2}"
    );
    assert!(
        cor_after - cor_min < (pub_after - pub_min) / 3.0,
        "the refit is supposed to shrink the turn: {cor_min:.2} to {cor_after:.2} dB/V against \
         the published {pub_min:.2} to {pub_after:.2}"
    );
    assert!(
        cor_at < pub_at,
        "the refit is supposed to push the turn deeper: {cor_at:.1} V against {pub_at:.1} V"
    );
    assert!(
        cor_after - cor_min > 0.0,
        "the turn is a property of the functional form and the refit does not remove it; if it \
         has gone, the form has changed and this crate's documentation is wrong"
    );
}
