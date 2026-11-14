use super::*;
use std::cell::RefCell;
#[repr(C)]
struct DrawItem {
    kind: u32,
    id: u32,
    item: u32,
    action: u32,
    state: u32,
    hwnd: H,
    dc: H,
    rect: Rect,
    data: usize,
}
#[repr(C)]
struct Paint {
    dc: H,
    erase: i32,
    rect: Rect,
    restore: i32,
    update: i32,
    reserved: [u8; 32],
}
#[link(name = "user32")]
extern "system" {
    fn BeginPaint(hwnd: H, paint: *mut Paint) -> H;
    fn EndPaint(hwnd: H, paint: *const Paint) -> i32;
    fn FillRect(dc: H, rect: *const Rect, brush: H) -> i32;
    fn DrawTextW(dc: H, text: *const u16, len: i32, rect: *mut Rect, flags: u32) -> i32;
    fn InvalidateRect(hwnd: H, rect: *const Rect, erase: i32) -> i32;
    fn GetClientRect(hwnd: H, rect: *mut Rect) -> i32;
    fn GetWindowRect(hwnd: H, rect: *mut Rect) -> i32;
    fn GetDpiForWindow(hwnd: H) -> u32;
    fn SetProcessDpiAwarenessContext(context: H) -> i32;
    fn AdjustWindowRectExForDpi(rect: *mut Rect, style: u32, menu: i32, ex: u32, dpi: u32) -> i32;
    fn GetFocus() -> H;
    fn GetForegroundWindow() -> H;
    fn SetFocus(hwnd: H) -> H;
    fn IsWindowEnabled(hwnd: H) -> i32;
    fn IsWindowVisible(hwnd: H) -> i32;
    fn IsDialogMessageW(hwnd: H, msg: *mut Msg) -> i32;
}
#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(color: u32) -> H;
    fn CreatePen(style: i32, width: i32, color: u32) -> H;
    fn SelectObject(dc: H, object: H) -> H;
    fn SetTextColor(dc: H, color: u32) -> u32;
    fn SetBkColor(dc: H, color: u32) -> u32;
    fn SetBkMode(dc: H, mode: i32) -> i32;
    fn RoundRect(dc: H, l: i32, t: i32, r: i32, b: i32, w: i32, h: i32) -> i32;
    fn Ellipse(dc: H, l: i32, t: i32, r: i32, b: i32) -> i32;
    fn MoveToEx(dc: H, x: i32, y: i32, old: *mut Point) -> i32;
    fn LineTo(dc: H, x: i32, y: i32) -> i32;
}
const BG: u32 = 0x202020;
const CARD: u32 = 0x2c2c2c;
const BORDER: u32 = 0x404040;
const WHITE: u32 = 0xf5f5f5;
const MUTED: u32 = 0xb8b8b8;
const ACCENT: u32 = 0xffc98e;
const SHOW: u32 = 0x8001;
const TRAY: u32 = 0x8002;
const QUIT: u32 = 0x8003;
const APPLIED: u32 = 0x8004;
static mut LATEST_JOB: u64 = 0;
const STYLE: u32 = 0x82040000;
static mut SLIDER: H = null_mut();
static mut SELECTOR: H = null_mut();
static mut VALUE: H = null_mut();
static mut MODE: H = null_mut();
static mut HELP: H = null_mut();
static mut ICON: H = null_mut();
static mut BRUSH: H = null_mut();
static mut TASKBAR: u32 = 0;
static mut DPI: u32 = 96;
static mut FONT: H = null_mut();
static mut SMALL: H = null_mut();
static mut BIG: H = null_mut();
static mut MEDIUM: H = null_mut();
static mut PENDING: bool = false;
static mut MENU_OPEN: bool = false;
static mut MAIN: H = null_mut();
static mut HAS_ERROR: bool = false;
thread_local! {static SELECTED:RefCell<Option<String>>=const{RefCell::new(None)};static LAST_GOOD:RefCell<Option<(String,u32)>>=const{RefCell::new(None)};}
fn selected_file() -> std::path::PathBuf {
    std::env::current_exe()
        .unwrap_or_default()
        .with_file_name("selected-monitor.txt")
}
fn selected_display() -> Result<Display, String> {
    let all = displays()?;
    let selected = SELECTED.with(|s| s.borrow().clone());
    if let Some(key) = selected {
        return all
            .into_iter()
            .find(|d| d.path == key)
            .ok_or("Selected display is disconnected. Choose another display above.".into());
    }
    let d = all
        .iter()
        .find(|d| d.name.to_uppercase().contains("27C2A"))
        .cloned()
        .or_else(|| all.into_iter().next())
        .ok_or("No connected displays. Connect a monitor and reopen this panel.".to_string())?;
    SELECTED.with(|s| *s.borrow_mut() = Some(d.path.clone()));
    Ok(d)
}
unsafe fn px(n: i32) -> i32 {
    (n as i64 * DPI as i64 / 96) as i32
}
unsafe fn set_text(h: H, s: &str) {
    SetWindowTextW(h, wide(s).as_ptr());
    InvalidateRect(h, null(), 1);
}
unsafe fn rounded(dc: H, r: Rect, color: u32, border: u32, radius: i32) {
    let b = CreateSolidBrush(color);
    let p = CreatePen(0, px(1).max(1), border);
    let ob = SelectObject(dc, b);
    let op = SelectObject(dc, p);
    RoundRect(dc, r.left, r.top, r.right, r.bottom, radius, radius);
    SelectObject(dc, ob);
    SelectObject(dc, op);
    DeleteObject(b);
    DeleteObject(p);
}
unsafe fn circle(dc: H, x: i32, y: i32, r: i32, color: u32) {
    let b = CreateSolidBrush(color);
    let p = CreatePen(0, 1, color);
    let ob = SelectObject(dc, b);
    let op = SelectObject(dc, p);
    Ellipse(dc, x - r, y - r, x + r, y + r);
    SelectObject(dc, ob);
    SelectObject(dc, op);
    DeleteObject(b);
    DeleteObject(p);
}
unsafe fn line(dc: H, x: i32, y: i32, x2: i32, y2: i32, color: u32, width: i32) {
    let p = CreatePen(0, width, color);
    let old = SelectObject(dc, p);
    MoveToEx(dc, x, y, null_mut());
    LineTo(dc, x2, y2);
    SelectObject(dc, old);
    DeleteObject(p);
}
unsafe fn draw_text(dc: H, s: &str, r: Rect, font: H, color: u32, flags: u32) {
    let old = SelectObject(dc, font);
    SetBkMode(dc, 1);
    SetTextColor(dc, color);
    let mut r = r;
    DrawTextW(dc, wide(s).as_ptr(), -1, &mut r, flags | 0x800);
    SelectObject(dc, old);
}
unsafe fn update(result: Result<(Display, u32), String>) {
    match result {
        Ok((d, v)) => {
            set_text(SELECTOR, &format!("Choose display: {}", d.name));
            set_text(VALUE, &format!("{v}%"));
            set_text(
                MODE,
                if d.hdr {
                    "Desktop brightness"
                } else {
                    "Brightness"
                },
            );
            HAS_ERROR = false;
            ShowWindow(HELP, 0);
            LAST_GOOD.with(|s| *s.borrow_mut() = Some((d.path.clone(), v)));
            SendMessageW(SLIDER, 0x405, 1, v as isize);
            EnableWindow(SLIDER, 1);
        }
        Err(_) => {
            let key = SELECTED.with(|s| s.borrow().clone());
            let cached = LAST_GOOD
                .with(|s| s.borrow().clone())
                .filter(|(path, _)| Some(path) == key.as_ref());
            if let Some((_, v)) = cached {
                set_text(VALUE, &format!("{v}%"));
                SendMessageW(SLIDER, 0x405, 1, v as isize);
                EnableWindow(SLIDER, 1);
            } else {
                set_text(VALUE, "—");
                EnableWindow(SLIDER, 0);
            }
            set_text(MODE, "Brightness");
            HAS_ERROR = true;
            set_text(HELP, "Display not responding. Try again.");
            ShowWindow(HELP, 5);
        }
    }
    if !MAIN.is_null() {
        fit_size(MAIN);
    }
    InvalidateRect(SELECTOR, null(), 1);
    InvalidateRect(SLIDER, null(), 1);
}
unsafe fn refresh() {
    if SLIDER.is_null() {
        return;
    }
    update(selected_display().and_then(|d| brightness(&d).map(|v| (d, v))));
}

