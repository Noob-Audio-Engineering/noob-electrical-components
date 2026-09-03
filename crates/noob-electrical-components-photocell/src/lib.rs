//! A photoconductive cell driven by an electroluminescent panel: the
//! light-dependent resistor at the heart of an optical compressor.
//!
//! This is the T4-family cell, an electroluminescent panel glued to a
//! cadmium-sulphide photoresistor, as used in the LA-2A and LA-3A. Its
//! behaviour is not a set of time constants somebody chose. The panel's
//! light follows the Alfrey-Taylor law, the photoresistor's conductance
//! follows a power law in that light, and the carriers fall into traps and
//! climb out again. The 10 ms attack, the 60 ms first release stage, the
//! half-to-five-second second stage and the programme dependence all fall
//! out of that, which is why a compressor built on it behaves the way the
//! hardware does without anybody dialling in a curve.
//!
//! # What is a component and what is a circuit
//!
//! This crate is the cell alone. The resistive divider it shunts, the
//! sidechain that drives it and the make-up gain after it belong to the
//! machine that uses it, not here, because those differ from unit to unit
//! while the cell does not. The dividing line is worth stating because it
//! has already been tested by real use: the LA-2A and the LA-3A share this
//! cell exactly, differing only in how hard they light it and in the
//! resistances around it, while the Tube-Tech CL-1B deliberately does not
//! use it at all, because its timing lives in an op-amp sidechain rather
//! than in the cell's own carriers.
//!
//! # Variants
//!
//! [`cell_params_for`] carries the three T4 positions an LA-2A offers.
//! What is documented about them, and what is not, is set out at
//! [`CELL_SPEEDS`] and [`CellParams::fast_share`]; the short version is
//! that the ordering is a manufacturer's description, the magnitudes are
//! estimates, and the one difference with a physical basis is the third
//! photocell the earliest cells carried.

#![forbid(unsafe_code)]

/// Flush a denormal to zero.
///
/// The cell's states decay exponentially towards zero and can sit in the
/// subnormal range for a long time after a signal stops, where arithmetic
/// is slow on some hardware. The compressor lab found one of its envelope
/// followers parked on a subnormal permanently after eleven seconds of
/// silence, so this is not theoretical.
#[inline]
fn flush(x: f32) -> f32 {
    if x.abs() < 1e-12 { 0.0 } else { x }
}

/// Photocell resistance in the dark (ohms).
pub const R_DARK: f32 = 2.0e6;
/// Photocell resistance under full light (ohms); with `R_DARK` this gives
/// about 38 dB of range.
pub const R_MIN: f32 = 500.0;

/// Electroluminescent light law exponent (`L = exp(−b / √(u / V_ref))`).
pub const EL_B: f32 = 5.0;

/// Photocell gamma (conductance ∝ light^γ).
pub const CELL_GAMMA: f32 = 0.8;
/// Photocell conductance for full light, so `n_f = 1` gives `R_MIN`.
pub const K_G: f32 = 1.0 / R_MIN - 1.0 / R_DARK;

