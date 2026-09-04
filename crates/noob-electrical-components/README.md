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

See the workspace README for what belongs in this repository and what does
not.