struct Job {
    display: Display,
    value: u32,
    sequence: u64,
}
type Completed = (String, u64, Result<(Display, u32), String>);
static WORKER: std::sync::OnceLock<std::sync::mpsc::Sender<Job>> = std::sync::OnceLock::new();
unsafe fn apply(v: u32) {
    let d = match selected_display() {
        Ok(d) => d,
        Err(e) => {
            update(Err(e));
            return;
        }
    };
    let handle = MAIN as usize;
    let sender = WORKER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel::<Job>();
        std::thread::spawn(move || {
            while let Ok(mut job) = rx.recv() {
                while let Ok(newer) = rx.try_recv() {
                    job = newer;
                }
                let path = job.display.path.clone();
                let result = displays()
                    .and_then(|all| {
                        all.into_iter()
                            .find(|d| d.path == path)
                            .ok_or("Display disconnected.".to_string())
                    })
                    .and_then(|d| {
                        set_brightness(&d, job.value)?;
                        let actual = brightness(&d)?;
                        Ok((d, actual))
                    });
                let payload = Box::into_raw(Box::new((path, job.sequence, result)));
                if PostMessageW(handle as H, APPLIED, 0, payload as isize) == 0 {
                    drop(Box::from_raw(payload));
                }
            }
        });
        tx
    });
    LATEST_JOB = LATEST_JOB.wrapping_add(1);
    set_text(VALUE, &format!("{v}%"));
    SendMessageW(SLIDER, 0x405, 1, v as isize);
    if sender
        .send(Job {
            display: d,
            value: v,
            sequence: LATEST_JOB,
        })
        .is_err()
    {
        update(Err("Display control stopped. Reopen the app.".into()));
    }
}
unsafe fn flush(hwnd: H) {
    KillTimer(hwnd, 1);
    if PENDING {
        PENDING = false;
        apply(SendMessageW(SLIDER, 0x400, 0, 0) as u32);
    }
}
unsafe fn fonts() {
    for h in [FONT, SMALL, BIG, MEDIUM] {
        if !h.is_null() {
            DeleteObject(h);
        }
    }
    let make = |size: i32, weight: i32| {
        CreateFontW(
            -px(size),
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            1,
            0,
            0,
            5,
            0,
            wide("Segoe UI").as_ptr(),
        )
    };
    FONT = make(15, 400);
    SMALL = make(12, 400);
    BIG = make(30, 600);
    MEDIUM = make(16, 600);
    for (control, font) in [
        (MODE, FONT),
        (HELP, SMALL),
        (VALUE, BIG),
        (SELECTOR, MEDIUM),
    ] {
        if !control.is_null() {
            SendMessageW(control, 0x30, font as usize, 1);
        }
    }
}
unsafe fn layout(hwnd: H) {
    let dpi = GetDpiForWindow(hwnd);
    DPI = if dpi == 0 { 96 } else { dpi };
    fonts();
    for (h, x, y, w, height) in [
        (SELECTOR, 24, 22, 392, 64),
        (MODE, 26, 111, 250, 26),
        (VALUE, 300, 98, 114, 45),
        (SLIDER, 24, 145, 392, 42),
        (HELP, 26, 199, 388, 24),
    ] {
        SetWindowPos(h, null_mut(), px(x), px(y), px(w), px(height), 0x14);
    }
    SendMessageW(SLIDER, 0x41b, px(26) as usize, 0);
    InvalidateRect(hwnd, null(), 1);
}
unsafe fn fit_size(hwnd: H) {
    let mut r = Rect {
        left: 0,
        top: 0,
        right: px(440),
        bottom: px(if HAS_ERROR { 238 } else { 211 }),
    };
    AdjustWindowRectExForDpi(&mut r, STYLE, 0, 0x80, DPI);
    SetWindowPos(
        hwnd,
        null_mut(),
        0,
        0,
        r.right - r.left,
        r.bottom - r.top,
        0x16,
    );
}
unsafe fn dwm(hwnd: H, attribute: u32, value: u32) {
    static DLL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let h = *DLL
        .get_or_init(|| LoadLibraryExW(wide("dwmapi.dll").as_ptr(), null_mut(), 0x800) as usize);
    if h == 0 {
        return;
    }
    let ptr = GetProcAddress(h as H, b"DwmSetWindowAttribute\0".as_ptr());
    if ptr.is_null() {
        return;
    }
    let f: unsafe extern "system" fn(H, u32, *const u32, u32) -> i32 = std::mem::transmute(ptr);
    f(hwnd, attribute, &value, 4);
}
unsafe fn show(hwnd: H) {
    refresh();
    let mut pt = Point::default();
    GetCursorPos(&mut pt);
    let mon = MonitorFromPoint(pt, 2);
    let mut info: MonitorInfo = zeroed();
    info.size = size_of::<MonitorInfo>() as u32;
    GetMonitorInfoW(mon, &mut info);
    // Move onto the destination monitor first so per-monitor DPI is resolved before sizing.
    SetWindowPos(hwnd, null_mut(), pt.x, pt.y, 0, 0, 0x15);
    layout(hwnd);
    fit_size(hwnd);
    let mut r = Rect::default();
    GetWindowRect(hwnd, &mut r);
    let (w, h) = (r.right - r.left, r.bottom - r.top);
    let x = (pt.x - w / 2).clamp(info.work.left, (info.work.right - w).max(info.work.left));
    let y = (pt.y - h - px(12)).clamp(info.work.top, (info.work.bottom - h).max(info.work.top));
    SetWindowPos(hwnd, -1isize as H, x, y, w, h, 0x40);
    ShowWindow(hwnd, 5);
    SetForegroundWindow(hwnd);
    SetFocus(SLIDER);
    SendMessageW(hwnd, 0x127, 0x10001, 0);
}
unsafe fn notify(hwnd: H, op: u32) {
    let mut n: NotifyIcon = zeroed();
    n.size = size_of::<NotifyIcon>() as u32;
    n.hwnd = hwnd;
    n.id = 1;
    n.flags = 7;
    n.callback = TRAY;
    n.icon = ICON;
    for (i, c) in wide("Brightness - click to adjust your displays")
        .into_iter()
        .enumerate()
    {
        n.tip[i] = c;
    }
    Shell_NotifyIconW(op, &n);
}
unsafe fn choose(hwnd: H) {
    flush(hwnd);
    MENU_OPEN = true;
    let all = match displays() {
        Ok(v) => v,
        Err(e) => {
            set_text(HELP, &e);
            MENU_OPEN = false;
            return;
        }
    };
    let current = SELECTED.with(|s| s.borrow().clone());
    let menu = CreatePopupMenu();
    for (i, d) in all.iter().enumerate() {
        AppendMenuW(
            menu,
            if current.as_ref() == Some(&d.path) {
                8
            } else {
                0
            },
            500 + i,
            wide(&format!(
                "{}   ·   {}",
                d.name,
                if d.hdr { "HDR" } else { "SDR" }
            ))
            .as_ptr(),
        );
    }
    let mut r = Rect::default();
    GetWindowRect(SELECTOR, &mut r);
    SetForegroundWindow(hwnd);
    let choice = TrackPopupMenu(menu, 0x100 | 2, r.left, r.bottom + px(4), 0, hwnd, null());
    DestroyMenu(menu);
    MENU_OPEN = false;
    if GetForegroundWindow() != hwnd {
        ShowWindow(hwnd, 0);
    }
    if choice >= 500 {
        if let Some(d) = all.get(choice as usize - 500) {
            SELECTED.with(|s| *s.borrow_mut() = Some(d.path.clone()));
            let _ = std::fs::write(selected_file(), &d.path);
            refresh();
        }
    }
}
unsafe fn draw_selector(item: &DrawItem) {
    let r = item.rect;
    FillRect(item.dc, &r, BRUSH);
    let fill = if item.state & 1 != 0 { 0x383838 } else { CARD };
    rounded(
        item.dc,
        r,
        fill,
        if item.state & 0x10 != 0 {
            ACCENT
        } else {
            BORDER
        },
        px(9),
    );
    let d = selected_display();
    let (name, sub) = match d {
        Ok(d) => (
            d.name,
            if d.hdr {
                "HDR enabled"
            } else {
                "Standard dynamic range"
            }
            .to_string(),
        ),
        Err(_) => ("Choose a display".into(), "View connected monitors".into()),
    };
    let (x, y) = (r.left + px(18), r.top + px(21));
    rounded(
        item.dc,
        Rect {
            left: x,
            top: y,
            right: x + px(26),
            bottom: y + px(18),
        },
        fill,
        MUTED,
        px(3),
    );
    line(
        item.dc,
        x + px(13),
        y + px(18),
        x + px(13),
        y + px(23),
        MUTED,
        px(1),
    );
    line(
        item.dc,
        x + px(6),
        y + px(23),
        x + px(20),
        y + px(23),
        MUTED,
        px(1),
    );
    draw_text(
        item.dc,
        &name,
        Rect {
            left: r.left + px(58),
            top: r.top + px(10),
            right: r.right - px(35),
            bottom: r.top + px(34),
        },
        MEDIUM,
        WHITE,
        0x20 | 0x8000,
    );
    draw_text(
        item.dc,
        &sub,
        Rect {
            left: r.left + px(58),
            top: r.top + px(35),
            right: r.right - px(35),
            bottom: r.bottom - px(6),
        },
        SMALL,
        MUTED,
        0x20 | 0x8000,
    );
    let x = r.right - px(25);
    let y = (r.top + r.bottom) / 2;
    line(item.dc, x - px(4), y - px(2), x, y + px(2), MUTED, px(1));
    line(item.dc, x, y + px(2), x + px(4), y - px(2), MUTED, px(1));
}