/// Time constants of the photocell, in seconds, for one cell variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellParams {
    /// Open-loop attack time constant in dim light. The loop closes faster
    /// than this (about 10 to 15 ms for a moderate hit), which is what the
    /// specifications quote.
    pub tau_f0: f32,
    /// Light (normalised 0..1) at which the attack has become twice as fast.
    pub l_a: f32,
    /// First-stage release (free carriers recombining).
    pub tau_r1: f32,
    /// Slow release with empty traps.
    pub tau_t0: f32,
    /// How much full traps slow the slow release (`tau_t = tau_t0 · (1 + k_m · n_t)`).
    pub k_m: f32,
    /// Trap capture rate, per second.
    pub capture: f32,
    /// Carrier generation at full light.
    pub k_gen: f32,
    /// Smoothing of the panel drive, seconds: the phosphor plus whatever
    /// the driver's output impedance does to it. 1 ms for the LA-2A, whose
    /// panel hangs off a pentode plate through 10 kΩ; a quarter of that for
    /// the LA-3A, which drives the same panel from a transistor stage
    /// through a step-up transformer (`research/LA-3A.md` 7.5).
    pub tau_u: f32,
    /// Share of the cell's conductance carried by a **second, faster
    /// photocell wired in parallel** with the main one. `0.0` for every
    /// cell that has only the main pair, which is every T4B from about
    /// 1969 onward and so both the Gray and Silver positions.
    ///
    /// The T4A in the LA-2 and early LA-2A, and very early T4Bs, carried
    /// three photocells: the main Clairex CL-505L pair plus a fast
    /// CL-705 across the audio cell, "giving a dual time constant that
    /// broadcast engineers liked" (`research/LA-2A.md` section 3). That
    /// is the one difference between the eras with a physical basis, and
    /// a speed multiplier cannot express it, because the point is a
    /// *shape*: a quick partial recovery followed by the slow one, not a
    /// uniformly quicker cell.
    ///
    /// **The share is an estimate.** The sources establish that the cell
    /// exists, that it is faster, and that it sits in parallel, but not
    /// how much of the conductance it carries. Kantor, who examined the
    /// modules, concluded the overall response "is dominated by the
    /// response of the slower photocell", so this is deliberately a
    /// secondary contribution: enough to hear on a transient, not enough
    /// to make the LA-2 position a different compressor.
    pub fast_share: f32,
    /// How much faster that second cell is than the main one, as a
    /// divisor on its time constants. **Estimate**: the research calls
    /// the CL-705 "fast" without giving a figure, and Clairex type-5
    /// material spans 5 ms to 120 ms of decay depending on light, so a
    /// single figure inside that span is the best that can be justified.
    pub fast_speed: f32,
}

impl CellParams {
    /// The reference ("Gray") cell.
    pub const GRAY: CellParams = CellParams {
        tau_f0: 0.040,
        l_a: 0.05,
        tau_r1: 0.060,
        tau_t0: 0.5,
        k_m: 12.0,
        capture: 1.0 / 0.3,
        k_gen: 7.0,
        tau_u: 0.001,
        fast_share: 0.0,
        fast_speed: 1.0,
    };

    /// Scale every time constant (the "cell" parameter: Silver 0.7, Gray
    /// 1.0, LA-2 1.6, estimates after the research).
    pub fn scaled(self, k: f32) -> CellParams {
        CellParams {
            tau_f0: self.tau_f0 * k,
            tau_r1: self.tau_r1 * k,
            tau_t0: self.tau_t0 * k,
            capture: self.capture / k,
            ..self
        }
    }

    /// Add the T4A's third photocell: a faster population carrying
    /// `share` of the conductance in parallel with the main one.
    pub fn with_fast_cell(self, share: f32, speed: f32) -> CellParams {
        CellParams {
            fast_share: share,
            fast_speed: speed,
            ..self
        }
    }
}

/// The T4A's fast parallel photocell, for the LA-2 position only.
///
/// **Both numbers are estimates**, for the reasons at
/// [`CellParams::fast_share`]. The share keeps the slower cell dominant,
/// as the one source that examined the modules says it is; the speed sits
/// inside the range Clairex quote for the material.
pub const LA2_FAST_SHARE: f32 = 0.22;
pub const LA2_FAST_SPEED: f32 = 8.0;

/// Cell parameters for a variant index: the speed multiplier, plus the
/// T4A's third photocell for the LA-2 position only.
///
/// **Only the LA-2 position gets the fast cell, and that is deliberate.**
/// The third photocell was fitted to the T4A and to very early T4Bs, and
/// dropped from about 1969; the Silver position is a late-1960s T4B and
/// every reissue is one, so neither has it. The LA-3A, which shares this
/// cell, is a 1969-onward unit whose own control is about a cell's *age*
/// rather than its era, so it does not get one either and calls the
/// parameters it already builds.
pub fn cell_params_for(cell: usize) -> CellParams {
    let base = CellParams::GRAY.scaled(CELL_SPEEDS[cell.min(2)]);
    if cell.min(2) == 2 {
        base.with_fast_cell(LA2_FAST_SHARE, LA2_FAST_SPEED)
    } else {
        base
    }
}

