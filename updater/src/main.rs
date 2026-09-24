#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::Gdi::{
        BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW,
        CreatePen, CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint,
        FillRect, InvalidateRect, RoundRect, SelectObject, SetBkMode, SetTextColor,
        DT_CENTER, DT_LEFT, DT_SINGLELINE, DT_VCENTER,
        PAINTSTRUCT, PS_SOLID, SRCCOPY, TRANSPARENT,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::Input::KeyboardAndMouse::{TrackMouseEvent, TRACKMOUSEEVENT, TME_LEAVE},
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
        GetSystemMetrics, LoadCursorW, PostQuitMessage, RegisterClassExW,
        SetTimer, SetWindowPos, ShowWindow, TranslateMessage,
        CS_HREDRAW, CS_VREDRAW, HWND_TOP, IDC_ARROW,
        SM_CXSCREEN, SM_CYSCREEN, SWP_SHOWWINDOW, SW_SHOW,
        WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
        WM_PAINT, WM_TIMER, WNDCLASSEXW, WS_CAPTION, WS_MINIMIZEBOX, WS_OVERLAPPED,
        WS_SYSMENU, WS_VISIBLE,
    },
};

#[cfg(windows)]
const WM_MOUSELEAVE: u32 = 0x02A3;
#[cfg(windows)]
const DT_RIGHT: u32 = 2;

const GITHUB_REPO: &str = "kambire/veloren";
const VERSION_FILE: &str = "version.json";
const GAME_EXECUTABLE: &str = if cfg!(target_os = "windows") {
    "veloren-voxygen.exe"
} else {
    "veloren-voxygen"
};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct LocalVersion {
    version: String,
    tag: String,
    updated_at: String,
}

#[derive(Deserialize, Debug, Clone)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize, Debug, Clone)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct GitHubCommit {
    sha: String,
}

#[derive(Clone, Debug, PartialEq)]
enum UpdateStatus {
    Checking,
    UpToDate,
    UpdateAvailable {
        remote_tag: String,
        download_url: Option<String>,
        total_bytes: u64,
    },
    Downloading {
        downloaded_bytes: u64,
        total_bytes: u64,
        pct: f32,
    },
    Extracting,
    Error(String),
    Offline(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ButtonId {
    PlayOrUpdate,
    CheckUpdates,
}

struct AppState {
    local_version: String,
    remote_version: String,
    status: UpdateStatus,
    game_binary: Option<PathBuf>,
    hovered_button: Option<ButtonId>,
    pressed_button: Option<ButtonId>,
    animation_tick: u32,
}

fn get_game_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn load_local_version(dir: &Path) -> LocalVersion {
    let path = dir.join(VERSION_FILE);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<LocalVersion>(&content) {
                return v;
            }
        }
    }
    LocalVersion {
        version: "0.18.0".to_string(),
        tag: "v0.18.0".to_string(),
        updated_at: String::new(),
    }
}

fn save_local_version(dir: &Path, version: &LocalVersion) {
    let path = dir.join(VERSION_FILE);
    if let Ok(content) = serde_json::to_string_pretty(version) {
        let _ = fs::write(path, content);
    }
}

fn find_game_binary(dir: &Path) -> Option<PathBuf> {
    // 1. En el mismo directorio del launcher
    let direct = dir.join(GAME_EXECUTABLE);
    if direct.exists() {
        return Some(direct);
    }
    // 2. En target/release
    let release_bin = dir.join("target").join("release").join(GAME_EXECUTABLE);
    if release_bin.exists() {
        return Some(release_bin);
    }
    // 3. En target/debug
    let debug_bin = dir.join("target").join("debug").join(GAME_EXECUTABLE);
    if debug_bin.exists() {
        return Some(debug_bin);
    }
    // 4. Si el launcher está dentro de target/debug o target/release
    if let Some(parent) = dir.parent().and_then(|p| p.parent()) {
        let rel = parent.join("target").join("release").join(GAME_EXECUTABLE);
        if rel.exists() {
            return Some(rel);
        }
        let deb = parent.join("target").join("debug").join(GAME_EXECUTABLE);
        if deb.exists() {
            return Some(deb);
        }
    }
    None
}

fn launch_game(game_path: &Path) {
    let mut cmd = Command::new(game_path);
    cmd.args(env::args().skip(1));
    if let Some(parent) = game_path.parent() {
        cmd.current_dir(parent);
    }
    let _ = cmd.spawn();
}

