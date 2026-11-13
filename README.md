# TCL Brightness

A small Windows tray app for adjusting TCL 27C2A brightness. Built with Rust.

## Use

Run `TCL Brightness.exe`, then click the sun icon by the clock and move the slider. Click outside or press Escape to close the flyout. Right-click the icon to exit.

- **HDR on:** adjusts desktop brightness while keeping HDR highlights bright.
- **HDR off:** adjusts the monitor backlight through DDC/CI.
- **Keyboard:** Win + Alt + Page Up / Down adjusts brightness by 5%.

Windows 11 required. Supports the TCL 27C2A; other models are not supported. No ads, tracking, network access, or paid dependencies.

## Build

Install Rust for Windows, then run:

```powershell
cargo build --release
.\install.ps1
```

The install script adds Start menu and Windows startup shortcuts. See [development notes](docs/development.md) for checks and command-line options.

## License

[MIT](LICENSE).