/// Speed multipliers for the three cell variants: Silver, Gray, LA-2.
///
/// **These are an estimate, and what follows is the whole of what the
/// research establishes**, because a variant switch calibrated against
/// nothing is how a control drifts away from the machine it names.
///
/// *The one physical, era-specific fact.* The T4A, fitted to the LA-2 and
/// early LA-2A, and very early T4Bs up to about 1969, contained **three**
/// photocells: the main Clairex CL-505L pair plus a fast CL-705 wired in
/// parallel with the audio cell, giving a dual time constant. Later T4Bs,
/// which is what the late-1960s silver units and every reissue use,
/// dropped the third cell. So the documented difference between the eras
/// is a *construction* difference, and it runs the opposite way to the
/// speed ordering: the older cell had an extra **fast** element, not a
/// slower one. Its own source qualifies that immediately, though: Kantor
/// concluded the overall response "is dominated by the response of the
/// slower photocell". (`research/LA-2A.md` section 3.)
///
/// *The ordering these multipliers follow* is Universal Audio's product
/// description of the three eras, with Silver fast, Gray the medium
/// reference and the LA-2 slowest, "mellowed" by fifty years of panel
/// ageing. That is a manufacturer's qualitative claim about ageing rather
/// than a measurement, and it is the only statement of ordering anywhere
/// in the research.
///
/// *The one real measurement does not support an era effect at all.*
/// Moore measured six units and found attack spread 33 to 81 ms and
/// release 449 to 1670 ms, wider than the 2.3 here, but reports **"no
/// consistent vintage-versus-reissue grouping"**. That spread is
/// therefore unit-to-unit variation, conflating cell age, component
/// tolerance and calibration, and borrowing it to size an era switch
/// would attribute to the three cells a variation its own source says the
/// three cells do not explain.
///
/// So: the ordering is documented, the magnitude is not, and the span
/// stays where a manufacturer's description puts it rather than being
/// widened to make the control feel more useful. Gray is exactly 1.0, so
/// the default sound is the reference one.
///
/// **What a multiplier cannot say, and what carries it instead.** A single
/// speed multiplier cannot express the T4A's dual time constant, because
/// the point is a *shape*: a quick partial recovery followed by the slow
/// one, not a uniformly quicker cell. That is why the LA-2 position also
/// gets a second, faster carrier population, through
/// [`CellParams::fast_share`], rather than a larger number here. These
/// multipliers carry only the ageing the manufacturer describes.
pub const CELL_SPEEDS: [f32; 3] = [0.7, 1.0, 1.6];

/// The T4 cell: an electroluminescent panel and a CdS photoresistor with
/// traps. Two states, free carriers `n_f` (0..1, conductance) and trapped
/// carriers `n_t` (0..1, the memory), plus the panel's smoothed drive `u`.
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    /// Smoothed rectified sidechain drive, in sidechain volts.
    pub u: f32,
    /// Light, normalised 0..1.
    pub light: f32,
    /// Free carriers (conductance), 0..1.
    pub n_f: f32,
    /// Free carriers in the T4A's second, faster photocell. Only moves
    /// when [`CellParams::fast_share`] is non-zero, which is the LA-2
    /// position alone.
    pub n_fast: f32,
    /// Trapped carriers (memory), 0..1.
    pub n_t: f32,
    params: CellParams,
    dt: f32,
    a_u: f32,
}