fn check_github(state_arc: &Arc<Mutex<AppState>>) {
    {
        let mut st = state_arc.lock().unwrap();
        st.status = UpdateStatus::Checking;
        st.remote_version = "Verificando...".to_string();
    }

    let url_release = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(Duration::from_secs(12))
        .build();

    let response = agent
        .get(&url_release)
        .set("User-Agent", "WorldOfAzeria-Launcher/1.0")
        .set("Accept", "application/vnd.github.v3+json")
        .call();

    match response {
        Ok(res) => {
            if let Ok(release) = res.into_json::<GitHubRelease>() {
                let mut st = state_arc.lock().unwrap();
                st.remote_version = release.tag_name.clone();
                let local = st.local_version.clone();

                let is_newer = !local.is_empty() && local != release.tag_name;
                if is_newer {
                    let zip_asset = release.assets.iter().find(|a| a.name.ends_with(".zip"));
                    st.status = UpdateStatus::UpdateAvailable {
                        remote_tag: release.tag_name.clone(),
                        download_url: zip_asset.map(|a| a.browser_download_url.clone()),
                        total_bytes: zip_asset.map_or(0, |a| a.size),
                    };
                } else {
                    st.status = UpdateStatus::UpToDate;
                }
                return;
            }
        }
        Err(ureq::Error::Status(404, _)) => {
            // No hay releases creados todavía en el repo: consultar el último commit de master
            let url_commits = format!("https://api.github.com/repos/{}/commits/master", GITHUB_REPO);
            if let Ok(res_c) = agent
                .get(&url_commits)
                .set("User-Agent", "WorldOfAzeria-Launcher/1.0")
                .call()
            {
                if let Ok(commit) = res_c.into_json::<GitHubCommit>() {
                    let short_sha = if commit.sha.len() >= 7 {
                        format!("master ({})", &commit.sha[..7])
                    } else {
                        commit.sha
                    };
                    let mut st = state_arc.lock().unwrap();
                    st.remote_version = short_sha;
                    st.status = UpdateStatus::UpToDate;
                    return;
                }
            }
            let mut st = state_arc.lock().unwrap();
            st.remote_version = "v0.18.0 (Al día)".to_string();
            st.status = UpdateStatus::UpToDate;
            return;
        }
        Err(e) => {
            let mut st = state_arc.lock().unwrap();
            st.remote_version = "Desconectado".to_string();
            st.status = UpdateStatus::Offline(format!("No se pudo conectar a GitHub: {}", e));
            return;
        }
    }

    let mut st = state_arc.lock().unwrap();
    st.status = UpdateStatus::UpToDate;
}

fn perform_update(
    state_arc: &Arc<Mutex<AppState>>,
    download_url: String,
    game_dir: PathBuf,
    remote_tag: String,
) {
    let temp_zip = game_dir.join("update_temp.zip");

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(300))
        .build();

    let res = agent
        .get(&download_url)
        .set("User-Agent", "WorldOfAzeria-Launcher/1.0")
        .call();

    let response = match res {
        Ok(r) => r,
        Err(e) => {
            let mut st = state_arc.lock().unwrap();
            st.status = UpdateStatus::Error(format!("Error de conexión: {}", e));
            return;
        }
    };

    let total_size = response
        .header("Content-Length")
        .and_then(|l| l.parse::<u64>().ok())
        .unwrap_or(0);

    let mut reader = response.into_reader();
    let mut file = match File::create(&temp_zip) {
        Ok(f) => f,
        Err(e) => {
            let mut st = state_arc.lock().unwrap();
            st.status = UpdateStatus::Error(format!("No se pudo crear archivo temporal: {}", e));
            return;
        }
    };

    let mut buffer = [0u8; 64 * 1024];
    let mut downloaded: u64 = 0;

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                if file.write_all(&buffer[..n]).is_err() {
                    let mut st = state_arc.lock().unwrap();
                    st.status = UpdateStatus::Error("Fallo al escribir actualización en disco".into());
                    return;
                }
                downloaded += n as u64;

                let pct = if total_size > 0 {
                    (downloaded as f32 / total_size as f32) * 100.0
                } else {
                    0.0
                };

                let mut st = state_arc.lock().unwrap();
                st.status = UpdateStatus::Downloading {
                    downloaded_bytes: downloaded,
                    total_bytes: total_size,
                    pct,
                };
            }
            Err(e) => {
                let mut st = state_arc.lock().unwrap();
                st.status = UpdateStatus::Error(format!("Error leyendo descarga: {}", e));
                return;
            }
        }
    }

    // Extracción del ZIP
    {
        let mut st = state_arc.lock().unwrap();
        st.status = UpdateStatus::Extracting;
    }

    let file_zip = match File::open(&temp_zip) {
        Ok(f) => f,
        Err(e) => {
            let mut st = state_arc.lock().unwrap();
            st.status = UpdateStatus::Error(format!("Error abriendo ZIP: {}", e));
            return;
        }
    };

    let mut archive = match zip::ZipArchive::new(file_zip) {
        Ok(a) => a,
        Err(e) => {
            let mut st = state_arc.lock().unwrap();
            st.status = UpdateStatus::Error(format!("ZIP inválido o corrupto: {}", e));
            return;
        }
    };

    for i in 0..archive.len() {
        if let Ok(mut item) = archive.by_index(i) {
            if let Some(enclosed) = item.enclosed_name() {
                let outpath = game_dir.join(enclosed);
                if item.is_dir() {
                    let _ = fs::create_dir_all(&outpath);
                } else {
                    if let Some(p) = outpath.parent() {
                        let _ = fs::create_dir_all(p);
                    }
                    if let Ok(mut outfile) = File::create(&outpath) {
                        let _ = io::copy(&mut item, &mut outfile);
                    }
                }
            }
        }
    }

    let _ = fs::remove_file(&temp_zip);

    let new_ver = LocalVersion {
        version: remote_tag.clone(),
        tag: remote_tag.clone(),
        updated_at: String::new(),
    };
    save_local_version(&game_dir, &new_ver);

    let mut st = state_arc.lock().unwrap();
    st.local_version = remote_tag;
    st.game_binary = find_game_binary(&game_dir);
    st.status = UpdateStatus::UpToDate;
}

