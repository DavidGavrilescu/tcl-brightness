#![windows_subsystem = "windows"]
use std::{
    ffi::c_void,
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
};
type H = *mut c_void;
type WndProc = unsafe extern "system" fn(H, u32, usize, isize) -> isize;
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct WndClass {
    style: u32,
    proc: Option<WndProc>,
    cls_extra: i32,
    win_extra: i32,
    instance: H,
    icon: H,
    cursor: H,
    background: H,
    menu: *const u16,
    class: *const u16,
}
#[repr(C)]
struct Msg {
    hwnd: H,
    msg: u32,
    wp: usize,
    lp: isize,
    time: u32,
    pt: Point,
    private: u32,
}
#[repr(C)]
struct MonitorInfo {
    size: u32,
    monitor: Rect,
    work: Rect,
    flags: u32,
    device: [u16; 32],
}
#[repr(C)]
struct Physical {
    handle: H,
    description: [u16; 128],
}
#[repr(C)]
struct NotifyIcon {
    size: u32,
    hwnd: H,
    id: u32,
    flags: u32,
    callback: u32,
    icon: H,
    tip: [u16; 128],
    state: u32,
    state_mask: u32,
    info: [u16; 256],
    version: u32,
    title: [u16; 64],
    info_flags: u32,
    guid: [u8; 16],
    balloon: H,
}
#[link(name = "user32")]
extern "system" {
    fn GetDisplayConfigBufferSizes(flags: u32, paths: *mut u32, modes: *mut u32) -> i32;
    fn QueryDisplayConfig(
        flags: u32,
        paths: *mut u32,
        path: *mut c_void,
        modes: *mut u32,
        mode: *mut c_void,
        topology: *mut u32,
    ) -> i32;
    fn DisplayConfigGetDeviceInfo(info: *mut c_void) -> i32;
    fn DisplayConfigSetDeviceInfo(info: *const c_void) -> i32;
    fn EnumDisplayMonitors(
        dc: H,
        rect: *const Rect,
        callback: Option<unsafe extern "system" fn(H, H, *mut Rect, isize) -> i32>,
        data: isize,
    ) -> i32;
    fn GetMonitorInfoW(monitor: H, info: *mut MonitorInfo) -> i32;
    fn RegisterClassW(class: *const WndClass) -> u16;
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: H,
        menu: H,
        instance: H,
        param: H,
    ) -> H;
    fn DefWindowProcW(hwnd: H, msg: u32, wp: usize, lp: isize) -> isize;
    fn GetMessageW(msg: *mut Msg, hwnd: H, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn ShowWindow(hwnd: H, cmd: i32) -> i32;
    fn SetForegroundWindow(hwnd: H) -> i32;
    fn SetWindowPos(hwnd: H, after: H, x: i32, y: i32, w: i32, h: i32, flags: u32) -> i32;
    fn SetWindowTextW(hwnd: H, text: *const u16) -> i32;
    fn SendMessageW(hwnd: H, msg: u32, wp: usize, lp: isize) -> isize;
    fn PostMessageW(hwnd: H, msg: u32, wp: usize, lp: isize) -> i32;
    fn PostQuitMessage(code: i32);
    fn GetCursorPos(pt: *mut Point) -> i32;
    fn MonitorFromPoint(pt: Point, flags: u32) -> H;
    fn LoadCursorW(instance: H, name: *const u16) -> H;
    fn CreateIcon(
        instance: H,
        w: i32,
        h: i32,
        planes: u8,
        bits: u8,
        and: *const u8,
        xor: *const u8,
    ) -> H;
    fn DestroyIcon(icon: H) -> i32;
    fn CreatePopupMenu() -> H;
    fn AppendMenuW(menu: H, flags: u32, id: usize, text: *const u16) -> i32;
    fn TrackPopupMenu(
        menu: H,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: H,
        rect: *const Rect,
    ) -> i32;
    fn DestroyMenu(menu: H) -> i32;
    fn FindWindowW(class: *const u16, title: *const u16) -> H;
    fn RegisterHotKey(hwnd: H, id: i32, mods: u32, key: u32) -> i32;
    fn UnregisterHotKey(hwnd: H, id: i32) -> i32;
    fn RegisterWindowMessageW(text: *const u16) -> u32;
    fn SetTimer(hwnd: H, id: usize, ms: u32, callback: H) -> usize;
    fn KillTimer(hwnd: H, id: usize) -> i32;
    fn EnableWindow(hwnd: H, enable: i32) -> i32;

}
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> H;
    fn LoadLibraryExW(name: *const u16, file: H, flags: u32) -> H;
    fn GetProcAddress(module: H, name: *const u8) -> *mut c_void;
}
#[allow(non_snake_case)]
unsafe fn dxva_function(name: &[u8]) -> *mut c_void {
    static DLL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let module =
        *DLL.get_or_init(|| LoadLibraryExW(wide("dxva2.dll").as_ptr(), null_mut(), 0x800) as usize);
    if module == 0 {
        return null_mut();
    }
    GetProcAddress(module as H, name.as_ptr())
}
macro_rules! dxva {
    ($name:ident, ($($arg:ident:$ty:ty),*)) => {
        #[allow(non_snake_case)] unsafe fn $name($($arg:$ty),*)->i32 {
            let ptr=dxva_function(concat!(stringify!($name),"\0").as_bytes());
            if ptr.is_null(){return 0}
            let f:unsafe extern "system" fn($($ty),*)->i32=std::mem::transmute(ptr);f($($arg),*)
        }
    }
}
dxva!(GetNumberOfPhysicalMonitorsFromHMONITOR,(m:H,n:*mut u32));
dxva!(GetPhysicalMonitorsFromHMONITOR,(m:H,n:u32,p:*mut Physical));
dxva!(DestroyPhysicalMonitors,(n:u32,p:*mut Physical));
dxva!(GetVCPFeatureAndVCPFeatureReply,(h:H,code:u8,t:*mut u32,v:*mut u32,max:*mut u32));
dxva!(SetVCPFeature,(h:H,code:u8,value:u32));
dxva!(GetMonitorBrightness,(h:H,min:*mut u32,current:*mut u32,max:*mut u32));
dxva!(SetMonitorBrightness,(h:H,value:u32));
#[link(name = "gdi32")]
extern "system" {
    fn CreateFontW(
        height: i32,
        width: i32,
        angle: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike: u32,
        charset: u32,
        outprecision: u32,
        clip: u32,
        quality: u32,
        pitch: u32,
        name: *const u16,
    ) -> H;
    fn DeleteObject(object: H) -> i32;
}
#[link(name = "shell32")]
extern "system" {
    fn Shell_NotifyIconW(msg: u32, data: *const NotifyIcon) -> i32;
}
#[link(name = "comctl32")]
extern "system" {
    fn InitCommonControls();
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn utf16(s: &[u16]) -> String {
    String::from_utf16_lossy(&s[..s.iter().position(|v| *v == 0).unwrap_or(s.len())])
}
#[derive(Clone, Debug)]
struct Display {
    adapter: [u32; 2],
    id: u32,
    name: String,
    gdi: String,
    hdr: bool,
    path: String,
}
fn header(t: u32, size: u32, d: &Display) -> [u32; 5] {
    [t, size, d.adapter[0], d.adapter[1], d.id]
}
fn displays() -> Result<Vec<Display>, String> {
    unsafe {
        for _ in 0..4 {
            let (mut np, mut nm) = (0, 0);
            let e = GetDisplayConfigBufferSizes(2, &mut np, &mut nm);
            if e != 0 {
                return Err(format!("Display query failed ({e})."));
            }
            let mut paths = vec![0u32; np as usize * 18];
            let mut modes = vec![0u64; nm as usize * 8];
            let e = QueryDisplayConfig(
                2,
                &mut np,
                paths.as_mut_ptr().cast(),
                &mut nm,
                modes.as_mut_ptr().cast(),
                null_mut(),
            );
            if e == 122 {
                continue;
            }
            if e != 0 {
                return Err(format!("Display query failed ({e})."));
            }
            let mut found = Vec::new();
            for p in paths.chunks_exact(18).take(np as usize) {
                let mut d = Display {
                    adapter: [p[5], p[6]],
                    id: p[7],
                    name: String::new(),
                    gdi: String::new(),
                    hdr: false,
                    path: String::new(),
                };
                let mut name = [0u32; 105];
                name[..5].copy_from_slice(&header(2, 420, &d));
                if DisplayConfigGetDeviceInfo(name.as_mut_ptr().cast()) != 0 {
                    continue;
                }
                let chars = std::slice::from_raw_parts(name.as_ptr().cast::<u16>(), 210);
                d.name = utf16(&chars[18..82]);
                d.path = utf16(&chars[82..210]);
                if !d.name.to_uppercase().contains("27C2A")
                    && !d.path.to_uppercase().contains("TCL2005")
                    && !d.name.eq_ignore_ascii_case("G27Q")
                {
                    continue;
                }
                if d.name.is_empty() {
                    d.name = "Display".into();
                }
                let mut source = [0u32; 21];
                source[..5].copy_from_slice(&[1, 84, p[0], p[1], p[2]]);
                if DisplayConfigGetDeviceInfo(source.as_mut_ptr().cast()) != 0 {
                    return Err("Cannot identify monitor connection.".into());
                }
                d.gdi = utf16(std::slice::from_raw_parts(
                    source.as_ptr().cast::<u16>().add(10),
                    32,
                ));
                let mut color = [0u32; 9];
                color[..5].copy_from_slice(&header(15, 36, &d));
                let e = DisplayConfigGetDeviceInfo(color.as_mut_ptr().cast());
                if e != 0 {
                    return Err(format!("Cannot determine HDR mode ({e})."));
                }
                d.hdr = color[8] == 2;
                found.push(d);
            }
            return Ok(found);
        }
        Err("Displays are changing. Try again.".into())
    }
}
struct Search {
    gdi: String,
    found: H,
}
unsafe extern "system" fn find_monitor(h: H, _: H, _: *mut Rect, data: isize) -> i32 {
    let s = &mut *(data as *mut Search);
    let mut info: MonitorInfo = zeroed();
    info.size = size_of::<MonitorInfo>() as u32;
    if GetMonitorInfoW(h, &mut info) != 0 && utf16(&info.device) == s.gdi {
        s.found = h;
        return 0;
    }
    1
}
fn with_physical<T>(d: &Display, f: impl FnOnce(H) -> Result<T, String>) -> Result<T, String> {
    unsafe {
        let mut search = Search {
            gdi: d.gdi.clone(),
            found: null_mut(),
        };
        EnumDisplayMonitors(
            null_mut(),
            null(),
            Some(find_monitor),
            &mut search as *mut _ as isize,
        );
        if search.found.is_null() {
            return Err("Monitor connection changed. Try again.".into());
        }
        let mut n = 0;
        if GetNumberOfPhysicalMonitorsFromHMONITOR(search.found, &mut n) == 0 || n == 0 {
            return Err("Monitor brightness control is unavailable.".into());
        }
        let mut handles: Vec<Physical> = (0..n).map(|_| zeroed()).collect();
        if GetPhysicalMonitorsFromHMONITOR(search.found, n, handles.as_mut_ptr()) == 0 {
            return Err("Cannot open monitor controls.".into());
        }
        let result = f(handles[0].handle);
        DestroyPhysicalMonitors(n, handles.as_mut_ptr());
        result
    }
}

fn retry_monitor<T>(mut op: impl FnMut() -> Result<T, String>) -> Result<T, String> {
    let mut last = Err("Display not responding. Try again.".into());
    for attempt in 0..3 {
        last = op();
        if last.is_ok() {
            return last;
        }
        if attempt < 2 {
            std::thread::sleep(std::time::Duration::from_millis(90));
        }
    }
    last
}
fn read_backlight(d: &Display) -> Result<u32, String> {
    with_physical(d, |h| unsafe {
        let (mut min, mut current, mut max) = (0, 0, 0);
        if GetMonitorBrightness(h, &mut min, &mut current, &mut max) != 0 && max > min {
            return Ok((current.saturating_sub(min) * 100 / (max - min)).min(100));
        }
        let mut t = 0;
        if GetVCPFeatureAndVCPFeatureReply(h, 0x10, &mut t, &mut current, &mut max) != 0 && max > 0
        {
            return Ok((current * 100 / max).min(100));
        }
        Err("Display not responding. Check DDC/CI in its menu and try again.".into())
    })
}
fn write_backlight(d: &Display, value: u32) -> Result<(), String> {
    with_physical(d, |h| unsafe {
        if d.name.eq_ignore_ascii_case("G27Q") {
            if SetMonitorBrightness(h, value) != 0 {
                std::thread::sleep(std::time::Duration::from_millis(150));
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(90));
            if SetVCPFeature(h, 0x10, value) != 0 {
                std::thread::sleep(std::time::Duration::from_millis(150));
                return Ok(());
            }
            return Err("Monitor is busy. Try again.".into());
        }
        let (mut min, mut current, mut max) = (0, 0, 0);
        if GetMonitorBrightness(h, &mut min, &mut current, &mut max) != 0
            && max > min
            && SetMonitorBrightness(h, min + value * (max - min) / 100) != 0
        {
            return Ok(());
        }
        let mut t = 0;
        if GetVCPFeatureAndVCPFeatureReply(h, 0x10, &mut t, &mut current, &mut max) != 0
            && max > 0
            && SetVCPFeature(h, 0x10, value * max / 100) != 0
        {
            return Ok(());
        }
        Err("Display rejected the change. Check DDC/CI in its menu and try again.".into())
    })
}
fn brightness(d: &Display) -> Result<u32, String> {
    unsafe {
        if d.hdr {
            let mut b = [0u32; 6];
            b[..5].copy_from_slice(&header(11, 24, d));
            let e = DisplayConfigGetDeviceInfo(b.as_mut_ptr().cast());
            if e != 0 {
                return Err(format!("Cannot read HDR desktop brightness ({e})."));
            }
            return Ok(b[5].saturating_sub(1000).div_ceil(50).min(100));
        }
        retry_monitor(|| read_backlight(d))
    }
}
fn set_brightness(d: &Display, value: u32) -> Result<(), String> {
    unsafe {
        if value > 100 {
            return Err("Brightness must be between 0 and 100.".into());
        }
        if d.hdr {
            // Windows' SDR-content brightness control. This private setter is used by existing HDR utilities;
            // fail visibly if Windows rejects it. HDR mode and peak highlight luminance are preserved.
            let mut b = [0u32; 7];
            b[..5].copy_from_slice(&header(0xffff_ffee, 28, d));
            b[5] = 1000 + value * 50;
            b[6] = 1;
            let e = DisplayConfigSetDeviceInfo(b.as_ptr().cast());
            if e != 0 {
                return Err(format!("Windows rejected desktop brightness ({e}). Use Display > HDR > SDR content brightness."));
            }
            Ok(())
        } else {
            retry_monitor(|| write_backlight(d, value))
        }
    }
}
unsafe fn sun_icon(instance: H) -> H {
    let mut pixels = [0u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            let r = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);
            if r < 7.0 || (r > 10.0 && r < 14.0 && (angle * 4.0).sin().abs() < 0.27) {
                let i = (y * 32 + x) * 4;
                pixels[i] = 70;
                pixels[i + 1] = 200;
                pixels[i + 2] = 255;
                pixels[i + 3] = 255;
            }
        }
    }
    CreateIcon(
        instance,
        32,
        32,
        1,
        32,
        [0u8; 128].as_ptr(),
        pixels.as_ptr(),
    )
}

