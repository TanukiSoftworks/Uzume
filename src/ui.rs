#![allow(static_mut_refs, unused_must_use, dead_code)]

use crate::config::{Config, StatusDisplayMode};
use crate::i18n::{get_strings, Strings};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{DRAWITEMSTRUCT, ODS_SELECTED, ODT_BUTTON};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const WM_TRAY_CALLBACK: u32 = WM_APP + 1;

const ID_TRAY_SETTINGS: usize = 1001;
const ID_TRAY_EXIT: usize = 1002;
const ID_TRAY_AI_NOTICE: usize = 1003;

const ID_EDIT_CUSTOM_TEXT: usize = 2004;
const ID_BTN_SAVE: usize = 2010;

pub static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

const COLOR_BG: COLORREF = rgb(22, 23, 26);
const COLOR_CARD: COLORREF = rgb(29, 30, 34);
const COLOR_CARD_ACTIVE: COLORREF = rgb(35, 32, 34);
const COLOR_BORDER: COLORREF = rgb(40, 41, 48);
const COLOR_BORDER_INPUT: COLORREF = rgb(53, 54, 63);
const COLOR_TEXT: COLORREF = rgb(226, 226, 224);
const COLOR_MUTED: COLORREF = rgb(141, 142, 150);
const COLOR_MUTED_DARK: COLORREF = rgb(110, 112, 120);
const COLOR_ACCENT: COLORREF = rgb(184, 67, 55);
const COLOR_ACCENT_HOVER: COLORREF = rgb(163, 56, 45);
const COLOR_STATUS_PLAYING: COLORREF = rgb(201, 169, 110);
const COLOR_BADGE_BG: COLORREF = rgb(37, 34, 36);
const COLOR_BADGE_BORDER: COLORREF = rgb(90, 35, 30);
const COLOR_BADGE_TEXT: COLORREF = rgb(196, 83, 71);

const ICON_PNG_BYTES: &[u8] = include_bytes!("../assets/icon.png");

#[repr(C)]
struct GdiplusStartupInput {
    gdiplus_version: u32,
    debug_event_callback: usize,
    suppress_background_thread: i32,
    suppress_external_codecs: i32,
}

#[link(name = "gdiplus")]
extern "system" {
    fn GdiplusStartup(token: *mut usize, input: *const GdiplusStartupInput, output: *mut usize) -> i32;
    fn GdipCreateBitmapFromFile(filename: *const u16, bitmap: *mut usize) -> i32;
    fn GdipDisposeImage(image: usize) -> i32;
    fn GdipCreateFromHDC(hdc: HDC, graphics: *mut usize) -> i32;
    fn GdipDeleteGraphics(graphics: usize) -> i32;
    fn GdipDrawImageRectI(graphics: usize, image: usize, x: i32, y: i32, width: i32, height: i32) -> i32;
}

struct UiState {
    config: Arc<Mutex<Config>>,
    current_status: Arc<Mutex<String>>,
    current_mode: StatusDisplayMode,
    strings: &'static Strings,
    hwnd_edit_custom: HWND,
    hwnd_btn_save: HWND,
    detected_app: String,
    state_desc: String,
    track_title: String,
    artist_name: String,
    status_preview: String,
    has_cover: bool,
    app_icon: HICON,
    font_title: HFONT,
    font_bold: HFONT,
    font_semibold: HFONT,
    font_regular: HFONT,
    font_small: HFONT,
    font_badge: HFONT,
}

unsafe impl Send for UiState {}

static UI_STATE: Mutex<Option<UiState>> = Mutex::new(None);
static mut G_BG_BRUSH: HBRUSH = HBRUSH(std::ptr::null_mut());
static mut G_EDIT_BRUSH: HBRUSH = HBRUSH(std::ptr::null_mut());

unsafe fn create_segoe_font(height: i32, weight: i32) -> HFONT {
    CreateFontW(
        height,
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
        w!("Segoe UI"),
    )
}