#[cfg(windows)]
const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

#[cfg(windows)]
struct UiContext {
    state: Arc<Mutex<AppState>>,
    game_dir: PathBuf,
    btn_play_rect: RECT,
    btn_check_rect: RECT,
}

#[cfg(windows)]
static GLOBAL_UI_CTX: Mutex<Option<UiContext>> = Mutex::new(None);

#[cfg(windows)]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rect: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut rect);
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, width, height);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            if let Ok(guard) = GLOBAL_UI_CTX.lock() {
                if let Some(ctx) = guard.as_ref() {
                    paint_ui(mem_dc, width, height, ctx);
                }
            }

            BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

            SelectObject(mem_dc, old_bmp);
            DeleteObject(mem_bmp);
            DeleteDC(mem_dc);
            EndPaint(hwnd, &ps);
            0
        }
        WM_TIMER => {
            if let Ok(guard) = GLOBAL_UI_CTX.lock() {
                if let Some(ctx) = guard.as_ref() {
                    let mut st = ctx.state.lock().unwrap();
                    st.animation_tick = st.animation_tick.wrapping_add(1);
                }
            }
            InvalidateRect(hwnd, std::ptr::null(), 0);
            0
        }
        WM_MOUSEMOVE => {
            let x = (lparam as usize & 0xFFFF) as i16 as i32;
            let y = ((lparam as usize >> 16) & 0xFFFF) as i16 as i32;

            let mut tme: TRACKMOUSEEVENT = std::mem::zeroed();
            tme.cbSize = std::mem::size_of::<TRACKMOUSEEVENT>() as u32;
            tme.dwFlags = TME_LEAVE;
            tme.hwndTrack = hwnd;
            TrackMouseEvent(&mut tme);

            if let Ok(guard) = GLOBAL_UI_CTX.lock() {
                if let Some(ctx) = guard.as_ref() {
                    let mut st = ctx.state.lock().unwrap();
                    let old_hover = st.hovered_button;

                    let in_btn_play = x >= ctx.btn_play_rect.left
                        && x <= ctx.btn_play_rect.right
                        && y >= ctx.btn_play_rect.top
                        && y <= ctx.btn_play_rect.bottom;

                    let in_btn_check = x >= ctx.btn_check_rect.left
                        && x <= ctx.btn_check_rect.right
                        && y >= ctx.btn_check_rect.top
                        && y <= ctx.btn_check_rect.bottom;

                    if in_btn_play {
                        st.hovered_button = Some(ButtonId::PlayOrUpdate);
                    } else if in_btn_check {
                        st.hovered_button = Some(ButtonId::CheckUpdates);
                    } else {
                        st.hovered_button = None;
                    }

                    if old_hover != st.hovered_button {
                        InvalidateRect(hwnd, std::ptr::null(), 0);
                    }
                }
            }
            0
        }
        WM_MOUSELEAVE => {
            if let Ok(guard) = GLOBAL_UI_CTX.lock() {
                if let Some(ctx) = guard.as_ref() {
                    let mut st = ctx.state.lock().unwrap();
                    st.hovered_button = None;
                    st.pressed_button = None;
                    InvalidateRect(hwnd, std::ptr::null(), 0);
                }
            }
            0
        }
        WM_LBUTTONDOWN => {
            if let Ok(guard) = GLOBAL_UI_CTX.lock() {
                if let Some(ctx) = guard.as_ref() {
                    let mut st = ctx.state.lock().unwrap();
                    st.pressed_button = st.hovered_button;
                    InvalidateRect(hwnd, std::ptr::null(), 0);
                }
            }
            0
        }
        WM_LBUTTONUP => {
            let mut action = None;
            if let Ok(guard) = GLOBAL_UI_CTX.lock() {
                if let Some(ctx) = guard.as_ref() {
                    let mut st = ctx.state.lock().unwrap();
                    if st.pressed_button.is_some() && st.pressed_button == st.hovered_button {
                        action = st.hovered_button;
                    }
                    st.pressed_button = None;
                }
            }

            if let Some(btn) = action {
                handle_button_click(hwnd, btn);
            }
            InvalidateRect(hwnd, std::ptr::null(), 0);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(windows)]
