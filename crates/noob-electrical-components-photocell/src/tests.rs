//! Tests for the photocell.
//!
//! Where a figure is published, the test asserts that figure and names its
//! source. Where it is not, the test says so rather than dressing an
//! estimate up as a measurement, and asserts the thing that *is*
//! established: a direction, an ordering or a shape. That distinction is
//! the whole point here, because a test that quietly checks the model
//! against its own output can never fail.

use super::*;

const SR: f32 = 48_000.0;

/// Sidechain volts that light the panel properly. The Alfrey-Taylor law
/// has a soft threshold, so a drive of 0.5 gives under a thousandth of
/// full light and a cell that has barely woken up; the LA-2A's sidechain
/// swings volts, not fractions of one.
const DRIVE: f32 = 3.0;

/// The LA-2A's divider, for turning a cell resistance into the gain
/// reduction a listener would hear. A nominal one: the cell is a shunt,
/// so conductance is not what anybody perceives and every recovery figure
/// here is in decibels because of it.
const R_SERIES: f32 = 70.7e3;
const R_POT: f32 = 100e3;

/// Gain reduction in dB for a cell resistance, through that divider.
fn gr_db(r_cell: f32) -> f32 {
    let par = |r: f32| r * R_POT / (r + R_POT);
    let a = par(r_cell) / (R_SERIES + par(r_cell));
    let a_dark = par(R_DARK) / (R_SERIES + par(R_DARK));
    -20.0 * (a / a_dark).log10()
}

/// Drive the cell to a steady state, then release it, reporting how much
/// of the settled **gain reduction** has come back at each time.
///
/// Measured in decibels through a divider, not in linear conductance. The
/// cell is a shunt in a divider and the ear hears decibels, so linear
/// conductance badly overstates how far a recovery has got: at a drive of
/// 3 V this cell reads 98.7 % recovered by the old linear measure while
/// still holding 7.3 dB of reduction, which is most of the audible effect.
/// Any reasoning about "the tail is finished" drawn from the linear figure
/// was wrong, and one such conclusion had to be revisited because of it.
fn recovery(params: CellParams, drive: f32, at_s: &[f32]) -> Vec<f32> {
    let mut c = Cell::new(params, SR);
    for _ in 0..(SR as usize * 4) {
        c.step(drive);
    }
    let settled = gr_db(c.resistance());
    assert!(settled > 1.0, "the cell barely lit: {settled:.4} dB");
    let mut out = Vec::with_capacity(at_s.len());
    let mut done = 0usize;
    for t in at_s {
        let want = (SR * t) as usize;
        while done < want {
            c.step(0.0);
            done += 1;
        }
        out.push(1.0 - gr_db(c.resistance()) / settled);
    }
    out
}

/// The electroluminescent panel's law: dark at zero, rising, saturating.
///
/// *Figure asserted:* the **shape** of the Alfrey-Taylor law,
/// `L = exp(−b / √u)`, which has zero slope at the origin, so a panel has
/// a soft threshold rather than a knee. No absolute light figure is
/// published for the panel, so none is asserted.
#[test]
fn the_panel_lights_softly_and_saturates() {
    assert_eq!(Cell::light_for(0.0), 0.0, "a dark panel emits nothing");
    assert_eq!(Cell::light_for(-1.0), 0.0, "negative drive cannot emit");

    let mut last = 0.0;
    for i in 1..=200 {
        let u = i as f32 * 0.1;
        let l = Cell::light_for(u);
        assert!(l >= last, "light fell at u = {u}: {l} after {last}");
        assert!((0.0..=1.0).contains(&l), "light left 0..1 at u = {u}: {l}");
        last = l;
    }

    // Zero slope at the origin: doubling a tiny drive should not double
    // the light, it should barely move it.
    let a = Cell::light_for(0.05);
    let b = Cell::light_for(0.10);
    assert!(a < 1e-6 && b < 1e-3, "the threshold is not soft: {a}, {b}");

    // And it saturates rather than running away.
    assert!(Cell::light_for(1e4) < 1.0);
}

