# Development

The app uses Rust's standard library and Win32 APIs. It has no external crates.

## Checks

```powershell
cargo fmt --check
cargo build --release
.\verify.ps1
```

`verify.ps1` reads brightness, changes it by one point, checks the result, and restores the original value. It requires a connected TCL 27C2A. An unsupported monitor must never be selected as a fallback.

## Controls

The flyout uses native accessible controls, dark styling, rounded corners, and per-monitor DPI scaling. It dismisses on focus loss or Escape. Slider writes are debounced by 100 ms. Failed commands retain the last confirmed brightness and allow another attempt. DDC/CI uses bounded retries and both high-level monitor APIs and VCP controls.

With HDR enabled, the app reads Windows SDR white level and sets it through the private `DisplayConfigSetDeviceInfo` request `0xFFFFFFEE`. This adjusts desktop brightness from 80 to 480 nits. HDR remains enabled. This private setter may require changes after a Windows update.

With HDR disabled, brightness uses DDC/CI. DDC/CI must be enabled in the monitor menu.

## Command line

The executable has no console. Add `--out` to save a command's result as text or JSON. Exit code 0 means success; 1 means failure.

```powershell
Start-Process '.\bin\TCL Brightness.exe' -ArgumentList '--list --out displays.json' -Wait
Start-Process '.\bin\TCL Brightness.exe' -ArgumentList '--status --out status.json' -Wait
Start-Process '.\bin\TCL Brightness.exe' -ArgumentList '--set 40 --out result.json' -Wait
Start-Process '.\bin\TCL Brightness.exe' -ArgumentList '--show'
Start-Process '.\bin\TCL Brightness.exe' -ArgumentList '--quit' -Wait
```

`--monitor` accepts an exact supported display name, GDI name, or device path. The picker saves the selected device path in `selected-monitor.txt` beside the executable. This local file must not be committed.

## Uninstall

Exit the app, remove its Start menu and Startup shortcuts, then delete the executable folder.
