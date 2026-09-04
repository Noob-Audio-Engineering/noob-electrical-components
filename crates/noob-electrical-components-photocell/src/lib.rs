//! The photoconductive element at the heart of an optical compressor, and
//! the T4-family cell built around it.
//!
//! Two things live here, and the difference matters. [`distortion`] and
//! the resistance laws are properties of a **photoresistor**, true of any
//! of them; the Tube-Tech CL-1B, whose potted element is emphatically not
//! a T4 and which refuses this crate's timing entirely, still obeys them.
//! [`Cell`] and its time constants are the **T4** specifically, an
//! electroluminescent panel glued to a cadmium-sulphide photoresistor, as
//! used in the LA-2A and LA-3A.
//!
//! Taking the T4 cell first: its
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
//! This crate is the element and the cell alone. The resistive divider it shunts, the
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
#![warn(missing_docs)]

/// Flush a state to zero once it has decayed far enough that arithmetic on
/// it is a cost rather than a signal.
///
/// The cell's states decay exponentially towards zero and can sit in the
/// subnormal range for a long time after a signal stops, where arithmetic
/// is slow on some hardware. It is not theoretical: an equaliser built on
/// the same foundations had an envelope follower parked on a subnormal
/// permanently after eleven seconds of silence.
///
/// The threshold is 1e-12, matching the plug-ins that use this crate. It
/// is the smaller of the two values that were in circulation, and smaller
/// is the conservative direction for a guard: subnormals begin near 1e-38,
/// so 1e-12 prevents the stall by twenty-six orders of magnitude while
/// touching a thousand times fewer real values than 1e-9 would. It is
/// around 240 dB below full scale, which is the honest way to put it: a
/// state is not a signal and has no loudness of its own. That
/// matters here in particular, because the trapped-carrier state is the
/// mechanism behind an optical compressor's memory and decays over
/// seconds, so clamping it early is exactly the wrong economy.
#[inline]
fn flush(x: f32) -> f32 {
    if x.abs() < 1e-12 { 0.0 } else { x }
}

/// Photocell resistance in the dark (ohms).
pub const R_DARK: f32 = 2.0e6;
/// Photocell resistance under full light (ohms).
///
/// With [`R_DARK`] this is a resistance ratio of 4000, which is 72.0 dB.
/// That is the part's own range and it is not what any compressor gets:
/// the cell is a shunt in a divider, and how much of 72 dB reaches the
/// audio depends on the resistances around it. Through the LA-2A's, a
/// 70.7 kΩ series resistor and the 100 kΩ Gain pot in parallel with the
/// cell, it works out at 38.3 dB. Both figures are asserted in the tests,
/// because the smaller one used to be stated here as the cell's own range
/// without a derivation and it is not.
pub const R_MIN: f32 = 500.0;

/// Electroluminescent light law exponent (`L = exp(−b / √(u / V_ref))`).
pub const EL_B: f32 = 5.0;

/// Photocell gamma (conductance ∝ light^γ).
pub const CELL_GAMMA: f32 = 0.8;
/// Photocell conductance for full light, so `n_f = 1` gives `R_MIN`.
pub const K_G: f32 = 1.0 / R_MIN - 1.0 / R_DARK;

/// Time constants of the photocell, in seconds, for one cell variant.
///
/// # Build one from a constant, not from scratch
///
/// Start from [`CellParams::GRAY`] and change what differs:
///
/// ```
/// # use noob_electrical_components_photocell::CellParams;
/// let hotter = CellParams { k_gen: 12.0, ..CellParams::GRAY };
/// ```
///
/// That is the supported way, and it is what makes adding a field here a
/// non-breaking change: a functional update fills anything new from the
/// base. Fields have been added once already, when the oldest cell's third
/// photocell arrived, and every caller using this pattern was unaffected.
///
/// This struct is deliberately **not** `#[non_exhaustive]`. That attribute
/// exists to give the same guarantee, but it forbids functional update
/// syntax from another crate, so it removes the mechanism that already
/// provides the guarantee and charges a setter per field for the
/// privilege. It was tried here and taken back off; please do not reach
/// for it again.
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
/// How much faster that third photocell is than the main pair, as a
/// divisor on its time constants. **Estimate**; see
/// [`CellParams::fast_speed`].
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
pub fn cell_params_for(cell: T4Variant) -> CellParams {
    let base = CellParams::GRAY.scaled(CELL_SPEEDS[cell as usize]);
    if cell == T4Variant::La2 {
        base.with_fast_cell(LA2_FAST_SHARE, LA2_FAST_SPEED)
    } else {
        base
    }
}