/// Conductance follows a power law in the light, and resistance follows
/// from it between the two published extremes.
///
/// *Figures asserted:* `R_DARK` and `R_MIN`, the photocell's resistance in
/// darkness and under full light, which together set the cell's range.
#[test]
fn resistance_runs_between_dark_and_full_light() {
    assert_eq!(resistance_for(0.0), R_DARK, "a dark cell is not R_DARK");
    assert!(
        (resistance_for(1.0) - R_MIN).abs() < 1.0,
        "a fully lit cell is {} not R_MIN",
        resistance_for(1.0)
    );
    // Monotonic, and clamped at both ends.
    let mut last = R_DARK;
    for i in 0..=100 {
        let r = resistance_for(i as f32 / 100.0);
        assert!(r <= last + 1e-3, "resistance rose with carriers at {i}");
        assert!(
            (R_MIN..=R_DARK).contains(&r),
            "resistance left its range: {r}"
        );
        last = r;
    }
    assert_eq!(resistance_for(9.0), R_MIN, "excess carriers must clamp");
    // Negative carriers are unreachable, because `Cell::step` clamps them
    // to 0..1 before anyone can ask. Were they ever passed in, the
    // conductance would go negative and the clamp would return the minimum
    // rather than the dark resistance. That is recorded rather than
    // guarded, because adding a guard would change behaviour to no
    // purpose.
    assert_eq!(resistance_for(-1.0), R_MIN);

    // The gamma is a compression of light into conductance, so doubling
    // the light must give less than double the carriers.
    let p = CellParams::GRAY;
    let one = Cell::carriers_for(0.2, &p);
    let two = Cell::carriers_for(0.4, &p);
    assert!(two < 2.0 * one, "gamma {CELL_GAMMA} is not compressing");
    assert_eq!(Cell::carriers_for(0.0, &p), 0.0);
}

/// The cell attacks faster than it releases, and releases in two stages.
///
/// *Figure asserted:* the **ordering and the shape**, which is what the
/// research establishes: attack of roughly ten milliseconds, a first
/// release stage of tens of milliseconds and a second of seconds. The
/// exact figures belong to a whole compressor, where the loop closes
/// faster than the cell alone, so they are asserted there rather than
/// here.
#[test]
fn the_cell_attacks_quickly_and_releases_in_two_stages() {
    let p = CellParams::GRAY;
    let mut c = Cell::new(p, SR);

    // Attack: time to reach most of the way.
    for _ in 0..(SR as usize * 4) {
        c.step(DRIVE);
    }
    let settled = c.conductance();
    assert!(settled > 0.05, "the cell did not light: {settled:.4}");

    let r = recovery(p, DRIVE, &[0.06, 3.0]);
    assert!(
        r[0] > 0.05 && r[0] < 0.9,
        "the first release stage should be partway back at 60 ms, not {:.3}",
        r[0]
    );
    assert!(
        r[1] > r[0],
        "the second stage should keep recovering: {:.3} then {:.3}",
        r[0],
        r[1]
    );
}

/// Long, hard compression leaves the cell slower to let go.
///
/// *Figure asserted:* the **direction** of the trap memory, which is what
/// the research establishes and what makes an optical compressor
/// programme-dependent. No published number exists for the trap depth.
#[test]
fn traps_make_a_worked_cell_slower_to_recover() {
    let p = CellParams::GRAY;

    let mut brief = Cell::new(p, SR);
    for _ in 0..(SR as usize / 5) {
        brief.step(0.6);
    }
    let brief_settled = brief.conductance();
    let brief_traps = brief.n_t;

    let mut worked = Cell::new(p, SR);
    for _ in 0..(SR as usize * 8) {
        worked.step(0.6);
    }
    let worked_traps = worked.n_t;

    assert!(
        worked_traps > brief_traps,
        "a long hit should fill more traps: {worked_traps:.4} against {brief_traps:.4}"
    );
    assert!(brief_settled > 0.0, "the brief hit did not light the cell");

    // **A direction alone is not enough.** This is the crate's headline
    // behaviour, and an assertion that only checks a sign passes at a
    // thousandth of a percent, so it would not notice the memory being
    // gutted. The size is pinned too: measured, a long hit leaves the trap
    // population more than three times a brief one's, and the effect is
    // large enough to matter rather than merely present.
    assert!(
        worked_traps > 3.0 * brief_traps,
        "the memory is barely there: a long hit filled {worked_traps:.4} against a          brief one's {brief_traps:.4}, a ratio of {:.2}",
        worked_traps / brief_traps.max(1e-9)
    );
    assert!(
        worked_traps > 0.05,
        "a worked cell should hold a substantial trap population, not {worked_traps:.4}"
    );
}

