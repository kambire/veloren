//! Lanzador y actualizador automático de World of Azeria.
//!
//! Comprueba en GitHub si hay una versión nueva, la descarga e instala, y
//! arranca el juego. La interfaz se dibuja en `ui.rs` y la red está en `net.rs`.
//!
//! `veloren-updater --captura <archivo.png> [estado]` dibuja la interfaz en un
//! PNG sin abrir la ventana (estados: listo, comprobando, actualizar,
//! descargando, sin-conexion, error).

#![windows_subsystem = "windows"]

mod net;
mod ui;

use net::{AppState, NewsItem, UpdateStatus};
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};

/// Argumentos que se pasan tal cual al juego (sin los del propio lanzador)
fn game_args() -> Vec<String> { env::args().skip(1).collect() }

fn initial_state() -> AppState {
    let game_dir = net::get_game_dir();
    AppState {
        local_version: net::load_local_version(&game_dir).tag,
        remote_version: "Comprobando...".to_string(),
        status: UpdateStatus::Checking,
        game_binary: net::find_game_binary(&game_dir),
        news: None,
    }
}

/// Comprueba versiones y carga las novedades en segundo plano
fn spawn_background_checks(state: &net::Shared) {
    let check = Arc::clone(state);
    thread::spawn(move || net::check_github(&check));
    let news = Arc::clone(state);
    thread::spawn(move || net::fetch_news(&news));
}

/// Dibuja la interfaz en un PNG con un estado de ejemplo
fn capture(path: &str, state_name: &str) {
    let mut st = initial_state();
    st.local_version = "v0.18.0".to_string();
    st.remote_version = "v0.18.0".to_string();
    st.game_binary = Some("veloren-voxygen.exe".into());
    st.news = Some(vec![
        NewsItem {
            date: "25/09".into(),
            text: "Nuevo mundo: 4 continentes separados por océano y sin lagos".into(),
        },
        NewsItem {
            date: "24/09".into(),
            text: "Misiones y cadenas de misiones con marcadores ! y ?".into(),
        },
        NewsItem {
            date: "24/09".into(),
            text: "Los NPC de los pueblos ya no son hostiles".into(),
        },
        NewsItem {
            date: "23/09".into(),
            text: "Nueva clase: el Entrenador y sus mascotas".into(),
        },
    ]);
    st.status = match state_name {
        "comprobando" => {
            st.remote_version = "Comprobando...".to_string();
            UpdateStatus::Checking
        },
        "actualizar" => {
            st.remote_version = "v0.19.0".to_string();
            UpdateStatus::UpdateAvailable {
                remote_tag: "v0.19.0".to_string(),
                download_url: Some(String::new()),
                total_bytes: 812 * 1024 * 1024,
            }
        },
        "descargando" => UpdateStatus::Downloading {
            downloaded: 347 * 1024 * 1024,
            total: 812 * 1024 * 1024,
            speed: 9.4 * 1024.0 * 1024.0,
        },
        "sin-conexion" => {
            st.remote_version = "Sin conexión".to_string();
            UpdateStatus::Offline
        },
        "error" => UpdateStatus::Error("No se pudo escribir la actualización en disco".into()),
        _ => UpdateStatus::UpToDate,
    };

    let res = ui::Resources::load();
    let mut frame = tiny_skia::Pixmap::new(ui::WIDTH, ui::HEIGHT).expect("tamaño válido");
    ui::render(&res, &st, ui::Pointer::default(), 1.2, &mut frame);
    let _ = frame.save_png(path);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--captura") {
        let path = args.get(i + 1).map_or("lanzador.png", String::as_str);
        let state = args.get(i + 2).map_or("listo", String::as_str);
        capture(path, state);
        return;
    }

    let state = Arc::new(Mutex::new(initial_state()));
    spawn_background_checks(&state);

    #[cfg(windows)]
    win::run(state);

    #[cfg(not(windows))]
    {
        let game_dir = net::get_game_dir();
        if let Some(game) = net::find_game_binary(&game_dir) {
            net::launch_game(&game, &game_dir, &game_args());
        } else {
            eprintln!("No se encontró el ejecutable del juego.");
        }
    }
}