/// Which T4 a unit is fitted with.
///
/// This used to be a `usize` clamped with `min(2)`, so a wrong index gave
/// the wrong cell silently instead of failing. The discriminants match the
/// old indices and [`CELL_SPEEDS`], so a stored parameter still maps the
/// same way, but the conversion is now [`T4Variant::from_index`] and
/// visible at the one boundary where a raw index genuinely arrives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum T4Variant {
    /// A late-1960s T4B, the fastest of the three.
    Silver = 0,
    /// The reference cell, and the default.
    #[default]
    Gray = 1,
    /// The T4A of the LA-2 and early LA-2A: the slowest, and the only one
    /// with the third, faster photocell.
    La2 = 2,
}

impl T4Variant {
    /// The variant a stored parameter index names.
    ///
    /// Anything past the last variant saturates at it, which is what the
    /// old `min(2)` did. That is a deliberate choice for a host handing
    /// back a value from a newer version of a plug-in, and it is stated
    /// here rather than buried in an expression.
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Silver,
            1 => Self::Gray,
            _ => Self::La2,
        }
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
    /// A cell at rest: dark, no carriers, no traps filled.
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

    /// Retune for a new sample rate, keeping the current state.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.dt = 1.0 / sr;
        self.a_u = 1.0 - (-self.dt / self.params.tau_u.max(1e-6)).exp();
    }

    /// Swap the cell's parameters, retuning the panel's smoothing only if
    /// its time constant actually changed.
    pub fn set_params(&mut self, params: CellParams) {
        let retune = params.tau_u != self.params.tau_u;
        self.params = params;
        if retune {
            let sr = 1.0 / self.dt;
            self.set_sample_rate(sr);
        }
    }

    /// Return to darkness: every state to exactly zero.
    pub fn reset(&mut self) {
        self.u = 0.0;
        self.light = 0.0;
        self.n_f = 0.0;
        self.n_fast = 0.0;
        self.n_t = 0.0;
    }

    /// The Alfrey-Taylor electroluminescent law: zero slope near zero (a
    /// soft threshold) and saturating at high drive.
    ///
    /// # A second simplification, also named
    ///
    /// Alfrey-Taylor is an **alternating-current** law, and a real panel's
    /// brightness rises with the drive's frequency as well as its
    /// amplitude. This takes a rectified, smoothed envelope, which throws
    /// that away. In an LA-2A the panel sees the sidechain audio itself, so
    /// the same level of high-frequency-dense programme lights it brighter,
    /// and this structure cannot produce that.
    ///
    /// Note for anyone reading a compressor built on this: a model may
    /// still react more to highs, through a sidechain emphasis filter ahead
    /// of the cell. That is a different mechanism with a different
    /// signature, and it is not this one.
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
    ///
    /// # A fixed exponent is a simplification, and it is named here
    ///
    /// Conductance goes as `light^γ` with γ fixed at [`CELL_GAMMA`]. A real
    /// cadmium-sulphide cell does not have one exponent: the datasheet
    /// figure is "a straight line on log-log paper" defined only **between
    /// 10 and 100 lux**, published values across common cells span 0.6 to
    /// 0.9, and the physical models in the literature use a *dual-slope*
    /// power law, a sum of two terms with different exponents, rather than
    /// one. So the single exponent is a simplification of a curve, and it
    /// is what puts the hard corner documented on [`Cell::step`] where a
    /// real cell has a gradual one.
    ///
    /// It stays fixed because no source gives the two endpoints a
    /// light-dependent exponent would need. Inventing them would move the
    /// behaviour of two shipped compressors on numbers nobody published,
    /// which is worse than a documented simplification.
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
    ///
    /// # Usable drive range
    ///
    /// The model is faithful up to roughly **4.2 V** of smoothed drive and
    /// saturates hard above it. [`carriers_for`](Self::carriers_for)
    /// returns `k_gen · light^γ`, which reaches 1.0 at a light of 0.0878,
    /// and the free-carrier state is clamped to 0..1. So past that point
    /// generation is pinned at the clamp, and neither the panel's law nor
    /// the photoconductor's contributes anything further: 5 V and 50 V of
    /// drive give bit-identical output. The attack also changes character
    /// across that line, from about 39 ms at 1 V to 8.3 ms at 6 V, so the
    /// roughly 10 ms the specifications quote is only produced above it.
    ///
    /// This is a real limit of the model rather than of the part, and a
    /// caller should keep its sidechain inside it. It is recorded rather
    /// than removed because removing it means choosing a light-dependent
    /// exponent, and no source gives one; see the note on the fixed
    /// exponent in [`carriers_for`](Self::carriers_for).
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
    /// # Do not judge a recovery by this number
    ///
    /// The cell is a shunt in a divider and a listener hears decibels, so
    /// linear conductance badly overstates how far a recovery has got. At
    /// half a second after a hard hit this reads 98.7 % recovered while
    /// 4.8 dB of the original 35.2 is still being held, and at three
    /// seconds it reads 98.9 % with 4.3 dB still held. Anyone comparing
    /// cells, or deciding that a tail has finished, must convert through
    /// the divider first.
    ///
    /// This is not hypothetical. Two people in succession concluded from
    /// the linear figure that the differences between cells past half a
    /// second were rounding, and both were wrong. Converted to decibels the
    /// differences are real, and the ordering actually **reverses**: the
    /// oldest cell leads late rather than trailing, because once its third
    /// photocell has recovered it returns its whole share of the parallel
    /// conductance. So the claim that the slow photocell dominates the tail
    /// is *not* assertable on a bare cell, and this crate does not assert
    /// it. A compressor can, where its own divider and a loaded trap
    /// population change the balance, and the LA-2A does.
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

