//! David Blackmer's log-antilog gain cell: the part that makes a VCA
//! compressor sound like one.
//!
//! This crate is the **cell**, not "a VCA". The distinction is the whole
//! reason it exists. "VCA" is a functional category covering Blackmer's
//! log-antilog cell, the operational transconductance amplifier and a
//! field-effect transistor used as a variable resistor, which share a word
//! rather than an equation. An audit of the plug-ins that use this
//! repository found three "tube stages" that were three different circuits
//! wearing one name, and a component called `Vca` would have been the
//! fourth. What is modelled here is one part with one law.
//!
//! # The law
//!
//! From US 3,714,462, the cell is an input amplifier with two feedback
//! paths through the collector-emitter circuits of opposite-conductivity
//! transistors, so a signal current becomes a log voltage for both halves
//! of the waveform without being rectified; a second pair takes the
//! antilogarithm; and the control voltage is summed with the log signal at
//! the bases. Adding a voltage in the log domain multiplies in the linear
//! domain, so the gain is exponential in the control voltage and exactly
//! so. Blackmer's stated objective was "a constant decibels per volt
//! control characteristic" over at least a 50 dB range.
//!
//! That is why [`BlackmerCell::gain_db`] takes millivolts rather than
//! decibels. The reason this is a component and not a multiply is the
//! 6.1 mV/dB constant, its tolerance and its temperature coefficient, and
//! a caller that passes decibels has already thrown all three away.
//!
//! # What is the part and what is the machine
//!
//! This crate holds the control law, its tolerance window, the temperature
//! coefficient and the even-order residual the symmetry trim exists to
//! null. It holds nothing about the resistor that converts a voltage into
//! the cell's input current, the current-to-voltage converter after it,
//! the detector, the threshold, the ratio, or how the control voltage was
//! derived. Those are the machine, they differ completely between the two
//! units documented to contain this cell, and they belong in the plug-in.
//!
//! This is the same line the photocell crate draws: it knows its own dark
//! and lit resistances and knows nothing about the 70.7 kilohm series
//! resistor an LA-2A puts in front of it.
//!
//! # Who is documented to contain one
//!
//! Two units, on two manufacturers' own drawings, which is the standard
//! the photocell met rather than the weaker one the diode bridge met:
//!
//! - the **dbx 160**, whose schematic calls out a plug-in module lettered
//!   `VCA (200)`, reference designator M1;
//! - the **SSL 4000 G bus compressor**, whose card 82E26 has `DBX 202C`
//!   lettered on it by SSL, at both the audio and the sidechain positions.
//!
//! A third unit joins them on weaker evidence: the **API 2500**, whose
//! cell is reported to be a THAT 2180, the monolithic descendant of the
//! same design. That comes from a reviewer identifying chips in 2001
//! rather than from a manufacturer's drawing, so it corroborates the part
//! without being allowed to shape it. It needs nothing this crate does not
//! already hold, which is itself a small piece of evidence that the
//! boundary is in the right place.
//!
//! A fourth unit, the Distressor, is usually described as using a
//! Blackmer-style cell, but that is an inference from how it behaves
//! rather than a part read off a drawing, and the model of it in the lab
//! stands a single distortion constant in for the entire cell. It is free
//! to consume this crate. It did not shape it, and the asymmetry of
//! evidence is recorded here deliberately.
//!
//! The two units that do contain the cell share it and share **nothing**
//! about how they decide what to do with it: the dbx uses a true-RMS
//! detector working in the log domain, whose attack and release are one
//! locked pair, and the SSL a precision rectifier into a passive network
//! with two independent time constants. That is a satisfying result, and
//! it is the argument for this boundary. The family resemblance people
//! hear between the two really does come from the cell; the large
//! difference in how they behave on programme really does come from the
//! detector, which is why no detector is modelled here.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Nominal gain-control constant, in millivolts per decibel.
///
/// THAT publish 6.1 mV/dB typical for the 2180 series, measured over a
/// gain range of −60 dB to +40 dB, on either control port. Blackmer's
/// patent states the design objective the constant expresses but not the
/// number, which is a property of the manufactured part.
pub const K_TYP_MV_PER_DB: f32 = 6.1;

