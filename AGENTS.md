# TCL Brightness instructions

Read README.md before changes. Preserve unrelated changes.

- Language: Rust 2021; Windows 11 native Win32 UI and FFI. No crates unless necessary.
- Build/format check: `./build.ps1 -Check`
- Hardware integration verification: `./verify.ps1` (temporarily changes brightness by 1%, then restores it).
- Install/run: `./install.ps1`
- Formatting: `cargo fmt` with the project-local Rust toolchain configured by build.ps1.
- Test exact native ABI layout, monitor selection, error reporting, and actual read-back. Never silently target another monitor.
- Keep toolchains, build outputs, and temporary test results untracked.
- Record HDR API limitations and validated modes accurately in README.md.