/// Largest `k · (1 - attenuation)` for which [`distortion`] stays
/// monotonic.
///
/// The term is `v - kc·v³/(v0² + v²)`, whose derivative in `q = v/v0` is
/// `1 - kc·q²(3 + q²)/(1 + q²)²`. That numerator loses positivity exactly
/// at `kc = 8/9`: measured, the minimum derivative is `+0.000000` at 8/9,
/// `-0.000125` at 0.889 and `-0.125` at 1.0, where the curve folds back on
/// itself. Above `kc = 1` the output changes sign at large input, which is
/// not distortion but inversion.
pub const MAX_DISTORTION_K: f32 = 8.0 / 9.0;

/// The photoconductor's own distortion: an odd-order term that grows with
/// how hard the cell is working.
///
/// A photoresistor is not a linear resistor. The voltage across it moves
/// its operating point within a sample, so it distorts in proportion to
/// that voltage, which is why the term is scaled by how much the cell is
/// attenuating: a dark cell passing the signal barely distorts, and a lit
/// one working hard does.
///
/// `v` is the already-attenuated sample, `attenuation` the cell's gain
/// (1.0 dark, falling as it lights), and `k` and `v0` the strength and the
/// reference amplitude. Those two stay with the caller, because each unit
/// anchors them to its own published distortion figure and they differ by
/// a factor of six across the three optical compressors this serves. The
/// law is the part's; the depth is the machine's.
///
/// Where the caller taps its detector relative to this is also the
/// caller's: two of those compressors distort the audio node and then take
/// the sidechain from it, so their detectors hear the distortion, while a
/// third takes its detector from a different node entirely.
///
/// # Constraint
///
/// `k · (1 - attenuation)` must not exceed [`MAX_DISTORTION_K`], or the
/// curve folds back on itself and stops being a distortion at all. Since
/// attenuation reaches zero when a cell works hardest, the effective
/// figure is `k` itself, so **`k` must be at most 8/9**. Debug builds
/// assert it. The three optical compressors using this pass 0.6, 0.2 and
/// 0.1, so all have margin, but the largest is within a factor of 1.5 of
/// the limit and the constraint was previously unstated.
///
/// This is not a saturator: as the input grows the term approaches a
/// constant and the output tends to a linear gain of `1 - kc`, so it bends
/// the curve rather than bounding it. A caller that needs a ceiling needs
/// one of its own.
#[inline]
pub fn distortion(v: f32, attenuation: f32, k: f32, v0: f32) -> f32 {
    let kc = k * (1.0 - attenuation);
    debug_assert!(
        kc <= MAX_DISTORTION_K,
        "distortion k·(1-a) = {kc} exceeds {MAX_DISTORTION_K}, where the curve folds"
    );
    let q = v / v0;
    let q2 = q * q;
    v * (1.0 - kc * q2 / (1.0 + q2))
}

/// The antiderivative of [`distortion`] with respect to `v`.
///
/// Antiderivative anti-aliasing needs the integral of the nonlinearity it
/// is smoothing, and a caller cannot write one without duplicating the law
/// above, which is exactly the drift this crate exists to prevent. The
/// machinery that uses this belongs to whoever is doing the processing;
/// the integral of the part's own curve belongs to the part.
///
/// Since `distortion` is `v - kc·v³/(v0² + v²)`, integrating term by term
/// gives `v²/2 - (kc/2)·(v² - v0²·ln(v0² + v²))`. Checked against
/// numerical differentiation of this function across `kc` from 0 to 8/9,
/// two values of `v0` and `v` from −8 to 8: the largest disagreement is
/// 4e-9, which is the finite-difference floor rather than the formula.
#[inline]
pub fn distortion_antiderivative(v: f32, attenuation: f32, k: f32, v0: f32) -> f32 {
    let kc = k * (1.0 - attenuation);
    let v2 = v * v;
    let v02 = v0 * v0;
    0.5 * v2 - 0.5 * kc * (v2 - v02 * (v02 + v2).ln())
}
