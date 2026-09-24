use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

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

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
    zipball_url: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

fn print_banner() {
    println!();
    println!("  ========================================================");
    println!("          WORLD OF AZERIA - LAUNCHER & AUTO-UPDATER       ");
    println!("  ========================================================");
    println!("   Repositorio: https://github.com/{}", GITHUB_REPO);
    println!("  ========================================================");
    println!();
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
    LocalVersion::default()
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

    None
}

fn check_github_update() -> Result<Option<GitHubRelease>, String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    println!("-> Verificando actualizaciones en GitHub...");

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(15))
        .build();

    let response = agent
        .get(&url)
        .set("User-Agent", "Veloren-Launcher-Updater/1.0")
        .set("Accept", "application/vnd.github.v3+json")
        .call();

    match response {
        Ok(res) => {
            let release: GitHubRelease = res
                .into_json()
                .map_err(|e| format!("Error procesando respuesta de GitHub: {}", e))?;
            Ok(Some(release))
        }
        Err(ureq::Error::Status(404, _)) => {
            // Aún no hay releases publicados en el repositorio
            Ok(None)
        }
        Err(e) => Err(format!("No se pudo conectar a GitHub: {}", e)),
    }
}

fn download_with_progress(url: &str, output_path: &Path) -> Result<(), String> {
    println!("-> Descargando actualización desde: {}", url);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(300))
        .build();

    let response = agent
        .get(url)
        .set("User-Agent", "Veloren-Launcher-Updater/1.0")
        .call()
        .map_err(|e| format!("Error iniciando descarga: {}", e))?;

    let total_size = response
        .header("Content-Length")
        .and_then(|l| l.parse::<u64>().ok())
        .unwrap_or(0);

    let mut reader = response.into_reader();
    let mut file = File::create(output_path).map_err(|e| format!("Error creando archivo temporal: {}", e))?;

    let mut buffer = [0u8; 64 * 1024]; // 64 KB
    let mut downloaded: u64 = 0;
    let mut last_print = std::time::Instant::now();

    loop {
        let n = reader.read(&mut buffer).map_err(|e| format!("Error de lectura durante la descarga: {}", e))?;
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n]).map_err(|e| format!("Error guardando archivo: {}", e))?;
        downloaded += n as u64;

        if last_print.elapsed() >= Duration::from_millis(150) || downloaded == total_size {
            last_print = std::time::Instant::now();
            let mb_downloaded = downloaded as f64 / (1024.0 * 1024.0);
            if total_size > 0 {
                let mb_total = total_size as f64 / (1024.0 * 1024.0);
                let pct = (downloaded as f64 / total_size as f64) * 100.0;
                let progress_bars = (pct / 5.0) as usize;
                let bar = format!("[{:=>width$}{:space$}]", "", "", width = progress_bars, space = 20 - progress_bars);
                print!("\r   {} {:5.1}% ({:.1} MB / {:.1} MB)   ", bar, pct, mb_downloaded, mb_total);
            } else {
                print!("\r   Descargados: {:.1} MB...   ", mb_downloaded);
            }
            let _ = io::stdout().flush();
        }
    }
    println!("\n-> Descarga completada exitosamente.");
    Ok(())
}

fn extract_zip(zip_path: &Path, target_dir: &Path) -> Result<(), String> {
    println!("-> Instalando actualización en {:?}...", target_dir);
    let file = File::open(zip_path).map_err(|e| format!("Error abriendo ZIP: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Error leyendo ZIP: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Error en archivo ZIP: {}", e))?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath).map_err(|e| format!("Error creando carpeta {:?}: {}", outpath, e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| format!("Error creando directorio {:?}: {}", p, e))?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| format!("Error creando archivo {:?}: {}", outpath, e))?;
            io::copy(&mut file, &mut outfile).map_err(|e| format!("Error extrayendo a {:?}: {}", outpath, e))?;
        }
    }
    println!("-> Extracción completada.");
    Ok(())
}

fn launch_game(game_path: &Path) {
    println!();
    println!("========================================================");
    println!("  Iniciando World of Azeria...");
    println!("  Ejecutable: {:?}", game_path);
    println!("========================================================");
    println!();

    let mut cmd = Command::new(game_path);
    // Heredar los argumentos de la línea de comandos
    cmd.args(env::args().skip(1));
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    if let Some(parent) = game_path.parent() {
        cmd.current_dir(parent);
    }

    match cmd.spawn() {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(e) => {
            eprintln!("[ERROR] No se pudo iniciar el juego: {}", e);
            eprintln!("Presiona Enter para salir...");
            let mut line = String::new();
            let _ = io::stdin().read_line(&mut line);
        }
    }
}

fn main() {
    print_banner();
    let game_dir = get_game_dir();
    let local_version = load_local_version(&game_dir);

    if !local_version.tag.is_empty() {
        println!("* Versión local instalada: {}", local_version.tag);
    } else {
        println!("* No se encontró versión local registrada.");
    }

    match check_github_update() {
        Ok(Some(release)) => {
            println!("* Última versión en GitHub: {}", release.tag_name);

            let is_new_version = local_version.tag.is_empty() || local_version.tag != release.tag_name;
            let binary_missing = find_game_binary(&game_dir).is_none();

            if is_new_version || binary_missing {
                if binary_missing {
                    println!("! El ejecutable del juego no fue encontrado en el directorio.");
                } else {
                    println!("! ¡Nueva actualización disponible! {} -> {}", local_version.tag, release.tag_name);
                }

                // Buscar un asset .zip en el release
                let zip_asset = release.assets.iter().find(|a| a.name.ends_with(".zip"));

                if let Some(asset) = zip_asset {
                    let temp_zip = game_dir.join("update_temp.zip");
                    match download_with_progress(&asset.browser_download_url, &temp_zip) {
                        Ok(()) => {
                            match extract_zip(&temp_zip, &game_dir) {
                                Ok(()) => {
                                    let new_version = LocalVersion {
                                        version: release.name.unwrap_or_else(|| release.tag_name.clone()),
                                        tag: release.tag_name.clone(),
                                        updated_at: release.published_at.unwrap_or_default(),
                                    };
                                    save_local_version(&game_dir, &new_version);
                                    let _ = fs::remove_file(&temp_zip);
                                    println!("-> [OK] ¡Juego actualizado con éxito a la versión {}!", release.tag_name);
                                }
                                Err(e) => {
                                    eprintln!("[ADVERTENCIA] Error extrayendo la actualización: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[ADVERTENCIA] Error descargando la actualización: {}", e);
                        }
                    }
                } else {
                    println!("(El release {} no contiene un archivo .zip compilado para descargar)", release.tag_name);
                }
            } else {
                println!("-> [OK] El juego ya está actualizado a la última versión disponible.");
            }
        }
        Ok(None) => {
            println!("* Conexión con GitHub exitosa: el repositorio aún no tiene releases creados.");
            println!("  El cliente utilizará la versión local.");
        }
        Err(err) => {
            println!("[AVISO] {}", err);
            println!("-> Modo fuera de línea o sin conexión a GitHub. Iniciando juego local...");
        }
    }

    if let Some(game_bin) = find_game_binary(&game_dir) {
        launch_game(&game_bin);
    } else {
        eprintln!();
        eprintln!("========================================================");
        eprintln!("  [ERROR] No se encontró el ejecutable '{GAME_EXECUTABLE}'.");
        eprintln!("  Por favor compila el juego con: cargo build --release --bin veloren-voxygen");
        eprintln!("  o descarga una versión pre-compilada desde GitHub Releases.");
        eprintln!("========================================================");
        eprintln!("Presiona Enter para salir...");
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
    }
}