unsafe fn draw_round_card(hdc: HDC, x: i32, y: i32, w: i32, h: i32, bg_col: COLORREF, border_col: COLORREF, radius: i32) {
    let brush = CreateSolidBrush(bg_col);
    let pen = CreatePen(PS_SOLID, 1, border_col);
    let old_brush = SelectObject(hdc, brush);
    let old_pen = SelectObject(hdc, pen);

    let _ = RoundRect(hdc, x, y, x + w, y + h, radius, radius);

    SelectObject(hdc, old_brush);
    SelectObject(hdc, old_pen);
    DeleteObject(brush);
    DeleteObject(pen);
}

unsafe fn draw_radio_circle(hdc: HDC, cx: i32, cy: i32, selected: bool) {
    let pen_col = if selected { COLOR_ACCENT } else { COLOR_BORDER };
    let pen = CreatePen(PS_SOLID, 1, pen_col);
    let brush = CreateSolidBrush(if selected { COLOR_CARD_ACTIVE } else { COLOR_CARD });
    let old_pen = SelectObject(hdc, pen);
    let old_brush = SelectObject(hdc, brush);

    let r = 8;
    let _ = Ellipse(hdc, cx - r, cy - r, cx + r, cy + r);

    if selected {
        let dot_brush = CreateSolidBrush(COLOR_ACCENT);
        let dot_pen = CreatePen(PS_SOLID, 1, COLOR_ACCENT);
        let ob = SelectObject(hdc, dot_brush);
        let op = SelectObject(hdc, dot_pen);
        let dr = 4;
        let _ = Ellipse(hdc, cx - dr, cy - dr, cx + dr, cy + dr);
        SelectObject(hdc, ob);
        SelectObject(hdc, op);
        DeleteObject(dot_brush);
        DeleteObject(dot_pen);
    }

    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
    DeleteObject(pen);
    DeleteObject(brush);
}

unsafe fn render_text(hdc: HDC, font: HFONT, color: COLORREF, text: &str, x: i32, y: i32, w: i32, h: i32) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let old_font = SelectObject(hdc, font);
    SetTextColor(hdc, color);
    SetBkMode(hdc, TRANSPARENT);
    let mut rect = RECT {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    };
    DrawTextW(hdc, &mut wide.clone(), &mut rect, DT_LEFT | DT_TOP | DT_END_ELLIPSIS | DT_NOPREFIX);
    SelectObject(hdc, old_font);
}

