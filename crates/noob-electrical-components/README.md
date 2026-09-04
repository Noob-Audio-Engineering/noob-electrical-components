# noob-electrical-components

The facade: every component crate in this workspace, re-exported behind a
feature, so a plug-in takes one dependency and compiles in only what it
uses.

```toml
[dependencies]
noob-electrical-components = { git = "https://github.com/Noob-Audio-Engineering/noob-electrical-components", features = ["photocell"] }
```

| Feature | Re-exports as | Component |
|---|---|---|
| `photocell` | `noob_electrical_components::photocell` | The T4-family light-dependent resistor. |
| `diode-bridge` | `noob_electrical_components::diode_bridge` | The balanced diode bridge used as a gain element. |
| `blackmer-cell` | `noob_electrical_components::blackmer_cell` | David Blackmer's log-antilog gain cell, and its control law. |
| `remote-cutoff-triode` | `noob_electrical_components::remote_cutoff_triode` | The remote-cutoff triode, the gain element of a variable-mu limiter. |
| `diode-arm-pair` | `noob_electrical_components::diode_arm_pair` | The diode arm pair of the EMI TG12413, which is not the diode bridge. |
| `log-rms-detector` | `noob_electrical_components::log_rms_detector` | Blackmer's log-domain true-RMS detector, the technique and no ballistics. |
| `fet-variable-resistor` | `noob_electrical_components::fet_variable_resistor` | A field-effect transistor used as a variable resistor, the 1176's gain element. |
| `small-signal-triode` | `noob_electrical_components::small_signal_triode` | The ordinary preamp valve: a 12AX7-class gain stage whose bias sets its asymmetry and never its gain. Not the remote-cutoff triode above. |
| `transformer` | `noob_electrical_components::transformer` | An audio transformer's low end: the roll-off its magnetising inductance puts under the band, and the flux its core can carry. |

See the workspace README for what belongs in this repository and what does
not.