fn find_display(query: Option<&str>) -> Result<Display, String> {
    let all = displays()?;
    if let Some(query) = query {
        let matches: Vec<_> = all
            .into_iter()
            .filter(|d| {
                d.path.eq_ignore_ascii_case(query)
                    || d.name.eq_ignore_ascii_case(query)
                    || d.gdi.eq_ignore_ascii_case(query)
            })
            .collect();
        if matches.len() != 1 {
            return Err(
                "Select one connected display by its unique name, path or GDI identifier.".into(),
            );
        }
        return Ok(matches.into_iter().next().unwrap());
    }
    all.iter()
        .find(|d| d.name.to_uppercase().contains("27C2A"))
        .cloned()
        .or_else(|| all.into_iter().next())
        .ok_or("Connect a TCL 27C2A monitor.".into())
}
fn json(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn status_json(d: &Display) -> Result<String, String> {
    let v = brightness(d)?;
    Ok(format!(
        "{{\"monitor\":{},\"mode\":\"{}\",\"brightness\":{},\"gdi\":{},\"path\":{}}}",
        json(&d.name),
        if d.hdr { "hdr-desktop" } else { "backlight" },
        v,
        json(&d.gdi),
        json(&d.path)
    ))
}

fn cli(args: &[String]) -> Result<String, String> {
    let query = args
        .iter()
        .position(|s| s == "--monitor")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str);
    match args.first().map(String::as_str){
        Some("--list")=>Ok(format!("[{}]",displays()?.iter().map(|d|format!("{{\"monitor\":{},\"hdr\":{},\"gdi\":{},\"path\":{}}}",json(&d.name),d.hdr,json(&d.gdi),json(&d.path))).collect::<Vec<_>>().join(","))),
        Some("--status")=>status_json(&find_display(query)?),
        Some("--set")=>{let v=args.get(1).ok_or("Expected --set 0..100")?.parse::<u32>().map_err(|_|"Expected brightness 0..100")?;let d=find_display(query)?;set_brightness(&d,v)?;status_json(&d)},
        Some("--self-test")=>{
            assert_eq!(size_of::<Physical>(),264);assert_eq!(size_of::<NotifyIcon>(),976);assert_eq!(size_of::<MonitorInfo>(),104);
            ui::test_layouts();assert_eq!(wide("TCL"),[84,67,76,0]);assert_eq!(utf16(&[65,0,66]),"A");assert_eq!(json("a\"\\\n"),"\"a\\\"\\\\\\n\"");
            assert!(set_brightness(&Display{adapter:[0,0],id:0,name:String::new(),gdi:String::new(),hdr:true,path:String::new()},101).is_err());
            Ok("ABI, JSON encoding and brightness-bound checks passed.".into())
        },
        _=>Err("Usage: --list | --status | --set 0..100 [--monitor NAME] | --self-test | --show | --quit".into())
    }
}
mod ui;
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s != "--show" && s != "--quit") {
        let (text, mut code) = match cli(&args) {
            Ok(s) => (s, 0),
            Err(e) => (format!("Error: {e}"), 1),
        };
        if let Some(i) = args.iter().position(|s| s == "--out") {
            if let Some(p) = args.get(i + 1) {
                if std::fs::write(p, &text).is_err() {
                    code = 1
                }
            }
        }
        std::process::exit(code)
    }
    unsafe { ui::run(&args) }
}