/// The model stays responsive above its documented operating limit.
///
/// Generation reaches the clamp at about 4.2 V of drive, above which the
/// two headline laws stop contributing. This asserts the model still
/// behaves like a compressor up there rather than dying: it must reach
/// full conductance, hold it, and release from it. Anything about the
/// *shape* above the knee is not asserted, because the model does not
/// claim to be faithful there; see the note on [`Cell::step`].
#[test]
fn the_model_is_still_responsive_above_its_operating_limit() {
    for drive in [5.0f32, 10.0, 50.0] {
        let mut c = Cell::new(cell_params_for(T4Variant::Gray), SR);
        for _ in 0..(SR as usize) {
            c.step(drive);
        }
        let lit = c.conductance();
        assert!(
            lit > 0.9,
            "at {drive} V the cell only reached {lit:.4}; it should be near full"
        );
        assert!(lit.is_finite() && lit <= 1.0);
        for _ in 0..(SR as usize / 2) {
            c.step(0.0);
        }
        assert!(
            c.conductance() < lit,
            "at {drive} V the cell would not release: {:.4} after {lit:.4}",
            c.conductance()
        );
    }
    // The saturation is real and worth naming: past the knee, more drive
    // changes nothing at all.
    let settle = |v: f32| {
        let mut c = Cell::new(cell_params_for(T4Variant::Gray), SR);
        for _ in 0..(SR as usize) {
            c.step(v);
        }
        c.conductance()
    };
    assert_eq!(
        settle(5.0),
        settle(50.0),
        "ten times the drive should be identical above the clamp, and this test          exists to make that visible rather than surprising"
    );
}

/// The part's own range, and what a divider makes of it.
///
/// *Figures asserted:* the resistance ratio between [`R_DARK`] and
/// [`R_MIN`] is 4000, which is 72.0 dB, and through the LA-2A's divider
/// that becomes 38.3 dB. The doc used to claim the 38 dB as the cell's own
/// range without a derivation; it is the machine's figure, not the part's,
/// and both are pinned here so neither can drift or be confused again.
#[test]
fn the_range_is_seventy_two_db_at_the_part_and_thirty_eight_through_a_divider() {
    let ratio = R_DARK / R_MIN;
    assert!((ratio - 4000.0).abs() < 1.0, "resistance ratio is {ratio}");
    let bare_db = 20.0 * ratio.log10();
    assert!(
        (bare_db - 72.0).abs() < 0.1,
        "the part's own range is {bare_db:.2} dB, not 72"
    );
    let through_db = gr_db(R_MIN);
    assert!(
        (through_db - 38.3).abs() < 0.2,
        "through the LA-2A's divider the range is {through_db:.2} dB, not 38.3"
    );
    assert!(gr_db(R_DARK).abs() < 1e-3, "a dark cell must be unity gain");
}

