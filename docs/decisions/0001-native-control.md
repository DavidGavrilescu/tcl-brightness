# Native Windows controls

Use Rust and Win32 for a small tray utility with no external crates. Keep the flyout compact and keyboard accessible. Avoid continuous polling.

Support the TCL 27C2A and Gigabyte G27Q explicitly. The G27Q uses its known 0–100 range so transient range-read failures cannot prevent writes. Retry busy operations, then confirm the result. In HDR, adjust Windows SDR white level. In SDR, use the monitor configuration APIs with a VCP fallback. Report failures without discarding the last confirmed value.