#[cfg(windows)]
mod win {
    use super::*;
    use crate::ui::{self, Hit, Pointer};
    use std::{path::PathBuf, process::Command, time::Instant};
    use tiny_skia::Pixmap;
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
        Graphics::{
            Dwm::DwmSetWindowAttribute,
            Gdi::{
                BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BeginPaint, DIB_RGB_COLORS, EndPaint,
                InvalidateRect, PAINTSTRUCT, ScreenToClient, SetDIBitsToDevice,
            },
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent},
            WindowsAndMessaging::{
                CS_DROPSHADOW, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
                GetMessageW, GetSystemMetrics, IDC_ARROW, IDC_HAND, LoadCursorW, LoadIconW, MSG,
                PostQuitMessage, RegisterClassExW, SM_CXSCREEN, SM_CYSCREEN, SW_MINIMIZE, SW_SHOW,
                SetCursor, SetProcessDPIAware, SetTimer, ShowWindow, TranslateMessage,
                WNDCLASSEXW, WS_EX_APPWINDOW, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU,
            },
        },
    };

    const WM_DESTROY: u32 = 0x0002;
    const WM_PAINT: u32 = 0x000F;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_SETCURSOR: u32 = 0x0020;
    const WM_NCHITTEST: u32 = 0x0084;
    const WM_TIMER: u32 = 0x0113;
    const WM_MOUSEMOVE: u32 = 0x0200;
    const WM_LBUTTONDOWN: u32 = 0x0201;
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_MOUSELEAVE: u32 = 0x02A3;
    const HTCLIENT: LRESULT = 1;
    const HTCAPTION: LRESULT = 2;
    /// Esquinas redondeadas de Windows 11 (`DWMWA_WINDOW_CORNER_PREFERENCE`)
    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: u32 = 2;

    struct Window {
        state: net::Shared,
        game_dir: PathBuf,
        res: ui::Resources,
        frame: Pixmap,
        bgra: Vec<u8>,
        pointer: Pointer,
        tracking_mouse: bool,
        start: Instant,
    }

    static WINDOW: Mutex<Option<Window>> = Mutex::new(None);

    fn to_wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

    fn lparam_point(lparam: LPARAM) -> (i32, i32) {
        (
            (lparam & 0xFFFF) as u16 as i16 as i32,
            ((lparam >> 16) & 0xFFFF) as u16 as i16 as i32,
        )
    }

    /// Acción que hay que ejecutar tras soltar el botón. Se hace fuera del
    /// cerrojo de la ventana porque algunas llamadas de Win32 envían mensajes
    /// que vuelven a entrar en `window_proc`.
    enum Action {
        Close,
        Minimize,
        LaunchAndClose,
        Update {
            url: String,
            tag: String,
        },
        CheckUpdates,
        OpenFolder,
        Website,
    }

    fn action_for(hit: Hit, window: &Window) -> Option<Action> {
        Some(match hit {
            Hit::Close => Action::Close,
            Hit::Minimize => Action::Minimize,
            Hit::CheckUpdates => Action::CheckUpdates,
            Hit::OpenFolder => Action::OpenFolder,
            Hit::Website => Action::Website,
            Hit::Play => {
                let st = window.state.lock().unwrap();
                match &st.status {
                    UpdateStatus::UpdateAvailable {
                        remote_tag,
                        download_url: Some(url),
                        ..
                    } => Action::Update {
                        url: url.clone(),
                        tag: remote_tag.clone(),
                    },
                    UpdateStatus::Downloading { .. } | UpdateStatus::Extracting => return None,
                    _ if st.game_binary.is_some() => Action::LaunchAndClose,
                    _ => return None,
                }
            },
        })
    }

    unsafe fn perform(hwnd: HWND, action: Action) {
        let (state, game_dir, game_binary) = {
            let guard = WINDOW.lock().unwrap();
            let Some(window) = guard.as_ref() else {
                return;
            };
            let binary = window.state.lock().unwrap().game_binary.clone();
            (Arc::clone(&window.state), window.game_dir.clone(), binary)
        };
        match action {
            Action::Close => unsafe {
                DestroyWindow(hwnd);
            },
            Action::Minimize => unsafe {
                ShowWindow(hwnd, SW_MINIMIZE);
            },
            Action::LaunchAndClose => {
                if let Some(binary) = game_binary {
                    net::launch_game(&binary, &game_dir, &game_args());
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                }
            },
            Action::Update { url, tag } => {
                thread::spawn(move || net::perform_update(&state, url, game_dir, tag));
            },
            Action::CheckUpdates => spawn_background_checks(&state),
            Action::OpenFolder => {
                let _ = Command::new("explorer").arg(&game_dir).spawn();
            },
            Action::Website => {
                let _ = Command::new("explorer")
                    .arg(format!("https://github.com/{}", net::GITHUB_REPO))
                    .spawn();
            },
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            // La barra de título propia arrastra la ventana, salvo sobre sus botones
            WM_NCHITTEST => {
                let (sx, sy) = lparam_point(lparam);
                let mut pt = POINT { x: sx, y: sy };
                unsafe { ScreenToClient(hwnd, &mut pt) };
                let (x, y) = (pt.x as f32, pt.y as f32);
                if y >= 0.0 && y < ui::TITLE_H && ui::hit_test(x, y).is_none() {
                    HTCAPTION
                } else {
                    HTCLIENT
                }
            },
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
                let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
                if let Ok(mut guard) = WINDOW.lock()
                    && let Some(window) = guard.as_mut()
                {
                    let time = window.start.elapsed().as_secs_f32();
                    {
                        let st = window.state.lock().unwrap();
                        ui::render(&window.res, &st, window.pointer, time, &mut window.frame);
                    }
                    // tiny-skia usa RGBA premultiplicado; la ventana es opaca, así
                    // que basta con pasar a BGRA
                    for (src, dst) in window
                        .frame
                        .data()
                        .chunks_exact(4)
                        .zip(window.bgra.chunks_exact_mut(4))
                    {
                        dst[0] = src[2];
                        dst[1] = src[1];
                        dst[2] = src[0];
                        dst[3] = 255;
                    }
                    let mut bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
                    bmi.bmiHeader = BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: ui::WIDTH as i32,
                        // Negativo: filas de arriba abajo
                        biHeight: -(ui::HEIGHT as i32),
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB,
                        ..unsafe { std::mem::zeroed() }
                    };
                    unsafe {
                        SetDIBitsToDevice(
                            hdc,
                            0,
                            0,
                            ui::WIDTH,
                            ui::HEIGHT,
                            0,
                            0,
                            0,
                            ui::HEIGHT,
                            window.bgra.as_ptr() as *const _,
                            &bmi,
                            DIB_RGB_COLORS,
                        );
                    }
                }
                unsafe { EndPaint(hwnd, &ps) };
                0
            },
            WM_TIMER => {
                unsafe { InvalidateRect(hwnd, std::ptr::null(), 0) };
                0
            },
            WM_SETCURSOR => {
                let over_button = WINDOW
                    .lock()
                    .ok()
                    .and_then(|guard| guard.as_ref().map(|w| w.pointer.hovered.is_some()))
                    .unwrap_or(false);
                let cursor = if over_button { IDC_HAND } else { IDC_ARROW };
                unsafe { SetCursor(LoadCursorW(std::ptr::null_mut(), cursor)) };
                1
            },
            WM_MOUSEMOVE => {
                let (x, y) = lparam_point(lparam);
                if let Ok(mut guard) = WINDOW.lock()
                    && let Some(window) = guard.as_mut()
                {
                    window.pointer.hovered = ui::hit_test(x as f32, y as f32);
                    if !window.tracking_mouse {
                        let mut tme = TRACKMOUSEEVENT {
                            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                            dwFlags: TME_LEAVE,
                            hwndTrack: hwnd,
                            dwHoverTime: 0,
                        };
                        unsafe { TrackMouseEvent(&mut tme) };
                        window.tracking_mouse = true;
                    }
                }
                0
            },
            WM_MOUSELEAVE => {
                if let Ok(mut guard) = WINDOW.lock()
                    && let Some(window) = guard.as_mut()
                {
                    window.pointer = Pointer::default();
                    window.tracking_mouse = false;
                }
                0
            },
            WM_LBUTTONDOWN => {
                if let Ok(mut guard) = WINDOW.lock()
                    && let Some(window) = guard.as_mut()
                {
                    window.pointer.pressed = window.pointer.hovered;
                }
                0
            },
            WM_LBUTTONUP => {
                let action = WINDOW.lock().ok().and_then(|mut guard| {
                    let window = guard.as_mut()?;
                    let clicked = window
                        .pointer
                        .pressed
                        .filter(|hit| window.pointer.hovered == Some(*hit));
                    window.pointer.pressed = None;
                    action_for(clicked?, window)
                });
                if let Some(action) = action {
                    unsafe { perform(hwnd, action) };
                }
                0
            },
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                0
            },
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }

    pub fn run(state: net::Shared) {
        *WINDOW.lock().unwrap() = Some(Window {
            state,
            game_dir: net::get_game_dir(),
            res: ui::Resources::load(),
            frame: Pixmap::new(ui::WIDTH, ui::HEIGHT).expect("tamaño válido"),
            bgra: vec![0; (ui::WIDTH * ui::HEIGHT * 4) as usize],
            pointer: Pointer::default(),
            tracking_mouse: false,
            start: Instant::now(),
        });

        unsafe {
            // Sin escalado borroso de Windows en pantallas con zoom
            SetProcessDPIAware();

            let hinstance = GetModuleHandleW(std::ptr::null());
            let class_name = to_wide("WorldOfAzeriaLauncher");
            // Icono incrustado por build.rs con el identificador 1
            let icon = LoadIconW(hinstance, 1 as *const u16);

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_DROPSHADOW,
                lpfnWndProc: Some(window_proc),
                hInstance: hinstance,
                hIcon: icon,
                hIconSm: icon,
                hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            };
            RegisterClassExW(&wc);

            let (w, h) = (ui::WIDTH as i32, ui::HEIGHT as i32);
            let x = (GetSystemMetrics(SM_CXSCREEN) - w) / 2;
            let y = (GetSystemMetrics(SM_CYSCREEN) - h) / 2;
            let title = to_wide("World of Azeria");
            let hwnd = CreateWindowExW(
                WS_EX_APPWINDOW,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_POPUP | WS_MINIMIZEBOX | WS_SYSMENU,
                x,
                y,
                w,
                h,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null(),
            );
            if hwnd.is_null() {
                return;
            }

            let corners = DWMWCP_ROUND;
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &corners as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            );

            ShowWindow(hwnd, SW_SHOW);
            // ~30 fotogramas por segundo para las animaciones
            SetTimer(hwnd, 1, 33, None);

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