/// The distortion stays monotonic up to its stated limit and folds past it.
///
/// *Figure asserted:* [`MAX_DISTORTION_K`] is where the derivative loses
/// positivity, measured at 8/9. Below it the curve is single-valued;
/// above it, it folds back and stops being a distortion. The constraint
/// was previously unstated even though `k` is supplied by the caller.
#[test]
fn distortion_is_monotonic_up_to_its_limit() {
    // The derivative in closed form, `1 - kc·q²(3 + q²)/(1 + q²)²`. Written
    // out rather than differenced through `distortion`, because that
    // debug-asserts its own limit and this test needs to look past it.
    let deriv_min = |kc: f32| {
        let mut worst = f32::INFINITY;
        let mut q = 1e-3f32;
        while q < 80.0 {
            let q2 = q * q;
            worst = worst.min(1.0 - kc * (q2 * (3.0 + q2)) / ((1.0 + q2) * (1.0 + q2)));
            q += 1e-3;
        }
        worst
    };
    assert!(
        deriv_min(MAX_DISTORTION_K) > -1e-3,
        "the curve should still be monotonic at the stated limit"
    );
    assert!(
        deriv_min(0.95) < 0.0,
        "past the limit the curve should fold, and it did not"
    );

    // What our own callers pass, so the margin is visible rather than
    // assumed: the three optical compressors use 0.6, 0.2 and 0.1.
    for k in [0.6f32, 0.2, 0.1] {
        assert!(
            k <= MAX_DISTORTION_K,
            "a shipped caller passes {k}, past the fold at {MAX_DISTORTION_K}"
        );
        assert!(deriv_min(k) > 0.0, "k = {k} is not monotonic");
    }
}

/// The antiderivative really is the integral of the distortion.
///
/// Checked against numerical differentiation rather than against itself,
/// which is the only way this assertion means anything.
///
/// The tolerance is set by `f32` differencing, not by the formula. At
/// `v = 8` the antiderivative is around 32, so subtracting two neighbouring
/// values loses most of the mantissa; the same check in double precision
/// agrees to 4e-9, which is that check's own floor.
#[test]
fn the_antiderivative_differentiates_back_to_the_distortion() {
    let mut worst = 0.0f32;
    for k in [0.0f32, 0.1, 0.3, 0.6, MAX_DISTORTION_K] {
        for v0 in [0.25f32, 1.0] {
            let mut v = -8.0f32;
            while v < 8.0 {
                // A wide step on purpose: narrower loses more to
                // cancellation in `f32` than it gains in truncation.
                let h = 0.01;
                let num = (distortion_antiderivative(v + h, 0.0, k, v0)
                    - distortion_antiderivative(v - h, 0.0, k, v0))
                    / (2.0 * h);
                worst = worst.max((num - distortion(v, 0.0, k, v0)).abs());
                v += 0.01;
            }
        }
    }
    assert!(
        worst < 1e-3,
        "the antiderivative does not differentiate back: worst error {worst:e}"
    );
    // An antiderivative is defined only up to a constant, and this one is
    // not zero at zero input, so a caller must difference it rather than
    // read it absolutely. That is what antiderivative anti-aliasing does,
    // and the difference over an interval must match the integral of the
    // curve across it.
    let (a, b, k, v0) = (0.1f32, 0.9f32, 0.6f32, 0.25f32);
    let diff = distortion_antiderivative(b, 0.0, k, v0) - distortion_antiderivative(a, 0.0, k, v0);
    let n = 200_000;
    let step = (b - a) / n as f32;
    let mut sum = 0.0f32;
    for i in 0..n {
        let v = a + (i as f32 + 0.5) * step;
        sum += distortion(v, 0.0, k, v0) * step;
    }
    assert!(
        (diff - sum).abs() < 1e-4,
        "differencing gave {diff:.6} where the integral is {sum:.6}"
    );
    assert!(distortion_antiderivative(0.0, 0.0, k, v0).is_finite());
}

