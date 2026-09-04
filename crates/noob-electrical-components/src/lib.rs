//! Physical models of the electrical components audio hardware is built
//! from, gathered behind one dependency.
//!
//! Each component is its own crate, so it can be read, tested and
//! documented as the single thing it is. This crate re-exports them behind
//! a feature each, so a plug-in writes one dependency line and turns on
//! what it actually uses; a compressor that needs a photocell does not
//! compile a transformer it will never call.
//!
//! ```toml
//! [dependencies]
//! noob-electrical-components = { git = "...", features = ["photocell"] }
//! ```
//!
//! # What belongs here
//!
//! A component earns a place once something real shares it, or is about to.
//! The point is not to atomise a codebase into parts that each have one
//! caller: an abstraction pulled out of a single user is usually the wrong
//! shape for the second one. The photocell qualified because two
//! compressors already shared it and a third established where its edge
//! lies by deliberately not using it.
//!
//! What does not belong here is circuitry. A component is the part; the
//! resistor network around it, the sidechain that drives it and the
//! make-up gain after it are the machine, and they differ from unit to
//! unit while the part does not. Nor does general signal processing:
//! filters, oversamplers and anti-aliasing are infrastructure rather than
//! components, and they live elsewhere.

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "photocell")]
#[cfg_attr(docsrs, doc(cfg(feature = "photocell")))]
pub use noob_electrical_components_photocell as photocell;

#[cfg(feature = "diode-bridge")]
#[cfg_attr(docsrs, doc(cfg(feature = "diode-bridge")))]
pub use noob_electrical_components_diode_bridge as diode_bridge;

#[cfg(feature = "blackmer-cell")]
#[cfg_attr(docsrs, doc(cfg(feature = "blackmer-cell")))]
pub use noob_electrical_components_blackmer_cell as blackmer_cell;

#[cfg(feature = "remote-cutoff-triode")]
#[cfg_attr(docsrs, doc(cfg(feature = "remote-cutoff-triode")))]
pub use noob_electrical_components_remote_cutoff_triode as remote_cutoff_triode;

#[cfg(feature = "diode-arm-pair")]
#[cfg_attr(docsrs, doc(cfg(feature = "diode-arm-pair")))]
pub use noob_electrical_components_diode_arm_pair as diode_arm_pair;

#[cfg(feature = "fet-variable-resistor")]
#[cfg_attr(docsrs, doc(cfg(feature = "fet-variable-resistor")))]
pub use noob_electrical_components_fet_variable_resistor as fet_variable_resistor;

#[cfg(feature = "log-rms-detector")]
#[cfg_attr(docsrs, doc(cfg(feature = "log-rms-detector")))]
pub use noob_electrical_components_log_rms_detector as log_rms_detector;

#[cfg(feature = "small-signal-triode")]
#[cfg_attr(docsrs, doc(cfg(feature = "small-signal-triode")))]
pub use noob_electrical_components_small_signal_triode as small_signal_triode;

#[cfg(feature = "transformer")]
#[cfg_attr(docsrs, doc(cfg(feature = "transformer")))]
pub use noob_electrical_components_transformer as transformer;
