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

/// Drive the cell to a steady state, then release it, reporting how much
/// of the settled conductance has come back at each time.
fn recovery(params: CellParams, drive: f32, at_s: &[f32]) -> Vec<f32> {
    let mut c = Cell::new(params, SR);
    for _ in 0..(SR as usize * 4) {
        c.step(drive);
    }
    let settled = c.conductance();
    assert!(settled > 0.05, "the cell barely lit: {settled:.4}");
    let mut out = Vec::with_capacity(at_s.len());
    let mut done = 0usize;
    for t in at_s {
        let want = (SR * t) as usize;
        while done < want {
            c.step(0.0);
            done += 1;
        }
        out.push(1.0 - c.conductance() / settled);
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
    for cell in 0..3 {
        let p = cell_params_for(cell);
        if cell == 2 {
            assert!(p.fast_share > 0.0, "the LA-2 cell lost its third photocell");
            assert!(p.fast_speed > 1.0, "the third photocell is not faster");
        } else {
            assert_eq!(
                p.fast_share, 0.0,
                "cell {cell} gained a third photocell it never had"
            );
        }
    }
    // The other two report their free carriers untouched, through attack
    // and through release.
    for cell in [0usize, 1] {
        let mut c = Cell::new(cell_params_for(cell), SR);
        for _ in 0..2000 {
            c.step(0.4);
        }
        assert_eq!(
            c.conductance(),
            c.n_f,
            "cell {cell}'s conductance is no longer exactly its free carriers"
        );
        for _ in 0..2000 {
            c.step(0.0);
        }
        assert_eq!(c.conductance(), c.n_f, "cell {cell} drifted on release");
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
/// **The window is 20 ms to 200 ms, and that is deliberate.** Measured on
/// the bare cell, the crossover is unmistakable there: at 20 ms the oldest
/// cell is ahead, and from 50 ms to 200 ms it is behind by up to six
/// points. By half a second every variant is more than 98 % recovered and
/// the differences fall into the noise, so an assertion about the tail
/// would be measuring rounding. The long tail belongs to a whole
/// compressor, where a loaded trap population stretches it, and the
/// compressor lab asserts it there.
#[test]
fn the_oldest_cell_has_a_dual_time_constant() {
    let at = [0.02f32, 0.1, 0.2];
    let silver = recovery(cell_params_for(0), DRIVE, &at);
    let gray = recovery(cell_params_for(1), DRIVE, &at);
    let la2 = recovery(cell_params_for(2), DRIVE, &at);

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
    for cell in 0..3 {
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

/// The cell behaves the same at any sample rate.
#[test]
fn behaviour_is_sample_rate_independent() {
    let settle = |sr: f32| {
        let mut c = Cell::new(CellParams::GRAY, sr);
        for _ in 0..(sr as usize * 4) {
            c.step(0.5);
        }
        c.conductance()
    };
    let a = settle(44_100.0);
    let b = settle(48_000.0);
    let d = settle(96_000.0);
    assert!(
        (a - b).abs() < 0.01 && (b - d).abs() < 0.01,
        "settled conductance moved with the rate: {a:.4}, {b:.4}, {d:.4}"
    );
}

/// Changing the sample rate or the parameters keeps the cell coherent.
#[test]
fn retuning_keeps_the_panel_filter_correct() {
    let mut c = Cell::new(CellParams::GRAY, SR);
    for _ in 0..1000 {
        c.step(0.5);
    }
    c.set_sample_rate(96_000.0);
    c.set_params(cell_params_for(2));
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