/// Only the oldest cell carries the T4A's third photocell.
///
/// *Figure asserted:* which variants have it. The T4A and very early T4Bs
/// carried a fast CL-705 in parallel with the main pair; later T4Bs, which
/// is every reissue and the late-1960s silver units, dropped it.
/// *Source:* `research/LA-2A.md` section 3 in the compressor lab.
///
/// This is a bit-for-bit guard as much as a fact: adding a parallel
/// population to a shared cell is exactly the change that moves a default
/// sound by accident, and two compressors draw on this cell.
#[test]
fn only_the_oldest_cell_has_the_third_photocell() {
    for v in [T4Variant::Silver, T4Variant::Gray, T4Variant::La2] {
        let p = cell_params_for(v);
        if v == T4Variant::La2 {
            assert!(p.fast_share > 0.0, "the LA-2 cell lost its third photocell");
            assert!(p.fast_speed > 1.0, "the third photocell is not faster");
        } else {
            assert_eq!(
                p.fast_share, 0.0,
                "cell {v:?} gained a third photocell it never had"
            );
        }
    }
    // The other two report their free carriers untouched, through attack
    // and through release.
    for cell in [T4Variant::Silver, T4Variant::Gray] {
        let mut c = Cell::new(cell_params_for(cell), SR);
        for _ in 0..2000 {
            c.step(0.4);
        }
        assert_eq!(
            c.conductance(),
            c.n_f,
            "cell {cell:?}'s conductance is no longer exactly its free carriers"
        );
        for _ in 0..2000 {
            c.step(0.0);
        }
        assert_eq!(c.conductance(), c.n_f, "cell {cell:?} drifted on release");
    }
}

/// The oldest cell recovers in two stages: quicker at first, then slower.
///
/// *Figure asserted:* the **shape**, not a magnitude. The T4A's third
/// photocell gives "a dual time constant that broadcast engineers liked",
/// and the same source concludes the response "is dominated by the
/// response of the slower photocell". *Source:* `research/LA-2A.md`
/// section 3 in the compressor lab.
///
/// Being ahead early and behind later is the assertion that matters,
/// because no single speed multiplier can be both, so a test measuring
/// only total speed would have passed with a scalar and missed the
/// feature entirely.
///
/// **The window is 20 ms to 200 ms, and the reason is not the one I first
/// gave.** Measured in decibels through a nominal divider, the crossover
/// is unmistakable there: recovered fractions of 0.093, 0.304 and 0.525
/// for the oldest cell against 0.078, 0.384 and 0.681 for the reference,
/// so it leads at 20 ms and trails by 16 points at 200 ms.
///
/// In the tail it leads again: 0.890, 0.895 and 0.901 at 1, 3 and 5
/// seconds against the reference's 0.868, 0.878 and 0.892. That is not
/// noise, and an earlier version of this comment dismissed it as such on
/// the strength of a *linear* conductance measure that read 98.7 %
/// recovered while 4.8 dB of 35.2 was still being held. The real reason is
/// physical: once the fast photocell has fully recovered, its 22 % share
/// of the parallel conductance is back in full, which pulls the oldest
/// cell's apparent recovery ahead late even though its own time constants
/// are slower.
///
/// So the claim that the slow photocell dominates the tail is not
/// assertable on the bare cell, and it is not asserted here. The
/// compressor lab asserts it through a whole LA-2A, where the divider and
/// a loaded trap population change the balance.
#[test]
fn the_oldest_cell_has_a_dual_time_constant() {
    let at = [0.02f32, 0.1, 0.2];
    let silver = recovery(cell_params_for(T4Variant::Silver), DRIVE, &at);
    let gray = recovery(cell_params_for(T4Variant::Gray), DRIVE, &at);
    let la2 = recovery(cell_params_for(T4Variant::La2), DRIVE, &at);

    assert!(
        la2[0] > gray[0],
        "at 20 ms the oldest cell had recovered {:.3} against the reference's {:.3};          its third photocell should make the first part quicker",
        la2[0],
        gray[0]
    );
    assert!(
        la2[1] < gray[1],
        "at 100 ms it had recovered {:.3} against {:.3}; once the fast cell has          done its work the slower one must take over",
        la2[1],
        gray[1]
    );
    assert!(
        la2[2] < gray[2],
        "at 200 ms it had recovered {:.3} against {:.3}; the slow photocell dominates",
        la2[2],
        gray[2]
    );
    assert!(
        la2[0] < silver[0],
        "the oldest cell's early recovery ({:.3}) overtook the fastest cell's ({:.3});          the third photocell is meant to be secondary",
        la2[0],
        silver[0]
    );
    assert!(
        silver[0] > gray[0],
        "the fastest cell should lead the reference early: {:.3} against {:.3}",
        silver[0],
        gray[0]
    );
}