/// The tolerance window on [`K_TYP_MV_PER_DB`], in millivolts per decibel.
///
/// THAT grade the part A, B and C, and publish the control constant as
/// 6.0, 6.1 and 6.2 mV/dB in magnitude across the grades. This is a
/// manufacturing spread, not a drift: a given cell sits somewhere in the
/// window and stays there.
pub const K_RANGE_MV_PER_DB: (f32, f32) = (6.0, 6.2);

/// Temperature coefficient of the gain-control constant, per degree
/// Celsius, referenced to [`TEMPCO_REF_C`].
///
/// THAT publish +0.33 %/°C over a −60 dB to +40 dB gain range. It is not
/// an empirical fudge: a junction's thermal voltage is proportional to
/// absolute temperature, so a control law built out of junctions should
/// scale the same way, and 1/300.15 K is 0.333 %/°C. The published
/// coefficient and the physics agree to three figures, which is asserted
/// in the tests rather than merely claimed here.
///
/// The sign is worth being careful about. The constant is millivolts *per
/// decibel*, so a rise in temperature means each decibel costs more
/// millivolts, and a fixed control voltage therefore produces **less**
/// gain change. A ten degree rise moves the law by 3.3 %, which shifts
/// both the threshold and the ratio of any compressor built on the cell.
pub const TEMPCO_PER_C: f32 = 0.0033;

/// Reference chip temperature for [`TEMPCO_PER_C`], in degrees Celsius.
pub const TEMPCO_REF_C: f32 = 27.0;

/// The gain span over which the published control-law figures hold, in
/// decibels: −60 dB to +40 dB, a 100 dB window.
pub const CONTROL_SPAN_DB: (f32, f32) = (-60.0, 40.0);

/// Typical control-law linearity, as a percentage of the 100 dB span over
/// which it is specified.
///
/// THAT publish 0.5 % typical. Over [`CONTROL_SPAN_DB`] that is half a
/// decibel of bow between the real law and a straight line. It is the
/// reason a ratio law that computes an infinite slope on paper produces a
/// large finite one in hardware.
pub const LINEARITY_TYP_PCT: f32 = 0.5;

/// Maximum control-law linearity error, as a percentage of the span.
pub const LINEARITY_MAX_PCT: f32 = 2.0;

/// Guaranteed gain range, in decibels. THAT publish "> 130 dB".
pub const GAIN_RANGE_DB: f32 = 130.0;

/// The symmetry control voltage window, in millivolts, within which an
/// untrimmed A-grade cell sits at 0 dB gain.
///
/// The 2150 datasheet gives −1.6 to +1.6 mV for total harmonic distortion
/// below 0.07 %. This is the port an SSL console's `DISTORTION NULL`
/// trimmer drives and the one dbx's factory procedure adjusts for minimum
/// distortion.
pub const SYMMETRY_WINDOW_MV: f32 = 1.6;

/// Total harmonic distortion published for an untrimmed cell, per grade,
/// as a fraction (not a percentage).
///
/// THAT's table, at 1 kHz, for grades A, B and C in that order. The three
/// rows are the three conditions they publish; there is no fourth, so
/// there is no interpolation between them here and a caller wanting the
/// surface between these points is on its own. Recording the table rather
/// than fitting a curve through three points is deliberate.
pub const THD_UNTRIMMED: ThdTable = ThdTable {
    unity_gain_0dbv: [0.00005, 0.00010, 0.00030],
    minus_15db_gain_plus_10dbv: [0.00020, 0.00030, 0.00040],
    plus_15db_gain_minus_5dbv: [0.00020, 0.00030, 0.00040],
};

/// The published distortion table, indexed by grade: A, B, then C.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThdTable {
    /// 0 dBV in, 0 dB gain.
    pub unity_gain_0dbv: [f32; 3],
    /// +10 dBV in, −15 dB gain.
    pub minus_15db_gain_plus_10dbv: [f32; 3],
    /// −5 dBV in, +15 dB gain.
    pub plus_15db_gain_minus_5dbv: [f32; 3],
}