fn handle_button_click(_hwnd: HWND, btn: ButtonId) {
    if let Ok(guard) = GLOBAL_UI_CTX.lock() {
        if let Some(ctx) = guard.as_ref() {
            match btn {
                ButtonId::PlayOrUpdate => {
                    let (status, binary) = {
                        let st = ctx.state.lock().unwrap();
                        (st.status.clone(), st.game_binary.clone())
                    };

                    match status {
                        UpdateStatus::UpdateAvailable {
                            remote_tag,
                            download_url,
                            ..
                        } => {
                            if let Some(url) = download_url {
                                let state_arc = Arc::clone(&ctx.state);
                                let gdir = ctx.game_dir.clone();
                                thread::spawn(move || {
                                    perform_update(&state_arc, url, gdir, remote_tag);
                                });
                            } else {
                                // Sin URL zip directa, lanzar juego si existe
                                if let Some(bin) = binary {
                                    launch_game(&bin);
                                    unsafe { PostQuitMessage(0); }
                                }
                            }
                        }
                        UpdateStatus::Downloading { .. } | UpdateStatus::Extracting => {
                            // En progreso, ignorar clicks
                        }
                        _ => {
                            // Iniciar el juego
                            if let Some(bin) = binary {
                                launch_game(&bin);
                                unsafe { PostQuitMessage(0); }
                            } else {
                                // Buscar nuevamente el binario
                                let found = find_game_binary(&ctx.game_dir);
                                if let Some(bin) = found {
                                    launch_game(&bin);
                                    unsafe { PostQuitMessage(0); }
                                }
                            }
                        }
                    }
                }
                ButtonId::CheckUpdates => {
                    let state_arc = Arc::clone(&ctx.state);
                    thread::spawn(move || {
                        check_github(&state_arc);
                    });
                }
            }
        }
    }
}