impl Cell {
    pub fn new(params: CellParams, sr: f32) -> Self {
        let mut c = Cell {
            u: 0.0,
            light: 0.0,
            n_f: 0.0,
            n_fast: 0.0,
            n_t: 0.0,
            params,
            dt: 1.0 / sr,
            a_u: 0.0,
        };
        c.set_sample_rate(sr);
        c
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.dt = 1.0 / sr;
        self.a_u = 1.0 - (-self.dt / self.params.tau_u.max(1e-6)).exp();
    }

    pub fn set_params(&mut self, params: CellParams) {
        let retune = params.tau_u != self.params.tau_u;
        self.params = params;
        if retune {
            let sr = 1.0 / self.dt;
            self.set_sample_rate(sr);
        }
    }

    pub fn reset(&mut self) {
        self.u = 0.0;
        self.light = 0.0;
        self.n_f = 0.0;
        self.n_fast = 0.0;
        self.n_t = 0.0;
    }

    /// The Alfrey-Taylor electroluminescent law: zero slope near zero (a
    /// soft threshold) and saturating at high drive.
    #[inline]
    pub fn light_for(u: f32) -> f32 {
        if u <= 1e-6 {
            0.0
        } else {
            (-EL_B / u.sqrt()).exp()
        }
    }

    /// Steady-state free carriers for a given light (what the cell settles
    /// to under constant illumination).
    #[inline]
    pub fn carriers_for(light: f32, params: &CellParams) -> f32 {
        if light <= 0.0 {
            0.0
        } else {
            params.k_gen * light.powf(CELL_GAMMA)
        }
    }

    /// Advance one sample with the instantaneous sidechain voltage `v`
    /// (signed; rectified here).
    #[inline]
    pub fn step(&mut self, v: f32) {
        let p = self.params;
        self.u += self.a_u * (v.abs() - self.u);
        self.u = flush(self.u);
        let light = Self::light_for(self.u);
        self.light = light;
        let generation = Self::carriers_for(light, &p);
        let tau = if generation > self.n_f {
            p.tau_f0 / (1.0 + light / p.l_a)
        } else {
            p.tau_r1
        };
        let capture = p.capture * self.n_f * (1.0 - self.n_t);
        let tau_t = p.tau_t0 * (1.0 + p.k_m * self.n_t);
        let detrap = self.n_t / tau_t;
        let n_f = self.n_f + self.dt * ((generation - self.n_f) / tau - capture + detrap);
        let n_t = self.n_t + self.dt * (capture - detrap);
        self.n_f = flush(n_f.clamp(0.0, 1.0));
        self.n_t = flush(n_t.clamp(0.0, 1.0));
        if p.fast_share > 0.0 {
            // The T4A's third cell: same light, no traps, and quicker.
            let tau_fast = tau / p.fast_speed;
            let n = self.n_fast + self.dt * (generation - self.n_fast) / tau_fast;
            self.n_fast = flush(n.clamp(0.0, 1.0));
        }
    }

    /// The conductance the divider sees, 0..1.
    ///
    /// With only the main photocell this is exactly `n_f`, returned
    /// untouched so the Gray and Silver positions are bit-for-bit what
    /// they were. With the T4A's third cell the two conductances add in
    /// parallel, weighted by the share the fast one carries.
    #[inline]
    pub fn conductance(&self) -> f32 {
        let share = self.params.fast_share;
        if share <= 0.0 {
            self.n_f
        } else {
            (1.0 - share) * self.n_f + share * self.n_fast
        }
    }

    /// Photocell resistance for the current carriers.
    #[inline]
    pub fn resistance(&self) -> f32 {
        resistance_for(self.conductance())
    }
}

/// Cell resistance for `n_f` free carriers (conductance linear in `n_f`).
#[inline]
pub fn resistance_for(n_f: f32) -> f32 {
    let g = 1.0 / R_DARK + K_G * n_f;
    (1.0 / g).clamp(R_MIN, R_DARK)
}

#[cfg(test)]
mod tests;