/// One Blackmer gain cell.
///
/// Construct from [`BlackmerCell::TYPICAL`] and adjust, rather than
/// filling every field, so that a field added later does not break a
/// caller:
///
/// ```
/// use noob_electrical_components_blackmer_cell::BlackmerCell;
/// let warm = BlackmerCell { temp_c: 45.0, ..BlackmerCell::TYPICAL };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlackmerCell {
    /// Gain-control constant in millivolts per decibel, before
    /// temperature. See [`K_TYP_MV_PER_DB`] and [`K_RANGE_MV_PER_DB`].
    pub k_mv_per_db: f32,
    /// Chip temperature in degrees Celsius. The datasheet references its
    /// coefficient to a chip at [`TEMPCO_REF_C`], which is warmer than the
    /// ambient it also quotes, because the part heats itself.
    pub temp_c: f32,
    /// Residual symmetry error at the control port, in millivolts. Zero is
    /// a perfectly trimmed cell. See [`SYMMETRY_WINDOW_MV`] for the window
    /// an untrimmed one sits in.
    pub symmetry_mv: f32,
    /// Control-law linearity error, as a percentage of the specified span.
    /// Zero gives the exact exponential law. See [`LINEARITY_TYP_PCT`].
    pub linearity_pct: f32,
    /// The cell's even-order residual, given as the total harmonic
    /// distortion it produces at 0 dBV in at 0 dB gain, as a fraction.
    ///
    /// Expressed as the published measurement rather than as a polynomial
    /// coefficient on purpose: the coefficient depends on how the caller
    /// scales its signal, the measurement does not, and it is the number
    /// THAT actually print. See [`THD_UNTRIMMED`] for the grades.
    pub thd_unity: f32,
}

impl BlackmerCell {
    /// A typically-graded cell, trimmed, at its reference temperature.
    ///
    /// Symmetry is zero because this represents a cell somebody has
    /// adjusted, which is what both documented users ship: SSL fit a
    /// `DISTORTION NULL` trimmer and dbx's factory procedure sets one.
    pub const TYPICAL: Self = Self {
        k_mv_per_db: K_TYP_MV_PER_DB,
        temp_c: TEMPCO_REF_C,
        symmetry_mv: 0.0,
        linearity_pct: 0.0,
        thd_unity: 0.0,
    };

    /// An untrimmed typical cell, carrying the symmetry residual and the
    /// typical control-law bow. Use this to hear what the part does when
    /// nobody has nulled it.
    pub const UNTRIMMED: Self = Self {
        symmetry_mv: SYMMETRY_WINDOW_MV,
        linearity_pct: LINEARITY_TYP_PCT,
        thd_unity: THD_UNTRIMMED.unity_gain_0dbv[GRADE_B],
        ..Self::TYPICAL
    };

    /// The gain-control constant at this cell's temperature, in millivolts
    /// per decibel.
    #[inline]
    #[must_use]
    pub fn k_at_temp(&self) -> f32 {
        self.k_mv_per_db * (1.0 + TEMPCO_PER_C * (self.temp_c - TEMPCO_REF_C))
    }

    /// Gain in decibels for the voltages on the two control ports, in
    /// millivolts.
    ///
    /// The part has a positive and a negative port, published at +6.1 and
    /// −6.1 mV/dB, so they subtract: the cell responds to the difference
    /// between them. Most callers drive one port and pass `0.0` for the
    /// other.
    ///
    /// The caller owns whatever divider produced these voltages. That is
    /// the boundary this crate draws and it is not a detail: the dbx's
    /// control port sees a log-domain true-RMS detector and the SSL's sees
    /// a rectified average, and neither belongs here.
    #[inline]
    #[must_use]
    pub fn gain_db(&self, ec_plus_mv: f32, ec_minus_mv: f32) -> f32 {
        let ideal = (ec_plus_mv - ec_minus_mv - self.symmetry_mv) / self.k_at_temp();
        ideal + self.linearity_error_db(ideal)
    }

    /// The control voltage, in millivolts, that produces a wanted gain on
    /// the negative port with the positive port grounded.
    ///
    /// The inverse of [`gain_db`](Self::gain_db) for the common
    /// single-port case, so a caller that thinks in decibels can find the
    /// voltage rather than assuming the constant. The linearity bow is not
    /// inverted, so this is exact only for a cell whose `linearity_pct` is
    /// zero; the doc comment says so because silently returning an
    /// approximation from a function named like an inverse is how a model
    /// acquires an error nobody can find.
    #[inline]
    #[must_use]
    pub fn control_mv_for_gain(&self, gain_db: f32) -> f32 {
        -self.symmetry_mv - gain_db * self.k_at_temp()
    }