/// The three variants order by speed as the manufacturer describes.
///
/// *Figure asserted:* the **ordering only**, Silver fastest and LA-2
/// slowest, which is a manufacturer's qualitative description and the only
/// statement of ordering in the research. The magnitudes are estimates:
/// see [`CELL_SPEEDS`]. The span is pinned loosely so that widening the
/// multipliers to make a control feel more useful fails this test rather
/// than passing quietly.
#[test]
// The assertions are on constants, and that is exactly the point: this
// test exists to fail the moment somebody edits those constants, which is
// the only way they can change.
#[allow(clippy::assertions_on_constants)]
fn the_variants_order_by_speed_and_the_span_stays_an_estimate() {
    assert!(
        CELL_SPEEDS[0] < CELL_SPEEDS[1] && CELL_SPEEDS[1] < CELL_SPEEDS[2],
        "the variants are no longer ordered: {CELL_SPEEDS:?}"
    );
    assert_eq!(
        CELL_SPEEDS[1], 1.0,
        "the reference cell must stay exactly 1.0 so the default sound does not move"
    );
    let span = CELL_SPEEDS[2] / CELL_SPEEDS[0];
    assert!(
        (1.4..=2.6).contains(&span),
        "the span is {span:.2}. It is an estimate from a manufacturer's description, \
         and the one real measurement of six units reports no consistent \
         vintage-versus-reissue grouping, so its wider spread is unit-to-unit \
         variation and must not be borrowed to widen this."
    );
}

/// Nothing goes non-finite, and the states park at exactly zero.
///
/// The compressor lab found an envelope follower stuck on a subnormal for
/// ever after eleven seconds of silence, so the flush is asserted rather
/// than assumed.
#[test]
fn numerical_hygiene() {
    for cell in [T4Variant::Silver, T4Variant::Gray, T4Variant::La2] {
        let mut c = Cell::new(cell_params_for(cell), SR);
        for v in [0.0f32, 1.0, -1.0, 1e6, -1e6, 1e-30] {
            for _ in 0..1000 {
                c.step(v);
            }
            assert!(c.u.is_finite() && c.n_f.is_finite() && c.n_t.is_finite());
            assert!(c.n_fast.is_finite() && c.light.is_finite());
            assert!((0.0..=1.0).contains(&c.n_f), "n_f left 0..1: {}", c.n_f);
            assert!((0.0..=1.0).contains(&c.n_t), "n_t left 0..1: {}", c.n_t);
        }
        c.reset();
        assert_eq!((c.u, c.n_f, c.n_t, c.n_fast), (0.0, 0.0, 0.0, 0.0));

        // Silence parks every state at exactly zero rather than leaving a
        // subnormal behind.
        for _ in 0..1000 {
            c.step(0.5);
        }
        for _ in 0..(SR as usize * 30) {
            c.step(0.0);
        }
        assert_eq!(c.u, 0.0, "the panel drive parked on {}", c.u);
        assert_eq!(c.n_f, 0.0, "free carriers parked on {}", c.n_f);
    }
}

/// The cell follows the same trajectory at any sample rate.
///
/// **The settled value alone proves nothing.** A fixed point is where the
/// derivative is zero, which is independent of the step size, so comparing
/// settled conductance across rates passes for any integration scheme
/// including a badly broken one. This walks the attack instead and
/// compares it partway, which is where a step-size error actually shows.
#[test]
fn behaviour_is_sample_rate_independent() {
    let at = |sr: f32, t: f32| {
        let mut c = Cell::new(CellParams::GRAY, sr);
        for _ in 0..((sr * t) as usize) {
            c.step(DRIVE);
        }
        c.conductance()
    };
    for t in [0.02f32, 0.06, 0.2] {
        let a = at(44_100.0, t);
        let b = at(48_000.0, t);
        let d = at(96_000.0, t);
        assert!(
            (a - b).abs() < 0.01 && (b - d).abs() < 0.01,
            "at {t} s the attack differed with the rate: {a:.4}, {b:.4}, {d:.4}"
        );
        assert!(
            a > 0.0 && a < 1.0,
            "at {t} s the cell was not mid-attack: {a:.4}"
        );
    }
    // And the fixed point still agrees, which is necessary but not
    // sufficient and is why it is not the whole test.
    let settled = |sr: f32| at(sr, 4.0);
    assert!((settled(44_100.0) - settled(96_000.0)).abs() < 0.01);
}

