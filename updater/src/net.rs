//! Versiones, novedades y descarga de actualizaciones desde GitHub

use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub const GITHUB_REPO: &str = "kambire/veloren";
const VERSION_FILE: &str = "version.json";
const USER_AGENT: &str = "WorldOfAzeria-Launcher/2.0";
pub const GAME_EXECUTABLE: &str = if cfg!(target_os = "windows") {
    "veloren-voxygen.exe"
} else {
    "veloren-voxygen"
};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LocalVersion {
    pub version: String,
    pub tag: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GitHubCommit {
    sha: String,
    commit: CommitInfo,
}

#[derive(Deserialize)]
struct CommitInfo {
    message: String,
    author: Option<CommitAuthor>,
}

#[derive(Deserialize)]
struct CommitAuthor {
    date: Option<String>,
}

/// Una entrada del panel de novedades
#[derive(Clone, Debug)]
pub struct NewsItem {
    /// Fecha corta ("24/09"), puede estar vacía
    pub date: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateStatus {
    Checking,
    UpToDate,
    UpdateAvailable {
        remote_tag: String,
        download_url: Option<String>,
        total_bytes: u64,
    },
    Downloading {
        downloaded: u64,
        total: u64,
        /// Bytes por segundo
        speed: f64,
    },
    Extracting,
    Error(String),
    /// Sin conexión con GitHub: se puede jugar igualmente
    Offline,
}

/// Estado que comparten la ventana y los hilos de red
pub struct AppState {
    pub local_version: String,
    pub remote_version: String,
    pub status: UpdateStatus,
    pub game_binary: Option<PathBuf>,
    /// `None` mientras se cargan
    pub news: Option<Vec<NewsItem>>,
}

pub type Shared = Arc<Mutex<AppState>>;

pub fn get_game_dir() -> PathBuf {
    let mut dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // Si se ejecuta desde target/debug o target/release, subir a la raíz del repo
    if dir.ends_with("target/debug")
        || dir.ends_with("target\\debug")
        || dir.ends_with("target/release")
        || dir.ends_with("target\\release")
    {
        if let Some(parent) = dir.parent().and_then(|p| p.parent()) {
            dir = parent.to_path_buf();
        }
    }
    dir
}

pub fn load_local_version(dir: &Path) -> LocalVersion {
    fs::read_to_string(dir.join(VERSION_FILE))
        .ok()
        .and_then(|content| serde_json::from_str::<LocalVersion>(&content).ok())
        .unwrap_or_else(|| LocalVersion {
            version: "0.18.0".to_string(),
            tag: "v0.18.0".to_string(),
            updated_at: String::new(),
        })
}

fn save_local_version(dir: &Path, version: &LocalVersion) {
    if let Ok(content) = serde_json::to_string_pretty(version) {
        let _ = fs::write(dir.join(VERSION_FILE), content);
    }
}

pub fn find_game_binary(dir: &Path) -> Option<PathBuf> {
    [
        // 1. Junto al lanzador (versión descargada)
        dir.join(GAME_EXECUTABLE),
        // 2. Compilación de desarrollo
        dir.join("target").join("debug").join(GAME_EXECUTABLE),
        // 3. Compilación optimizada
        dir.join("target").join("release").join(GAME_EXECUTABLE),
    ]
    .into_iter()
    .find(|path| path.exists())
}

pub fn launch_game(game_path: &Path, game_dir: &Path, args: &[String]) {
    let _ = Command::new(game_path)
        .args(args)
        .current_dir(game_dir)
        .spawn();
}

fn agent(read_timeout: Duration) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(read_timeout)
        .build()
}

fn github_get(agent: &ureq::Agent, url: &str) -> Result<ureq::Response, ureq::Error> {
    agent
        .get(url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github.v3+json")
        .call()
}

/// Comprueba si hay una versión nueva publicada en GitHub
pub fn check_github(state: &Shared) {
    {
        let mut st = state.lock().unwrap();
        st.status = UpdateStatus::Checking;
        st.remote_version = "Comprobando...".to_string();
    }

    let agent = agent(Duration::from_secs(12));
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");

    match github_get(&agent, &url) {
        Ok(res) => {
            let Ok(release) = res.into_json::<GitHubRelease>() else {
                let mut st = state.lock().unwrap();
                st.remote_version = "Desconocida".to_string();
                st.status = UpdateStatus::Error("Respuesta de GitHub no válida".to_string());
                return;
            };
            let mut st = state.lock().unwrap();
            st.remote_version = release.tag_name.clone();
            if !st.local_version.is_empty() && st.local_version != release.tag_name {
                let zip_asset = release.assets.iter().find(|a| a.name.ends_with(".zip"));
                st.status = UpdateStatus::UpdateAvailable {
                    remote_tag: release.tag_name.clone(),
                    download_url: zip_asset.map(|a| a.browser_download_url.clone()),
                    total_bytes: zip_asset.map_or(0, |a| a.size),
                };
            } else {
                st.status = UpdateStatus::UpToDate;
            }
        },
        // Todavía no hay ninguna versión publicada: mostrar el último commit
        Err(ureq::Error::Status(404, _)) => {
            let url = format!("https://api.github.com/repos/{GITHUB_REPO}/commits/master");
            let sha = github_get(&agent, &url)
                .ok()
                .and_then(|r| r.into_json::<GitHubCommit>().ok())
                .map(|c| c.sha.chars().take(7).collect::<String>());
            let mut st = state.lock().unwrap();
            st.remote_version = match sha {
                Some(sha) => format!("master ({sha})"),
                None => st.local_version.clone(),
            };
            st.status = UpdateStatus::UpToDate;
        },
        Err(_) => {
            let mut st = state.lock().unwrap();
            st.remote_version = "Sin conexión".to_string();
            st.status = UpdateStatus::Offline;
        },
    }
}

/// Carga los últimos cambios del proyecto para el panel de novedades
pub fn fetch_news(state: &Shared) {
    let agent = agent(Duration::from_secs(12));
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/commits?per_page=12");
    let news = github_get(&agent, &url)
        .ok()
        .and_then(|r| r.into_json::<Vec<GitHubCommit>>().ok())
        .map(|commits| {
            commits
                .into_iter()
                .filter_map(|c| {
                    let text = c.commit.message.lines().next()?.trim().to_string();
                    // Los merges no dicen nada al jugador
                    if text.is_empty() || text.starts_with("Merge") {
                        return None;
                    }
                    let date = c
                        .commit
                        .author
                        .and_then(|a| a.date)
                        .map(|d| short_date(&d))
                        .unwrap_or_default();
                    Some(NewsItem { date, text })
                })
                .take(4)
                .collect::<Vec<_>>()
        })
        .filter(|news| !news.is_empty())
        .unwrap_or_else(default_news);
    state.lock().unwrap().news = Some(news);
}

/// "2026-09-24T10:00:00Z" -> "24/09"
fn short_date(iso: &str) -> String {
    let parts: Vec<&str> = iso.get(..10).unwrap_or("").split('-').collect();
    match parts.as_slice() {
        [_, month, day] => format!("{day}/{month}"),
        _ => String::new(),
    }
}

/// Novedades que se muestran si no hay conexión
pub fn default_news() -> Vec<NewsItem> {
    [
        "Nuevo mundo: 4 continentes separados por océano",
        "Misiones y cadenas de misiones con marcadores ! y ?",
        "Nueva clase: el Entrenador y sus mascotas",
        "Talentos propios para cada clase",
    ]
    .into_iter()
    .map(|text| NewsItem {
        date: String::new(),
        text: text.to_string(),
    })
    .collect()
}

/// Descarga el .zip de la nueva versión y lo instala sobre la carpeta del juego
pub fn perform_update(state: &Shared, download_url: String, game_dir: PathBuf, remote_tag: String) {
    let set_error = |msg: String| state.lock().unwrap().status = UpdateStatus::Error(msg);
    let temp_zip = game_dir.join("update_temp.zip");

    let response = match agent(Duration::from_secs(300))
        .get(&download_url)
        .set("User-Agent", USER_AGENT)
        .call()
    {
        Ok(r) => r,
        Err(e) => return set_error(format!("Error de conexión: {e}")),
    };

    let total = response
        .header("Content-Length")
        .and_then(|l| l.parse::<u64>().ok())
        .unwrap_or(0);

    let mut reader = response.into_reader();
    let mut file = match File::create(&temp_zip) {
        Ok(f) => f,
        Err(e) => return set_error(format!("No se pudo crear el archivo temporal: {e}")),
    };

    let start = Instant::now();
    let mut buffer = [0u8; 64 * 1024];
    let mut downloaded: u64 = 0;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                if file.write_all(&buffer[..n]).is_err() {
                    return set_error("No se pudo escribir la actualización en disco".to_string());
                }
                downloaded += n as u64;
                let elapsed = start.elapsed().as_secs_f64().max(0.001);
                state.lock().unwrap().status = UpdateStatus::Downloading {
                    downloaded,
                    total,
                    speed: downloaded as f64 / elapsed,
                };
            },
            Err(e) => return set_error(format!("Error durante la descarga: {e}")),
        }
    }
    drop(file);

    state.lock().unwrap().status = UpdateStatus::Extracting;

    let archive = File::open(&temp_zip)
        .map_err(|e| format!("No se pudo abrir el .zip: {e}"))
        .and_then(|f| {
            zip::ZipArchive::new(f).map_err(|e| format!("El .zip está dañado: {e}"))
        });
    let mut archive = match archive {
        Ok(a) => a,
        Err(msg) => return set_error(msg),
    };

    for i in 0..archive.len() {
        if let Ok(mut item) = archive.by_index(i)
            // `enclosed_name` impide escribir fuera de la carpeta del juego
            && let Some(enclosed) = item.enclosed_name()
        {
            let outpath = game_dir.join(enclosed);
            if item.is_dir() {
                let _ = fs::create_dir_all(&outpath);
            } else {
                if let Some(parent) = outpath.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(mut outfile) = File::create(&outpath) {
                    let _ = io::copy(&mut item, &mut outfile);
                }
            }
        }
    }
    let _ = fs::remove_file(&temp_zip);

    save_local_version(&game_dir, &LocalVersion {
        version: remote_tag.clone(),
        tag: remote_tag.clone(),
        updated_at: String::new(),
    });

    let mut st = state.lock().unwrap();
    st.local_version = remote_tag;
    st.game_binary = find_game_binary(&game_dir);
    st.status = UpdateStatus::UpToDate;
}