pub fn run_ui(config: Arc<Mutex<Config>>, current_status: Arc<Mutex<String>>) {
    let mut gdiplus_token: usize = 0;
    let input = GdiplusStartupInput {
        gdiplus_version: 1,
        debug_event_callback: 0,
        suppress_background_thread: 0,
        suppress_external_codecs: 0,
    };
    let _ = unsafe { GdiplusStartup(&mut gdiplus_token, &input, std::ptr::null_mut()) };

    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None).unwrap_or_default().into();
        let class_name = w!("UzumeMainWindowClass");

        let bg_brush = CreateSolidBrush(COLOR_BG);
        let edit_brush = CreateSolidBrush(COLOR_BG);
        G_BG_BRUSH = bg_brush;
        G_EDIT_BRUSH = edit_brush;

        let font_title = create_segoe_font(-24, 700);
        let font_bold = create_segoe_font(-14, 700);
        let font_semibold = create_segoe_font(-13, 600);
        let font_regular = create_segoe_font(-12, 400);
        let font_small = create_segoe_font(-11, 400);
        let font_badge = create_segoe_font(-11, 600);

        let app_icon = CreateIconFromResourceEx(
            ICON_PNG_BYTES,
            true,
            0x00030000,
            32,
            32,
            LR_DEFAULTCOLOR,
        ).unwrap_or_default();

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hIcon: app_icon,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: bg_brush,
            lpszClassName: class_name,
            ..Default::default()
        };

        let _ = RegisterClassW(&wc);

        let mut wr = RECT {
            left: 0,
            top: 0,
            right: 524,
            bottom: 466,
        };
        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN;
        let _ = AdjustWindowRectEx(&mut wr, style, false, WINDOW_EX_STYLE::default());

        let win_w = wr.right - wr.left;
        let win_h = wr.bottom - wr.top;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let win_x = (screen_w - win_w) / 2;
        let win_y = (screen_h - win_h) / 2;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Uzume"),
            style | WS_VISIBLE,
            win_x,
            win_y,
            win_w,
            win_h,
            None,
            None,
            instance,
            None,
        ).unwrap_or_default();

        let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_BIG as usize), LPARAM(app_icon.0 as isize));
        let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_SMALL as usize), LPARAM(app_icon.0 as isize));

        let dark_mode: i32 = 1;
        let caption_color = 0x001A1716u32;
        let text_color = 0x00E0E2E2u32;
        let border_color = 0x00332C2Bu32;
        let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(20), &dark_mode as *const _ as *const _, 4);
        let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(35), &caption_color as *const _ as *const _, 4);
        let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(36), &text_color as *const _ as *const _, 4);
        let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(34), &border_color as *const _ as *const _, 4);

        let initial_cfg = config.lock().unwrap().clone();
        let strings = get_strings(initial_cfg.language);

        let hwnd_edit_custom = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("EDIT"),
            w!(""),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            74, 200, 400, 18,
            hwnd,
            HMENU(ID_EDIT_CUSTOM_TEXT as isize as *mut _),
            instance,
            None,
        ).unwrap_or_default();
        SendMessageW(hwnd_edit_custom, WM_SETFONT, WPARAM(font_regular.0 as usize), LPARAM(1));

        const EM_SETCUEBANNER: u32 = 0x1501;
        let wide_placeholder: Vec<u16> = strings.opt_custom_placeholder.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = SendMessageW(hwnd_edit_custom, EM_SETCUEBANNER, WPARAM(1), LPARAM(wide_placeholder.as_ptr() as isize));

        let wide_custom: Vec<u16> = initial_cfg.custom_text.encode_utf16().chain(std::iter::once(0)).collect();
        SetWindowTextW(hwnd_edit_custom, PCWSTR(wide_custom.as_ptr()));

        let hwnd_btn_save = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!(""),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_OWNERDRAW as u32),
            24, 408, 476, 38,
            hwnd,
            HMENU(ID_BTN_SAVE as isize as *mut _),
            instance,
            None,
        ).unwrap_or_default();

        let state = UiState {
            config: config.clone(),
            current_status: current_status.clone(),
            current_mode: initial_cfg.status_mode,
            strings,
            hwnd_edit_custom,
            hwnd_btn_save,
            detected_app: "Uzume".to_string(),
            state_desc: strings.state_inactive.to_string(),
            track_title: strings.no_music_active.to_string(),
            artist_name: String::new(),
            status_preview: "Listening to Uzume".to_string(),
            has_cover: false,
            app_icon,
            font_title,
            font_bold,
            font_semibold,
            font_regular,
            font_small,
            font_badge,
        };

        if let Ok(mut state_guard) = UI_STATE.lock() {
            *state_guard = Some(state);
        }

        add_tray_icon(hwnd, app_icon);
        SetTimer(hwnd, 1, 1000, None);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            if IsDialogMessageW(hwnd, &msg).as_bool() {
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        remove_tray_icon(hwnd);
        GdipDisposeImage(app_icon.0 as usize);
        DeleteObject(bg_brush);
        DeleteObject(edit_brush);
        DeleteObject(font_title);
        DeleteObject(font_bold);
        DeleteObject(font_semibold);
        DeleteObject(font_regular);
        DeleteObject(font_small);
        DeleteObject(font_badge);
    }
}