#[link(name = "comctl32")]
extern "system" {
    fn SetWindowSubclass(
        hwnd: H,
        callback: Option<unsafe extern "system" fn(H, u32, usize, isize, usize, usize) -> isize>,
        id: usize,
        data: usize,
    ) -> i32;
    fn DefSubclassProc(hwnd: H, msg: u32, wp: usize, lp: isize) -> isize;
}
unsafe fn paint_slider(hwnd: H, dc: H) {
    let mut r = Rect::default();
    GetClientRect(hwnd, &mut r);
    FillRect(dc, &r, BRUSH);
    let mut channel = Rect::default();
    let mut thumb = Rect::default();
    SendMessageW(hwnd, 0x41a, 0, &mut channel as *mut _ as isize);
    SendMessageW(hwnd, 0x419, 0, &mut thumb as *mut _ as isize);
    let (x, y) = (
        (thumb.left + thumb.right) / 2,
        (thumb.top + thumb.bottom) / 2,
    );
    let accent = if IsWindowEnabled(hwnd) != 0 {
        ACCENT
    } else {
        0x777777
    };
    rounded(
        dc,
        Rect {
            left: channel.left,
            top: y - px(2),
            right: channel.right,
            bottom: y + px(2),
        },
        0x5b5b5b,
        0x5b5b5b,
        px(4),
    );
    if x > channel.left {
        rounded(
            dc,
            Rect {
                left: channel.left,
                top: y - px(2),
                right: x,
                bottom: y + px(2),
            },
            accent,
            accent,
            px(4),
        );
    }
    if GetFocus() == hwnd && SendMessageW(hwnd, 0x129, 0, 0) & 1 == 0 {
        circle(dc, x, y, px(12), 0x626262);
    }
    circle(dc, x, y, px(10), WHITE);
    circle(dc, x, y, px(5), accent);
}
unsafe extern "system" fn slider_proc(
    hwnd: H,
    msg: u32,
    wp: usize,
    lp: isize,
    _: usize,
    _: usize,
) -> isize {
    match msg {
        0x0f => {
            let mut p: Paint = zeroed();
            let dc = BeginPaint(hwnd, &mut p);
            paint_slider(hwnd, dc);
            EndPaint(hwnd, &p);
            0
        }
        0x318 => {
            paint_slider(hwnd, wp as H);
            0
        }
        0x14 => 1,
        _ => {
            let result = DefSubclassProc(hwnd, msg, wp, lp);
            if msg == 7 || msg == 8 || msg == 0x128 {
                InvalidateRect(hwnd, null(), 0);
            }
            result
        }
    }
}
unsafe fn quit(hwnd: H) {
    flush(hwnd);
    notify(hwnd, 2);
    UnregisterHotKey(hwnd, 1);
    UnregisterHotKey(hwnd, 2);
    PostQuitMessage(0);
}
unsafe extern "system" fn wndproc(hwnd: H, msg: u32, wp: usize, lp: isize) -> isize {
    match msg {
        APPLIED => {
            let result = Box::from_raw(lp as *mut Completed);
            let selected = SELECTED.with(|s| s.borrow().clone());
            if result.1 == LATEST_JOB && !PENDING && selected.as_ref() == Some(&result.0) {
                update(result.2);
            }
            return 0;
        }
        0x83 if wp != 0 => return 0,
        0x06 if wp & 0xffff == 0 && !MENU_OPEN => {
            flush(hwnd);
            ShowWindow(hwnd, 0);
            return 0;
        }
        0x10 => {
            flush(hwnd);
            ShowWindow(hwnd, 0);
            return 0;
        }
        0x2 | QUIT => {
            quit(hwnd);
            return 0;
        }
        SHOW => {
            show(hwnd);
            return 0;
        }
        0x0f => {
            let mut p: Paint = zeroed();
            let dc = BeginPaint(hwnd, &mut p);
            let mut r = Rect::default();
            GetClientRect(hwnd, &mut r);
            FillRect(dc, &r, BRUSH);
            EndPaint(hwnd, &p);
            return 0;
        }
        0x14 => return 1,
        0x138 => {
            let dc = wp as H;
            SetTextColor(
                dc,
                if lp as H == VALUE || lp as H == MODE {
                    WHITE
                } else {
                    MUTED
                },
            );
            SetBkColor(dc, BG);
            SetBkMode(dc, 1);
            return BRUSH as isize;
        }
        0x2b => {
            let item = &*(lp as *const DrawItem);
            if item.hwnd == SELECTOR {
                draw_selector(item);
                return 1;
            }
        }
        0x111 => {
            if wp & 0xffff == 101 {
                choose(hwnd);
                return 0;
            }
        }
        TRAY => {
            match lp as u32 {
                0x202 => {
                    if IsWindowVisible(hwnd) != 0 {
                        flush(hwnd);
                        ShowWindow(hwnd, 0);
                    } else {
                        show(hwnd)
                    }
                }
                0x205 => {
                    MENU_OPEN = true;
                    let menu = CreatePopupMenu();
                    AppendMenuW(menu, 0, 1, wide("Adjust brightness").as_ptr());
                    AppendMenuW(menu, 0, 2, wide("Exit").as_ptr());
                    let mut p = Point::default();
                    GetCursorPos(&mut p);
                    SetForegroundWindow(hwnd);
                    let choice = TrackPopupMenu(menu, 0x102, p.x, p.y, 0, hwnd, null());
                    DestroyMenu(menu);
                    MENU_OPEN = false;
                    if choice == 1 {
                        show(hwnd)
                    }
                    if choice == 2 {
                        quit(hwnd)
                    }
                }
                _ => {}
            }
            return 0;
        }
        0x114 => {
            if lp as H == SLIDER {
                let v = SendMessageW(SLIDER, 0x400, 0, 0);
                set_text(VALUE, &format!("{v}%"));
                PENDING = true;
                SetTimer(hwnd, 1, 100, null_mut());
                InvalidateRect(SLIDER, null(), 0);
            }
            return 0;
        }
        0x113 => {
            if wp == 1 {
                flush(hwnd)
            }
            return 0;
        }

        0x312 => {
            flush(hwnd);
            let result = selected_display().and_then(|d| {
                let cached = LAST_GOOD
                    .with(|s| s.borrow().clone())
                    .filter(|(path, _)| path == &d.path);
                let v = if cached.is_some() {
                    SendMessageW(SLIDER, 0x400, 0, 0) as u32
                } else {
                    brightness(&d)?
                };
                Ok(if wp == 1 {
                    v.saturating_add(5).min(100)
                } else {
                    v.saturating_sub(5)
                })
            });
            match result {
                Ok(v) => apply(v),
                Err(e) => update(Err(e)),
            }
            return 0;
        }
        0x7e => {
            KillTimer(hwnd, 1);
            PENDING = false;
            refresh();
            return 0;
        }
        0x2e0 => {
            let r = *(lp as *const Rect);
            SetWindowPos(
                hwnd,
                null_mut(),
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                0x14,
            );
            if !SLIDER.is_null() {
                layout(hwnd);
                fit_size(hwnd);
            }
            return 0;
        }
        _ => {
            if msg == TASKBAR && TASKBAR != 0 {
                notify(hwnd, 0);
                return 0;
            }
        }
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}
pub fn test_layouts() {
    assert_eq!(size_of::<DrawItem>(), 64);
    assert_eq!(size_of::<Paint>(), 72);
}
pub unsafe fn run(args: &[String]) {
    SetProcessDpiAwarenessContext(-4isize as H);
    let class = wide("TclBrightnessRustV1");
    let existing = FindWindowW(class.as_ptr(), null());
    if !existing.is_null() {
        PostMessageW(
            existing,
            if args.first().is_some_and(|s| s == "--quit") {
                QUIT
            } else {
                SHOW
            },
            0,
            0,
        );
        return;
    }
    if args.first().is_some_and(|s| s == "--quit") {
        return;
    }
    if let Ok(key) = std::fs::read_to_string(selected_file()) {
        if key.to_uppercase().contains("TCL2005") || key.to_uppercase().contains("GBT2709") {
            SELECTED.with(|s| *s.borrow_mut() = Some(key));
        }
    }
    let instance = GetModuleHandleW(null());
    ICON = sun_icon(instance);
    BRUSH = CreateSolidBrush(BG);
    TASKBAR = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());
    InitCommonControls();
    let wc = WndClass {
        style: 0x20000,
        proc: Some(wndproc),
        cls_extra: 0,
        win_extra: 0,
        instance,
        icon: ICON,
        cursor: LoadCursorW(null_mut(), 32512usize as *const u16),
        background: BRUSH,
        menu: null(),
        class: class.as_ptr(),
    };
    if RegisterClassW(&wc) == 0 {
        return;
    }
    let hwnd = CreateWindowExW(
        0x80,
        class.as_ptr(),
        wide("Brightness").as_ptr(),
        STYLE,
        0,
        0,
        440,
        360,
        null_mut(),
        null_mut(),
        instance,
        null_mut(),
    );
    if hwnd.is_null() {
        return;
    }
    MAIN = hwnd;
    dwm(hwnd, 20, 1);
    dwm(hwnd, 33, 2);
    dwm(hwnd, 35, BG);
    dwm(hwnd, 34, BORDER);
    let make = |class: &str, name: &str, style: u32, id: usize| {
        CreateWindowExW(
            0,
            wide(class).as_ptr(),
            wide(name).as_ptr(),
            0x50000000 | style,
            0,
            0,
            1,
            1,
            hwnd,
            id as H,
            instance,
            null_mut(),
        )
    };
    SELECTOR = make("BUTTON", "Choose display", 0x1000b, 101);
    MODE = make("STATIC", "Brightness", 0, 0);
    VALUE = make("STATIC", "—", 2, 0);
    SLIDER = make("msctls_trackbar32", "Brightness", 0x10050, 100);
    HELP = make("STATIC", "", 0, 0);
    RegisterHotKey(hwnd, 1, 0x4009, 0x21);
    RegisterHotKey(hwnd, 2, 0x4009, 0x22);
    ShowWindow(HELP, 0);
    SetWindowSubclass(SLIDER, Some(slider_proc), 1, 0);
    SendMessageW(SLIDER, 0x406, 1, 100 << 16);
    SendMessageW(SLIDER, 0x415, 0, 5);
    layout(hwnd);
    fit_size(hwnd);
    notify(hwnd, 0);
    if args.first().is_some_and(|s| s == "--show") {
        show(hwnd)
    }
    let mut msg: Msg = zeroed();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        if msg.msg == 0x100 && msg.wp == 0x1b {
            flush(hwnd);
            ShowWindow(hwnd, 0);
            continue;
        }
        if IsDialogMessageW(hwnd, &mut msg) == 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    for h in [FONT, SMALL, BIG, MEDIUM, BRUSH] {
        if !h.is_null() {
            DeleteObject(h);
        }
    }
    DestroyIcon(ICON);
}