/// Changing the sample rate or the parameters keeps the cell coherent.
#[test]
fn retuning_keeps_the_panel_filter_correct() {
    let mut c = Cell::new(CellParams::GRAY, SR);
    for _ in 0..1000 {
        c.step(0.5);
    }
    c.set_sample_rate(96_000.0);
    c.set_params(cell_params_for(T4Variant::La2));
    for _ in 0..1000 {
        c.step(0.5);
    }
    assert!(c.conductance().is_finite() && c.conductance() > 0.0);
    // A parameter set with a different panel smoothing must retune.
    let mut d = Cell::new(CellParams::GRAY, SR);
    let faster = CellParams {
        tau_u: CellParams::GRAY.tau_u / 4.0,
        ..CellParams::GRAY
    };
    d.set_params(faster);
    for _ in 0..200 {
        d.step(1.0);
    }
    let mut slow = Cell::new(CellParams::GRAY, SR);
    for _ in 0..200 {
        slow.step(1.0);
    }
    assert!(
        d.u > slow.u,
        "a shorter panel smoothing should follow the drive sooner: {} against {}",
        d.u,
        slow.u
    );
}

/// The photoconductor's distortion grows with how hard the cell works, is
/// odd-order, and vanishes when the cell is dark.
///
/// *Figure asserted:* the **shape and the scaling**, which is what the
/// physics gives: "a photoresistor distorts in proportion to the voltage
/// across it, which is why it is scaled by the reduction". No depth is
/// asserted here, because `k` and `v0` belong to the caller: the three
/// optical compressors that use this anchor them to their own published
/// distortion figures and they differ by a factor of six.
#[test]
fn the_photoconductor_distorts_in_proportion_to_its_work() {
    const K: f32 = 0.6;
    const V0: f32 = 0.25;

    // A dark cell passes the signal untouched, whatever the amplitude.
    for v in [0.0f32, 0.1, 0.5, 1.0, -0.7] {
        assert_eq!(
            distortion(v, 1.0, K, V0),
            v,
            "a cell doing no work still distorted at v = {v}"
        );
    }

    // Deeper reduction distorts more.
    let light = (1.0 - distortion(0.5, 0.9, K, V0) / 0.5).abs();
    let hard = (1.0 - distortion(0.5, 0.2, K, V0) / 0.5).abs();
    assert!(
        hard > light,
        "a harder-working cell should distort more: {hard:.5} against {light:.5}"
    );

    // Odd order: the law is a function of the square, so it acts on the
    // magnitude and keeps the sign, which is what makes it odd-order.
    for v in [0.05f32, 0.2, 0.6, 1.0] {
        let pos = distortion(v, 0.4, K, V0);
        let neg = distortion(-v, 0.4, K, V0);
        assert!(
            (pos + neg).abs() < 1e-6,
            "the term is not odd at v = {v}: {pos} against {neg}"
        );
        assert!(
            pos.abs() <= v.abs() + 1e-6,
            "it should compress, not expand"
        );
    }

    // Small signals are barely touched; the term is a soft saturation
    // rather than a gate.
    let tiny = distortion(1e-4, 0.2, K, V0);
    assert!(
        (tiny - 1e-4).abs() / 1e-4 < 0.01,
        "small signals moved too much"
    );

    // Zero strength is an exact bypass, so a caller can turn it off.
    for v in [0.3f32, -0.8] {
        assert_eq!(distortion(v, 0.1, 0.0, V0), v);
    }

    // And it stays finite at extremes.
    for v in [1e6f32, -1e6, 0.0] {
        assert!(distortion(v, 0.0, K, V0).is_finite());
    }
}
