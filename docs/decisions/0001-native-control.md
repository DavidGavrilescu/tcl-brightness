# Native Windows controls

Use Rust and Win32 for a small tray utility with no external crates. Keep the flyout compact and keyboard accessible. Avoid continuous polling.

Support the TCL 27C2A explicitly. Other monitors may reject brightness commands and are outside the supported scope. In HDR, adjust Windows SDR white level. In SDR, use the monitor configuration APIs with a VCP fallback. Report failures without discarding the last confirmed value.