unsafe fn add_tray_icon(hwnd: HWND, icon: HICON) {
    let mut tip = [0u16; 128];
    let tip_text: Vec<u16> = "Uzume".encode_utf16().collect();
    for (i, &c) in tip_text.iter().enumerate().take(127) {
        tip[i] = c;
    }

    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY_CALLBACK,
        hIcon: icon,
        szTip: tip,
        ..Default::default()
    };
    let _ = Shell_NotifyIconW(NIM_ADD, &mut nid);
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        ..Default::default()
    };
    let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut client_rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut client_rect);
            let w = client_rect.right;
            let h = client_rect.bottom;

            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, w, h);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            FillRect(mem_dc, &client_rect, G_BG_BRUSH);

            if let Ok(state_guard) = UI_STATE.try_lock() {
                if let Some(state) = state_guard.as_ref() {
                    let s = state.strings;

                    DrawIconEx(mem_dc, 24, 18, state.app_icon, 28, 28, 0, None, DI_NORMAL);
                    render_text(mem_dc, state.font_title, COLOR_TEXT, "Uzume", 60, 19, 400, 30);
                    render_text(mem_dc, state.font_small, COLOR_MUTED, &s.section_format, 24, 60, 476, 18);

                    let is_opt1 = state.current_mode == StatusDisplayMode::ListeningSong;
                    draw_round_card(
                        mem_dc, 24, 84, 476, 52,
                        if is_opt1 { COLOR_CARD_ACTIVE } else { COLOR_CARD },
                        if is_opt1 { COLOR_ACCENT } else { COLOR_BORDER },
                        8,
                    );
                    draw_radio_circle(mem_dc, 38, 110, is_opt1);
                    render_text(mem_dc, state.font_semibold, COLOR_TEXT, &s.opt_song_title, 64, 91, 420, 20);
                    render_text(mem_dc, state.font_small, COLOR_MUTED, &s.opt_song_desc, 64, 111, 420, 18);

                    let is_opt2 = state.current_mode == StatusDisplayMode::CustomText;
                    draw_round_card(
                        mem_dc, 24, 142, 476, 92,
                        if is_opt2 { COLOR_CARD_ACTIVE } else { COLOR_CARD },
                        if is_opt2 { COLOR_ACCENT } else { COLOR_BORDER },
                        8,
                    );
                    draw_radio_circle(mem_dc, 38, 168, is_opt2);
                    render_text(mem_dc, state.font_semibold, COLOR_TEXT, &s.opt_custom_title, 64, 149, 420, 20);
                    render_text(mem_dc, state.font_small, COLOR_MUTED, &s.opt_custom_desc, 64, 169, 420, 18);

                    let input_border = if is_opt2 { rgb(75, 45, 45) } else { COLOR_BORDER_INPUT };
                    draw_round_card(mem_dc, 64, 194, 420, 30, COLOR_BG, input_border, 6);

                    let is_opt3 = state.current_mode == StatusDisplayMode::AppDefault;
                    let opt3_title = s.opt_app_title_fmt.replace("{}", &state.detected_app);
                    draw_round_card(
                        mem_dc, 24, 240, 476, 52,
                        if is_opt3 { COLOR_CARD_ACTIVE } else { COLOR_CARD },
                        if is_opt3 { COLOR_ACCENT } else { COLOR_BORDER },
                        8,
                    );
                    draw_radio_circle(mem_dc, 38, 266, is_opt3);
                    render_text(mem_dc, state.font_semibold, COLOR_TEXT, &opt3_title, 64, 247, 420, 20);
                    render_text(mem_dc, state.font_small, COLOR_MUTED, &s.opt_app_desc, 64, 267, 420, 18);

                    draw_round_card(mem_dc, 24, 298, 476, 96, COLOR_CARD, COLOR_BORDER, 8);
                    render_text(mem_dc, state.font_small, COLOR_MUTED, &s.active_player_header, 38, 306, 200, 18);

                    let is_playing = state.state_desc == s.state_playing || state.state_desc.contains("Playing") || state.state_desc.contains("reproducción") || state.state_desc.contains("播放");
                    let status_dot_color = if is_playing { COLOR_STATUS_PLAYING } else { COLOR_MUTED };
                    let dot_brush = CreateSolidBrush(status_dot_color);
                    let dot_pen = CreatePen(PS_SOLID, 1, status_dot_color);
                    let ob = SelectObject(mem_dc, dot_brush);
                    let op = SelectObject(mem_dc, dot_pen);
                    let _ = Ellipse(mem_dc, 342, 311, 349, 318);
                    SelectObject(mem_dc, ob);
                    SelectObject(mem_dc, op);
                    DeleteObject(dot_brush);
                    DeleteObject(dot_pen);

                    let status_pill_text = format!("{} • {}", state.detected_app, state.state_desc);
                    render_text(mem_dc, state.font_small, status_dot_color, &status_pill_text, 354, 306, 136, 18);

                    let appdata = std::env::var_os("APPDATA").unwrap_or_default();
                    let cover_path = std::path::PathBuf::from(appdata).join("Uzume").join("cover.jpg");
                    let mut cover_drawn = false;

                    if cover_path.exists() {
                        let path_str: Vec<u16> = cover_path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
                        let mut bitmap: usize = 0;
                        if GdipCreateBitmapFromFile(path_str.as_ptr(), &mut bitmap) == 0 && bitmap != 0 {
                            let mut graphics: usize = 0;
                            if GdipCreateFromHDC(mem_dc, &mut graphics) == 0 && graphics != 0 {
                                GdipDrawImageRectI(graphics, bitmap, 38, 328, 54, 54);
                                GdipDeleteGraphics(graphics);
                                cover_drawn = true;
                            }
                            GdipDisposeImage(bitmap);
                        }
                    }

                    if !cover_drawn {
                        draw_round_card(mem_dc, 38, 328, 54, 54, COLOR_BG, COLOR_BORDER, 6);
                        render_text(mem_dc, state.font_bold, COLOR_ACCENT, "♪", 59, 344, 20, 20);
                    }

                    let cov_pen = CreatePen(PS_SOLID, 1, COLOR_BORDER);
                    let o_pen = SelectObject(mem_dc, cov_pen);
                    let o_br = SelectObject(mem_dc, GetStockObject(HOLLOW_BRUSH));
                    let _ = RoundRect(mem_dc, 38, 328, 92, 382, 6, 6);
                    SelectObject(mem_dc, o_pen);
                    SelectObject(mem_dc, o_br);
                    DeleteObject(cov_pen);

                    render_text(mem_dc, state.font_bold, COLOR_TEXT, &state.track_title, 102, 330, 386, 20);
                    let artist_display = if !state.artist_name.is_empty() { state.artist_name.as_str() } else { "Uzume Music Presence" };
                    render_text(mem_dc, state.font_regular, COLOR_MUTED, artist_display, 102, 348, 386, 18);

                    render_text(mem_dc, state.font_small, COLOR_MUTED_DARK, &s.in_profile_label, 102, 368, 80, 18);

                    let badge_text = format!("♪ {}", state.status_preview);
                    let wide_badge: Vec<u16> = badge_text.encode_utf16().collect();
                    let mut text_size = windows::Win32::Foundation::SIZE::default();
                    SelectObject(mem_dc, state.font_badge);
                    let _ = GetTextExtentPoint32W(mem_dc, &wide_badge, &mut text_size);
                    let pill_w = (text_size.cx + 16).min(310);

                    draw_round_card(mem_dc, 182, 366, pill_w, 20, COLOR_BADGE_BG, COLOR_BADGE_BORDER, 4);
                    render_text(mem_dc, state.font_badge, COLOR_BADGE_TEXT, &badge_text, 190, 368, pill_w - 14, 18);
                }
            }

            let _ = BitBlt(hdc, 0, 0, w, h, mem_dc, 0, 0, SRCCOPY);

            SelectObject(mem_dc, old_bmp);
            DeleteObject(mem_bmp);
            DeleteDC(mem_dc);
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xffff) as i32;
            let y = ((lparam.0 >> 16) & 0xffff) as i32;

            if x >= 24 && x <= 500 {
                if y >= 84 && y <= 136 {
                    if let Ok(mut state_guard) = UI_STATE.lock() {
                        if let Some(state) = state_guard.as_mut() {
                            state.current_mode = StatusDisplayMode::ListeningSong;
                        }
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                } else if y >= 142 && y <= 234 {
                    let mut edit_hwnd = None;
                    if let Ok(mut state_guard) = UI_STATE.lock() {
                        if let Some(state) = state_guard.as_mut() {
                            state.current_mode = StatusDisplayMode::CustomText;
                            edit_hwnd = Some(state.hwnd_edit_custom);
                        }
                    }
                    if let Some(eh) = edit_hwnd {
                        let _ = SetFocus(eh);
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                } else if y >= 240 && y <= 292 {
                    if let Ok(mut state_guard) = UI_STATE.lock() {
                        if let Some(state) = state_guard.as_mut() {
                            state.current_mode = StatusDisplayMode::AppDefault;
                        }
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
            }
            LRESULT(0)
        }
        WM_SETCURSOR => {
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            let _ = ScreenToClient(hwnd, &mut pt);

            if pt.x >= 24 && pt.x <= 500 {
                let over_card1 = pt.y >= 84 && pt.y <= 136;
                let over_card2 = pt.y >= 142 && pt.y <= 190;
                let over_card3 = pt.y >= 240 && pt.y <= 292;
                if over_card1 || over_card2 || over_card3 {
                    SetCursor(LoadCursorW(None, IDC_HAND).unwrap_or_default());
                    return LRESULT(1);
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_DRAWITEM => {
            let dis = &*(lparam.0 as *const DRAWITEMSTRUCT);
            if dis.CtlType == ODT_BUTTON && dis.CtlID as usize == ID_BTN_SAVE {
                let is_pressed = (dis.itemState.0 & ODS_SELECTED.0) != 0;
                let bg_col = if is_pressed { COLOR_ACCENT_HOVER } else { COLOR_ACCENT };

                let brush = CreateSolidBrush(bg_col);
                let pen = CreatePen(PS_SOLID, 1, bg_col);
                let ob = SelectObject(dis.hDC, brush);
                let op = SelectObject(dis.hDC, pen);
                let _ = RoundRect(dis.hDC, dis.rcItem.left, dis.rcItem.top, dis.rcItem.right, dis.rcItem.bottom, 6, 6);
                SelectObject(dis.hDC, ob);
                SelectObject(dis.hDC, op);
                DeleteObject(brush);
                DeleteObject(pen);

                SetBkMode(dis.hDC, TRANSPARENT);
                SetTextColor(dis.hDC, COLOR_TEXT);

                let btn_text = if let Ok(state_guard) = UI_STATE.try_lock() {
                    if let Some(s) = state_guard.as_ref() {
                        s.strings.btn_save.as_str()
                    } else {
                        "Guardar Cambios"
                    }
                } else {
                    "Guardar Cambios"
                };

                let mut wide: Vec<u16> = btn_text.encode_utf16().chain(std::iter::once(0)).collect();
                let mut rect = dis.rcItem;
                let old_font = if let Ok(state_guard) = UI_STATE.try_lock() {
                    if let Some(s) = state_guard.as_ref() {
                        SelectObject(dis.hDC, s.font_semibold)
                    } else {
                        HGDIOBJ(std::ptr::null_mut())
                    }
                } else {
                    HGDIOBJ(std::ptr::null_mut())
                };

                DrawTextW(dis.hDC, &mut wide, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                if !old_font.0.is_null() {
                    SelectObject(dis.hDC, old_font);
                }
                return LRESULT(1);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_CTLCOLOREDIT => {
            let hdc = HDC(wparam.0 as *mut _);
            SetTextColor(hdc, COLOR_TEXT);
            SetBkColor(hdc, COLOR_BG);
            LRESULT(G_EDIT_BRUSH.0 as isize)
        }
        WM_TIMER => {
            let status_arc = if let Ok(state_guard) = UI_STATE.try_lock() {
                state_guard.as_ref().map(|s| s.current_status.clone())
            } else {
                None
            };

            if let Some(arc) = status_arc {
                if let Ok(st_guard) = arc.try_lock() {
                    let text = st_guard.clone();
                    let parts: Vec<String> = text.split('|').map(|s| s.to_string()).collect();
                    if parts.len() >= 5 {
                        if let Ok(mut state_guard) = UI_STATE.lock() {
                            if let Some(s) = state_guard.as_mut() {
                                let mut changed = false;
                                if s.detected_app != parts[0] { s.detected_app = parts[0].clone(); changed = true; }
                                if s.state_desc != parts[1] { s.state_desc = parts[1].clone(); changed = true; }
                                if s.track_title != parts[2] { s.track_title = parts[2].clone(); changed = true; }
                                if s.artist_name != parts[3] { s.artist_name = parts[3].clone(); changed = true; }
                                if s.status_preview != parts[4] { s.status_preview = parts[4].clone(); changed = true; }
                                if changed {
                                    let _ = InvalidateRect(hwnd, None, false);
                                }
                            }
                        }
                    }
                }
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xffff) as usize;
            match id {
                ID_TRAY_SETTINGS => {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                }
                ID_TRAY_EXIT => {
                    SHOULD_EXIT.store(true, Ordering::SeqCst);
                    PostQuitMessage(0);
                }
                ID_TRAY_AI_NOTICE => {
                    let notice_data: Option<(&'static str, &'static str)> = {
                        if let Ok(state_guard) = UI_STATE.lock() {
                            if let Some(state) = state_guard.as_ref() {
                                if !state.strings.ai_translation_notice.is_empty() {
                                    Some((
                                        state.strings.tray_ai_notice.as_str(),
                                        state.strings.ai_translation_notice.as_str(),
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some((title, msg)) = notice_data {
                        let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
                        let wide_msg: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
                        let _ = MessageBoxW(hwnd, PCWSTR(wide_msg.as_ptr()), PCWSTR(wide_title.as_ptr()), MB_OK | MB_ICONINFORMATION);
                    }
                }
                ID_BTN_SAVE => {
                    let save_result: Option<(StatusDisplayMode, String, &'static Strings)> = {
                        if let Ok(state_guard) = UI_STATE.lock() {
                            if let Some(state) = state_guard.as_ref() {
                                let mut buf = [0u16; 128];
                                let len = GetWindowTextW(state.hwnd_edit_custom, &mut buf) as usize;
                                let custom_text = String::from_utf16_lossy(&buf[..len]);
                                Some((state.current_mode, custom_text, state.strings))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some((mode, custom_text, strings)) = save_result {
                        if let Ok(state_guard) = UI_STATE.lock() {
                            if let Some(state) = state_guard.as_ref() {
                                let mut cfg = state.config.lock().unwrap();
                                cfg.status_mode = mode;
                                cfg.custom_text = custom_text.clone();
                                cfg.save();
                            }
                        }

                        let desc = match mode {
                            StatusDisplayMode::ListeningSong => strings.opt_song_title.to_string(),
                            StatusDisplayMode::CustomText => format!("{}: '{}'", strings.opt_custom_title, custom_text),
                            StatusDisplayMode::AppDefault => strings.opt_app_title_fmt.replace("{}", "App"),
                        };

                        let msg_str = format!("{}\n{}", strings.save_success_body, desc);
                        let wide_info: Vec<u16> = msg_str.encode_utf16().chain(std::iter::once(0)).collect();
                        let wide_title: Vec<u16> = strings.save_success_title.encode_utf16().chain(std::iter::once(0)).collect();
                        let _ = MessageBoxW(hwnd, PCWSTR(wide_info.as_ptr()), PCWSTR(wide_title.as_ptr()), MB_OK | MB_ICONINFORMATION);
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }
        WM_TRAY_CALLBACK => {
            let event = (lparam.0 & 0xFFFF) as u32;
            if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = SetForegroundWindow(hwnd);
            } else if event == WM_RBUTTONUP {
                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);

                let (tray_open, tray_exit, tray_ai_notice) = if let Ok(state_guard) = UI_STATE.try_lock() {
                    if let Some(s) = state_guard.as_ref() {
                        let ai_notice = if !s.strings.ai_translation_notice.is_empty() {
                            Some(s.strings.tray_ai_notice.as_str())
                        } else {
                            None
                        };
                        (s.strings.tray_open.as_str(), s.strings.tray_exit.as_str(), ai_notice)
                    } else {
                        ("Abrir Uzume", "Salir", None)
                    }
                } else {
                    ("Abrir Uzume", "Salir", None)
                };

                let wide_open: Vec<u16> = tray_open.encode_utf16().chain(std::iter::once(0)).collect();
                let wide_exit: Vec<u16> = tray_exit.encode_utf16().chain(std::iter::once(0)).collect();

                let hmenu = CreatePopupMenu().unwrap_or_default();
                let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_SETTINGS, PCWSTR(wide_open.as_ptr()));

                if let Some(ai_txt) = tray_ai_notice {
                    let wide_ai: Vec<u16> = ai_txt.encode_utf16().chain(std::iter::once(0)).collect();
                    let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_AI_NOTICE, PCWSTR(wide_ai.as_ptr()));
                }
                let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT, PCWSTR(wide_exit.as_ptr()));

                let _ = SetForegroundWindow(hwnd);
                let _ = TrackPopupMenu(
                    hmenu,
                    TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
                    pt.x,
                    pt.y,
                    0,
                    hwnd,
                    None,
                );
                let _ = DestroyMenu(hmenu);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            SHOULD_EXIT.store(true, Ordering::SeqCst);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