#[cfg(windows)]
unsafe fn paint_ui(
    hdc: windows_sys::Win32::Graphics::Gdi::HDC,
    width: i32,
    height: i32,
    ctx: &UiContext,
) {
    let (
        local_ver,
        remote_ver,
        status,
        game_binary_found,
        hovered_btn,
        pressed_btn,
        _anim_tick,
    ) = {
        let st = ctx.state.lock().unwrap();
        (
            st.local_version.clone(),
            st.remote_version.clone(),
            st.status.clone(),
            st.game_binary.is_some(),
            st.hovered_button,
            st.pressed_button,
            st.animation_tick,
        )
    };

    // 1. Fondo principal elegante de obsidiana / noche profunda
    let bg_brush = CreateSolidBrush(rgb(14, 18, 25));
    let window_rect = RECT {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    };
    FillRect(hdc, &window_rect, bg_brush);
    DeleteObject(bg_brush);

    // 2. Banner de cabecera con textura de alta fantasía
    let header_brush = CreateSolidBrush(rgb(20, 26, 38));
    let header_rect = RECT {
        left: 0,
        top: 0,
        right: width,
        bottom: 82,
    };
    FillRect(hdc, &header_rect, header_brush);
    DeleteObject(header_brush);

    // Línea de acento dorado bajo la cabecera
    let gold_pen = CreatePen(PS_SOLID, 2, rgb(214, 158, 46));
    let old_pen = SelectObject(hdc, gold_pen);
    let mut pt: POINT = std::mem::zeroed();
    windows_sys::Win32::Graphics::Gdi::MoveToEx(hdc, 0, 82, &mut pt);
    windows_sys::Win32::Graphics::Gdi::LineTo(hdc, width, 82);

    SetBkMode(hdc, TRANSPARENT as i32);

    // Título Principal "WORLD OF AZERIA"
    let font_title = CreateFontW(
        26, 0, 0, 0, 700, 0, 0, 0, 0, 0, 0, 0, 0,
        to_wide("Segoe UI").as_ptr(),
    );
    let old_font = SelectObject(hdc, font_title);
    SetTextColor(hdc, rgb(246, 173, 85)); // Oro cálido
    let mut title_rect = RECT {
        left: 28,
        top: 14,
        right: width - 28,
        bottom: 46,
    };
    let title_w = to_wide("WORLD OF AZERIA");
    DrawTextW(
        hdc,
        title_w.as_ptr(),
        title_w.len() as i32 - 1,
        &mut title_rect,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
    );

    // Subtítulo
    let font_sub = CreateFontW(
        15, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0,
        to_wide("Segoe UI").as_ptr(),
    );
    SelectObject(hdc, font_sub);
    SetTextColor(hdc, rgb(160, 174, 192)); // Gris azulado
    let mut sub_rect = RECT {
        left: 30,
        top: 48,
        right: width - 30,
        bottom: 72,
    };
    let sub_w = to_wide("Lanzador Oficial y Sistema de Actualizaciones Automáticas");
    DrawTextW(
        hdc,
        sub_w.as_ptr(),
        sub_w.len() as i32 - 1,
        &mut sub_rect,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
    );

    // 3. Tarjetas de Versión: Tu Versión vs Servidor/GitHub
    let card_y_top = 98;
    let card_y_bot = 178;
    let card_w = 265;
    let card_1_rect = RECT {
        left: 30,
        top: card_y_top,
        right: 30 + card_w,
        bottom: card_y_bot,
    };
    let card_2_rect = RECT {
        left: width - 30 - card_w,
        top: card_y_top,
        right: width - 30,
        bottom: card_y_bot,
    };

    let card_brush = CreateSolidBrush(rgb(23, 30, 43));
    let border_pen = CreatePen(PS_SOLID, 1, rgb(45, 55, 72));
    SelectObject(hdc, border_pen);
    SelectObject(hdc, card_brush);
    RoundRect(hdc, card_1_rect.left, card_1_rect.top, card_1_rect.right, card_1_rect.bottom, 10, 10);
    RoundRect(hdc, card_2_rect.left, card_2_rect.top, card_2_rect.right, card_2_rect.bottom, 10, 10);

    // Etiquetas de las tarjetas
    let font_label = CreateFontW(
        13, 0, 0, 0, 600, 0, 0, 0, 0, 0, 0, 0, 0,
        to_wide("Segoe UI").as_ptr(),
    );
    let font_val = CreateFontW(
        19, 0, 0, 0, 700, 0, 0, 0, 0, 0, 0, 0, 0,
        to_wide("Segoe UI").as_ptr(),
    );

    // Tarjeta 1: Tu Versión Local
    SelectObject(hdc, font_label);
    SetTextColor(hdc, rgb(160, 174, 192));
    let mut lbl1 = RECT {
        left: card_1_rect.left + 16,
        top: card_1_rect.top + 12,
        right: card_1_rect.right - 16,
        bottom: card_1_rect.top + 32,
    };
    let lbl1_w = to_wide("TU VERSIÓN LOCAL");
    DrawTextW(hdc, lbl1_w.as_ptr(), lbl1_w.len() as i32 - 1, &mut lbl1, DT_LEFT | DT_SINGLELINE);

    SelectObject(hdc, font_val);
    SetTextColor(hdc, rgb(129, 230, 217)); // Cian menta luminoso
    let mut val1 = RECT {
        left: card_1_rect.left + 16,
        top: card_1_rect.top + 34,
        right: card_1_rect.right - 16,
        bottom: card_1_rect.bottom - 10,
    };
    let val1_w = to_wide(&local_ver);
    DrawTextW(hdc, val1_w.as_ptr(), val1_w.len() as i32 - 1, &mut val1, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

    // Tarjeta 2: Servidor / GitHub
    SelectObject(hdc, font_label);
    SetTextColor(hdc, rgb(160, 174, 192));
    let mut lbl2 = RECT {
        left: card_2_rect.left + 16,
        top: card_2_rect.top + 12,
        right: card_2_rect.right - 16,
        bottom: card_2_rect.top + 32,
    };
    let lbl2_w = to_wide("SERVIDOR / GITHUB");
    DrawTextW(hdc, lbl2_w.as_ptr(), lbl2_w.len() as i32 - 1, &mut lbl2, DT_LEFT | DT_SINGLELINE);

    SelectObject(hdc, font_val);
    SetTextColor(hdc, rgb(250, 204, 21)); // Oro vibrante
    let mut val2 = RECT {
        left: card_2_rect.left + 16,
        top: card_2_rect.top + 34,
        right: card_2_rect.right - 16,
        bottom: card_2_rect.bottom - 10,
    };
    let val2_w = to_wide(&remote_ver);
    DrawTextW(hdc, val2_w.as_ptr(), val2_w.len() as i32 - 1, &mut val2, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

    // 4. Mensaje de Estado
    let font_status = CreateFontW(
        15, 0, 0, 0, 600, 0, 0, 0, 0, 0, 0, 0, 0,
        to_wide("Segoe UI").as_ptr(),
    );
    SelectObject(hdc, font_status);

    let (status_text, status_color, progress_pct) = match &status {
        UpdateStatus::Checking => (
            "⏳ Conectando con GitHub y verificando actualizaciones...".to_string(),
            rgb(99, 179, 237), // Azul claro
            20.0,
        ),
        UpdateStatus::UpToDate => (
            "✔ Ya estás en la última versión. ¡Todo listo para jugar!".to_string(),
            rgb(72, 187, 120), // Verde esmeralda
            100.0,
        ),
        UpdateStatus::UpdateAvailable { remote_tag, .. } => (
            format!("⚡ ¡Nueva versión disponible ({})! Presiona 'Actualizar' para descargar.", remote_tag),
            rgb(236, 201, 75), // Oro/ámbar
            100.0,
        ),
        UpdateStatus::Downloading { downloaded_bytes, total_bytes, pct } => {
            let mb_down = *downloaded_bytes as f32 / (1024.0 * 1024.0);
            let mb_tot = *total_bytes as f32 / (1024.0 * 1024.0);
            (
                format!("📥 Descargando actualización: {:.1} MB / {:.1} MB ({:.0}%)", mb_down, mb_tot, pct),
                rgb(246, 173, 85),
                *pct,
            )
        }
        UpdateStatus::Extracting => (
            "📦 Instalando y extrayendo archivos del juego...".to_string(),
            rgb(214, 158, 46),
            95.0,
        ),
        UpdateStatus::Offline(_) => (
            "ℹ Conexión con GitHub no disponible. Modo fuera de línea activo.".to_string(),
            rgb(226, 232, 240),
            100.0,
        ),
        UpdateStatus::Error(err) => (
            format!("⚠ Error: {}", err),
            rgb(245, 101, 101),
            0.0,
        ),
    };

    SetTextColor(hdc, status_color);
    let mut status_rect = RECT {
        left: 32,
        top: 194,
        right: width - 32,
        bottom: 220,
    };
    let st_w = to_wide(&status_text);
    DrawTextW(hdc, st_w.as_ptr(), st_w.len() as i32 - 1, &mut status_rect, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

    // 5. Barra de Progreso Estilizada
    let bar_left = 30;
    let bar_right = width - 30;
    let bar_top = 226;
    let bar_bottom = 258;
    let bar_total_w = bar_right - bar_left;

    let bar_bg_brush = CreateSolidBrush(rgb(10, 14, 20));
    let bar_border_pen = CreatePen(PS_SOLID, 1, rgb(45, 55, 72));
    SelectObject(hdc, bar_border_pen);
    SelectObject(hdc, bar_bg_brush);
    RoundRect(hdc, bar_left, bar_top, bar_right, bar_bottom, 8, 8);

    // Relleno de la barra
    let fill_w = ((bar_total_w - 4) as f32 * (progress_pct.clamp(0.0, 100.0) / 100.0)) as i32;
    if fill_w > 0 {
        let bar_fill_col = match &status {
            UpdateStatus::Downloading { .. } | UpdateStatus::Extracting => rgb(217, 119, 6),
            UpdateStatus::UpdateAvailable { .. } => rgb(202, 138, 4),
            UpdateStatus::Error(_) => rgb(185, 28, 28),
            _ => rgb(47, 133, 90), // Verde esmeralda
        };
        let fill_brush = CreateSolidBrush(bar_fill_col);
        let no_pen = CreatePen(PS_SOLID, 1, bar_fill_col);
        SelectObject(hdc, no_pen);
        SelectObject(hdc, fill_brush);
        RoundRect(
            hdc,
            bar_left + 2,
            bar_top + 2,
            bar_left + 2 + fill_w,
            bar_bottom - 2,
            6,
            6,
        );
        DeleteObject(fill_brush);
        DeleteObject(no_pen);
    }

    // Texto sobre la barra de progreso
    let font_bar = CreateFontW(
        13, 0, 0, 0, 700, 0, 0, 0, 0, 0, 0, 0, 0,
        to_wide("Segoe UI").as_ptr(),
    );
    SelectObject(hdc, font_bar);
    SetTextColor(hdc, rgb(255, 255, 255));
    let bar_text = match &status {
        UpdateStatus::Downloading { pct, .. } => format!("{:.0}% completado", pct),
        UpdateStatus::Extracting => "Extrayendo...".to_string(),
        UpdateStatus::UpdateAvailable { .. } => "Actualización lista para descargar".to_string(),
        UpdateStatus::Checking => "Comprobando servidor...".to_string(),
        _ => "100% — Cliente al día".to_string(),
    };
    let mut bar_lbl_rect = RECT {
        left: bar_left,
        top: bar_top,
        right: bar_right,
        bottom: bar_bottom,
    };
    let bar_lbl_w = to_wide(&bar_text);
    DrawTextW(hdc, bar_lbl_w.as_ptr(), bar_lbl_w.len() as i32 - 1, &mut bar_lbl_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    // 6. Botones de Acción
    let font_btn = CreateFontW(
        18, 0, 0, 0, 700, 0, 0, 0, 0, 0, 0, 0, 0,
        to_wide("Segoe UI").as_ptr(),
    );
    SelectObject(hdc, font_btn);

    // Botón Primario: JUGAR o ACTUALIZAR
    let is_play_hovered = hovered_btn == Some(ButtonId::PlayOrUpdate);
    let is_play_pressed = pressed_btn == Some(ButtonId::PlayOrUpdate);

    let (btn_main_text, btn_main_col, btn_main_border) = match &status {
        UpdateStatus::UpdateAvailable { .. } => {
            if is_play_pressed {
                ("⚡ ACTUALIZAR AHORA", rgb(180, 83, 9), rgb(245, 158, 11))
            } else if is_play_hovered {
                ("⚡ ACTUALIZAR AHORA", rgb(245, 158, 11), rgb(251, 191, 36))
            } else {
                ("⚡ ACTUALIZAR AHORA", rgb(217, 119, 6), rgb(245, 158, 11))
            }
        }
        UpdateStatus::Downloading { .. } | UpdateStatus::Extracting => {
            ("DESCARGANDO...", rgb(55, 65, 81), rgb(75, 85, 99))
        }
        _ => {
            if !game_binary_found {
                ("JUEGO NO ENCONTRADO", rgb(75, 85, 99), rgb(107, 114, 128))
            } else if is_play_pressed {
                ("▶ JUGAR WORLD OF AZERIA", rgb(34, 110, 68), rgb(72, 187, 120))
            } else if is_play_hovered {
                ("▶ JUGAR WORLD OF AZERIA", rgb(56, 161, 105), rgb(104, 211, 145))
            } else {
                ("▶ JUGAR WORLD OF AZERIA", rgb(47, 133, 90), rgb(72, 187, 120))
            }
        }
    };

    let btn_play_brush = CreateSolidBrush(btn_main_col);
    let btn_play_pen = CreatePen(PS_SOLID, 2, btn_main_border);
    SelectObject(hdc, btn_play_pen);
    SelectObject(hdc, btn_play_brush);
    RoundRect(
        hdc,
        ctx.btn_play_rect.left,
        ctx.btn_play_rect.top,
        ctx.btn_play_rect.right,
        ctx.btn_play_rect.bottom,
        12,
        12,
    );

    SetTextColor(hdc, rgb(255, 255, 255));
    let mut btn_play_lbl = ctx.btn_play_rect;
    let btn_play_w = to_wide(btn_main_text);
    DrawTextW(hdc, btn_play_w.as_ptr(), btn_play_w.len() as i32 - 1, &mut btn_play_lbl, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    DeleteObject(btn_play_brush);
    DeleteObject(btn_play_pen);

    // Botón Secundario: Buscar Actualizaciones
    let is_check_hovered = hovered_btn == Some(ButtonId::CheckUpdates);
    let is_check_pressed = pressed_btn == Some(ButtonId::CheckUpdates);

    let (btn_sec_col, btn_sec_border) = if is_check_pressed {
        (rgb(30, 41, 59), rgb(100, 116, 139))
    } else if is_check_hovered {
        (rgb(51, 65, 85), rgb(148, 163, 184))
    } else {
        (rgb(30, 41, 59), rgb(71, 85, 105))
    };

    let btn_sec_brush = CreateSolidBrush(btn_sec_col);
    let btn_sec_pen = CreatePen(PS_SOLID, 1, btn_sec_border);
    SelectObject(hdc, btn_sec_pen);
    SelectObject(hdc, btn_sec_brush);
    RoundRect(
        hdc,
        ctx.btn_check_rect.left,
        ctx.btn_check_rect.top,
        ctx.btn_check_rect.right,
        ctx.btn_check_rect.bottom,
        12,
        12,
    );

    let font_btn_sec = CreateFontW(
        15, 0, 0, 0, 600, 0, 0, 0, 0, 0, 0, 0, 0,
        to_wide("Segoe UI").as_ptr(),
    );
    SelectObject(hdc, font_btn_sec);
    SetTextColor(hdc, rgb(226, 232, 240));
    let mut btn_check_lbl = ctx.btn_check_rect;
    let btn_check_w = to_wide("🔄 Buscar Updates");
    DrawTextW(hdc, btn_check_w.as_ptr(), btn_check_w.len() as i32 - 1, &mut btn_check_lbl, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    DeleteObject(btn_sec_brush);
    DeleteObject(btn_sec_pen);

    // 7. Pie de Página
    let font_footer = CreateFontW(
        12, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0,
        to_wide("Segoe UI").as_ptr(),
    );
    SelectObject(hdc, font_footer);
    SetTextColor(hdc, rgb(100, 116, 139));
    let mut footer_left = RECT {
        left: 30,
        top: height - 32,
        right: width / 2,
        bottom: height - 10,
    };
    let footer_w = to_wide("GitHub: kambire/veloren");
    DrawTextW(hdc, footer_w.as_ptr(), footer_w.len() as i32 - 1, &mut footer_left, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

    let mut footer_right = RECT {
        left: width / 2,
        top: height - 32,
        right: width - 30,
        bottom: height - 10,
    };
    let mode_w = to_wide(if matches!(status, UpdateStatus::Offline(_)) { "Modo Fuera de Línea" } else { "Conectado" });
    DrawTextW(hdc, mode_w.as_ptr(), mode_w.len() as i32 - 1, &mut footer_right, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);

    // Limpieza de objetos GDI creados
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_font);
    DeleteObject(gold_pen);
    DeleteObject(card_brush);
    DeleteObject(border_pen);
    DeleteObject(bar_bg_brush);
    DeleteObject(bar_border_pen);
    DeleteObject(font_title);
    DeleteObject(font_sub);
    DeleteObject(font_label);
    DeleteObject(font_val);
    DeleteObject(font_status);
    DeleteObject(font_bar);
    DeleteObject(font_btn);
    DeleteObject(font_btn_sec);
    DeleteObject(font_footer);
}

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn run_gui_launcher() {
    let game_dir = get_game_dir();
    let local = load_local_version(&game_dir);
    let binary = find_game_binary(&game_dir);

    let state = Arc::new(Mutex::new(AppState {
        local_version: local.tag,
        remote_version: "Comprobando...".to_string(),
        status: UpdateStatus::Checking,
        game_binary: binary,
        hovered_button: None,
        pressed_button: None,
        animation_tick: 0,
    }));

    // Iniciar verificación en segundo plano
    let state_bg = Arc::clone(&state);
    thread::spawn(move || {
        check_github(&state_bg);
    });

    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide("WorldOfAzeriaLauncherClass");

        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.style = CS_HREDRAW | CS_VREDRAW;
        wc.lpfnWndProc = Some(window_proc);
        wc.hInstance = hinstance;
        wc.hCursor = LoadCursorW(std::ptr::null_mut(), IDC_ARROW);
        wc.lpszClassName = class_name.as_ptr();

        RegisterClassExW(&wc);

        let win_w = 640;
        let win_h = 440;

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = (screen_w - win_w) / 2;
        let pos_y = (screen_h - win_h) / 2;

        let title = to_wide("World of Azeria — Lanzador y Actualizador");
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
            pos_x,
            pos_y,
            win_w,
            win_h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null(),
        );

        if let Ok(mut guard) = GLOBAL_UI_CTX.lock() {
            *guard = Some(UiContext {
                state: Arc::clone(&state),
                game_dir,
                btn_play_rect: RECT {
                    left: 30,
                    top: 280,
                    right: 420,
                    bottom: 340,
                },
                btn_check_rect: RECT {
                    left: 440,
                    top: 280,
                    right: win_w - 30,
                    bottom: 340,
                },
            });
        }

        // Temporizador de refresco a ~30 FPS para animaciones y actualización de progreso
        SetTimer(hwnd, 1, 33, None);

        ShowWindow(hwnd, SW_SHOW);
        SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_SHOWWINDOW);

        let mut msg: windows_sys::Win32::UI::WindowsAndMessaging::MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn main() {
    #[cfg(windows)]
    {
        run_gui_launcher();
    }

    #[cfg(not(windows))]
    {
        println!("World of Azeria Launcher");
        let game_dir = get_game_dir();
        if let Some(game_bin) = find_game_binary(&game_dir) {
            launch_game(&game_bin);
        } else {
            eprintln!("No se encontró el ejecutable.");
        }
    }
}