    /// Deviation of the real control law from a straight line, in
    /// decibels, at a given ideal gain.
    ///
    /// **The magnitude here is published and the shape is not.** THAT
    /// specify 0.5 % typical and 2 % maximum over a 100 dB span but say
    /// nothing about how the error is distributed within it, so the bow
    /// used is an estimate: a half-cycle sine that is zero at both ends of
    /// [`CONTROL_SPAN_DB`] and largest in the middle, which is the usual
    /// shape of a best-fit-line residual. The tests assert the published
    /// bound, never this curve, and a caller that wants no invention
    /// leaves `linearity_pct` at zero, which [`TYPICAL`](Self::TYPICAL)
    /// does.
    #[inline]
    #[must_use]
    pub fn linearity_error_db(&self, gain_db: f32) -> f32 {
        if self.linearity_pct == 0.0 {
            return 0.0;
        }
        let (lo, hi) = CONTROL_SPAN_DB;
        let span = hi - lo;
        let u = ((gain_db - lo) / span).clamp(0.0, 1.0);
        self.linearity_pct / 100.0 * span * (u * core::f32::consts::PI).sin()
    }
}

/// Index of the A grade in [`ThdTable`]'s rows.
pub const GRADE_A: usize = 0;
/// Index of the B grade in [`ThdTable`]'s rows.
pub const GRADE_B: usize = 1;
/// Index of the C grade in [`ThdTable`]'s rows.
pub const GRADE_C: usize = 2;

/// Peak volts of a 0 dBV sine, the level THAT quote their distortion at.
///
/// 0 dBV is one volt RMS, so the peak is the square root of two. This
/// constant is the reason [`BlackmerCell::process`] is documented as
/// taking **volts**: an even-order coefficient has units of reciprocal
/// volts, so a caller feeding it signal normalised to full scale rather
/// than referenced to a volt will get the wrong amount of distortion and
/// no error message.
pub const DBV_PEAK_VOLTS: f32 = core::f32::consts::SQRT_2;

impl BlackmerCell {
    /// The even-order coefficient, in reciprocal volts, that produces this
    /// cell's [`thd_unity`](Self::thd_unity).
    ///
    /// For a squared residual driven by a sine of peak amplitude A, the
    /// second harmonic comes out at half the coefficient times A squared
    /// against a fundamental of A, so the distortion fraction is the
    /// coefficient times A over two, and the coefficient follows.
    #[inline]
    #[must_use]
    pub fn even_coefficient(&self) -> f32 {
        2.0 * self.thd_unity / DBV_PEAK_VOLTS
    }

    /// Apply the cell's own distortion to one sample, in **volts**.
    ///
    /// This is the residual of a part whose two halves go through
    /// different transistors. If those paths are not identical the two
    /// halves are not amplified identically, and an asymmetric transfer
    /// curve is an even-order one, which is why the part has a symmetry
    /// trim pin at all and why dbx's factory procedure adjusts it for
    /// minimum distortion.
    ///
    /// This is the cell's imperfection, not a colour a machine adds. That
    /// distinction is load-bearing: in the Distressor a separate
    /// switchable generator sits after the gain element, while in the SSL
    /// the gain element **is** the distortion and there is no such stage
    /// anywhere in the audio path. Callers wanting the former should write
    /// it themselves; this is only the latter.
    ///
    /// A squared term has a mean, so this emits a small direct-current
    /// offset, exactly as the real cell does. It is not removed here
    /// because the coupling that removes it in hardware is downstream of
    /// the part, and because a high-pass filter is infrastructure rather
    /// than a component and this repository keeps those out.
    #[inline]
    #[must_use]
    pub fn process(&self, x_volts: f32) -> f32 {
        if self.thd_unity == 0.0 {
            return x_volts;
        }
        x_volts + self.even_coefficient() * x_volts * x_volts
    }
}

#[cfg(test)]
mod tests;
