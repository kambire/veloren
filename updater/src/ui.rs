//! Interfaz del lanzador: se dibuja entera en software con tiny-skia (formas
//! suavizadas, degradados y el logo con transparencia) y ab_glyph (texto con
//! las fuentes del juego), y la ventana solo copia la imagen resultante.

use crate::net::{AppState, NewsItem, UpdateStatus};
use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use tiny_skia::{
    Color, FillRule, FilterQuality, GradientStop, LinearGradient, Paint, Path, PathBuilder,
    Pixmap, PixmapPaint, Point, RadialGradient, Rect, Shader, SpreadMode, Stroke, Transform,
};

pub const WIDTH: u32 = 1024;
pub const HEIGHT: u32 = 640;
const W: f32 = WIDTH as f32;
const H: f32 = HEIGHT as f32;
/// Alto de la barra de título propia (sirve para arrastrar la ventana)
pub const TITLE_H: f32 = 40.0;

// Disposición
const LOGO_W: f32 = 440.0;
const LOGO_Y: f32 = 46.0;
const PANEL_Y: f32 = 364.0;
const PANEL_H: f32 = 132.0;
const NEWS_X: f32 = 28.0;
const NEWS_W: f32 = 560.0;
const INFO_X: f32 = 604.0;
const INFO_W: f32 = 392.0;
const BAND_Y: f32 = 512.0;
const STATUS_Y: f32 = 541.0;
const PROG_X: f32 = 28.0;
const PROG_Y: f32 = 553.0;
const PROG_W: f32 = 690.0;
const PROG_H: f32 = 12.0;

// Paleta: azul noche, dorado del logo y azul de las gemas
const GOLD: [u8; 4] = [233, 196, 106, 255];
const GOLD_DARK: [u8; 4] = [216, 169, 72, 255];
const TEXT: [u8; 4] = [236, 240, 246, 255];
const MUTED: [u8; 4] = [147, 164, 189, 255];
const GREEN: [u8; 4] = [111, 208, 140, 255];
const BLUE: [u8; 4] = [127, 179, 255, 255];
const RED: [u8; 4] = [242, 110, 110, 255];

static FONT_TITLE: &[u8] = include_bytes!("../../assets/voxygen/font/Metamorphous-Regular.ttf");
static FONT_BODY: &[u8] = include_bytes!("../../assets/voxygen/font/OpenSans-Regular.ttf");
static IMG_LOGO: &[u8] = include_bytes!("../assets/logo.png");
static IMG_ICON: &[u8] = include_bytes!("../assets/icono.png");
static IMG_BACKGROUND: &[u8] = include_bytes!("../assets/fondo.png");

/// Zonas interactivas de la ventana
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Hit {
    Play,
    CheckUpdates,
    OpenFolder,
    Website,
    Minimize,
    Close,
}

const HITS: [Hit; 6] = [
    Hit::Play,
    Hit::CheckUpdates,
    Hit::OpenFolder,
    Hit::Website,
    Hit::Minimize,
    Hit::Close,
];

/// Rectángulo (x, y, ancho, alto) de cada zona interactiva
fn hit_rect(hit: Hit) -> (f32, f32, f32, f32) {
    match hit {
        Hit::Close => (W - 46.0, 0.0, 46.0, TITLE_H),
        Hit::Minimize => (W - 92.0, 0.0, 46.0, TITLE_H),
        Hit::Play => (746.0, 530.0, 250.0, 82.0),
        Hit::CheckUpdates => (28.0, 578.0, 204.0, 32.0),
        Hit::OpenFolder => (242.0, 578.0, 156.0, 32.0),
        Hit::Website => (408.0, 578.0, 146.0, 32.0),
    }
}

pub fn hit_test(x: f32, y: f32) -> Option<Hit> {
    HITS.iter().copied().find(|hit| {
        let (rx, ry, rw, rh) = hit_rect(*hit);
        x >= rx && x < rx + rw && y >= ry && y < ry + rh
    })
}

/// Recursos cargados una vez: fuentes, icono y la capa estática ya compuesta
pub struct Resources {
    title_font: FontRef<'static>,
    body_font: FontRef<'static>,
    background: Pixmap,
}

impl Resources {
    pub fn load() -> Self {
        let title_font = FontRef::try_from_slice(FONT_TITLE).expect("fuente Metamorphous");
        let body_font = FontRef::try_from_slice(FONT_BODY).expect("fuente Open Sans");
        let mut res = Self {
            title_font,
            body_font,
            background: Pixmap::new(WIDTH, HEIGHT).expect("tamaño de ventana válido"),
        };
        res.background = res.compose_background();
        res
    }

    /// Todo lo que no cambia: fondo, logo, marcos de los paneles y títulos
    fn compose_background(&self) -> Pixmap {
        let mut pm = Pixmap::new(WIDTH, HEIGHT).expect("tamaño de ventana válido");
        pm.fill(rgba([7, 11, 20, 255]));

        // Mapa de Azeria difuminado al fondo
        if let Ok(map) = Pixmap::decode_png(IMG_BACKGROUND) {
            pm.draw_pixmap(
                0,
                0,
                map.as_ref(),
                &PixmapPaint {
                    opacity: 0.6,
                    quality: FilterQuality::Bicubic,
                    ..Default::default()
                },
                Transform::identity(),
                None,
            );
        }
        fill_rect(&mut pm, 0.0, 0.0, W, H, vgrad(0.0, H, &[
            (0.0, [7, 11, 20, 200]),
            (0.32, [7, 11, 20, 80]),
            (0.6, [7, 11, 20, 150]),
            (1.0, [4, 7, 14, 250]),
        ]));
        // Viñeta y halo azul detrás del logo
        fill_rect(&mut pm, 0.0, 0.0, W, H, radial(W / 2.0, 300.0, 720.0, &[
            (0.45, [0, 0, 0, 0]),
            (1.0, [0, 0, 0, 175]),
        ]));
        fill_rect(&mut pm, 0.0, 0.0, W, H, radial(W / 2.0, 200.0, 330.0, &[
            (0.0, [60, 120, 255, 70]),
            (1.0, [60, 120, 255, 0]),
        ]));

        // Logo
        if let Ok(logo) = Pixmap::decode_png(IMG_LOGO) {
            let scale = LOGO_W / logo.width() as f32;
            pm.draw_pixmap(
                0,
                0,
                logo.as_ref(),
                &PixmapPaint {
                    quality: FilterQuality::Bicubic,
                    ..Default::default()
                },
                Transform::from_scale(scale, scale).post_translate((W - LOGO_W) / 2.0, LOGO_Y),
                None,
            );
        }

        // Barra de título
        fill_rect(&mut pm, 0.0, 0.0, W, TITLE_H, solid([3, 6, 12, 150]));
        fill_rect(&mut pm, 0.0, TITLE_H - 1.0, W, 1.0, hgrad(0.0, W, &[
            (0.0, [216, 169, 72, 0]),
            (0.5, [216, 169, 72, 110]),
            (1.0, [216, 169, 72, 0]),
        ]));
        if let Ok(icon) = Pixmap::decode_png(IMG_ICON) {
            let scale = 26.0 / icon.width() as f32;
            pm.draw_pixmap(
                0,
                0,
                icon.as_ref(),
                &PixmapPaint {
                    quality: FilterQuality::Bicubic,
                    ..Default::default()
                },
                Transform::from_scale(scale, scale).post_translate(12.0, 7.0),
                None,
            );
        }
        let x = draw_text(&mut pm, &self.title_font, 17.0, 46.0, 26.0, GOLD, "World of Azeria");
        draw_text(&mut pm, &self.body_font, 12.5, x + 12.0, 25.0, MUTED, "Lanzador oficial");

        // Paneles de novedades e información
        for (x, w, title) in [
            (NEWS_X, NEWS_W, "NOVEDADES"),
            (INFO_X, INFO_W, "INFORMACIÓN"),
        ] {
            if let Some(path) = rounded_rect(x, PANEL_Y, w, PANEL_H, 10.0) {
                fill_path(&mut pm, &path, solid([9, 15, 29, 205]));
                stroke_path(&mut pm, &path, [216, 169, 72, 80], 1.0);
            }
            draw_text(&mut pm, &self.title_font, 15.0, x + 18.0, PANEL_Y + 26.0, GOLD, title);
            fill_rect(&mut pm, x + 18.0, PANEL_Y + 35.0, w - 36.0, 1.0, hgrad(x, x + w, &[
                (0.0, [216, 169, 72, 90]),
                (1.0, [216, 169, 72, 0]),
            ]));
        }
        draw_text_right(
            &mut pm,
            &self.body_font,
            11.5,
            NEWS_X + NEWS_W - 18.0,
            PANEL_Y + 25.0,
            MUTED,
            "desde GitHub",
        );

        // Banda inferior
        fill_rect(&mut pm, 0.0, BAND_Y, W, H - BAND_Y, solid([5, 9, 18, 235]));
        fill_rect(&mut pm, 0.0, BAND_Y, W, 1.5, hgrad(0.0, W, &[
            (0.0, [216, 169, 72, 0]),
            (0.5, [216, 169, 72, 200]),
            (1.0, [216, 169, 72, 0]),
        ]));
        let dim = [MUTED[0], MUTED[1], MUTED[2], 150];
        draw_text(
            &mut pm,
            &self.body_font,
            11.0,
            28.0,
            630.0,
            dim,
            "World of Azeria · Basado en Veloren (GPL-3.0)",
        );
        draw_text_right(
            &mut pm,
            &self.body_font,
            11.0,
            W - 28.0,
            630.0,
            dim,
            concat!("Lanzador v", env!("CARGO_PKG_VERSION")),
        );
        pm
    }
}

/// Estado de la interacción con el ratón
#[derive(Copy, Clone, Default)]
pub struct Pointer {
    pub hovered: Option<Hit>,
    pub pressed: Option<Hit>,
}

/// Dibuja un fotograma completo. `time` son los segundos desde que se abrió
/// la ventana, para las animaciones.
pub fn render(res: &Resources, st: &AppState, pointer: Pointer, time: f32, pm: &mut Pixmap) {
    pm.data_mut().copy_from_slice(res.background.data());
    title_buttons(pm, pointer);
    news_panel(res, pm, st.news.as_deref());
    info_panel(res, pm, st);
    status_and_progress(res, pm, st, time);
    for (hit, label) in [
        (Hit::CheckUpdates, "Buscar actualizaciones"),
        (Hit::OpenFolder, "Abrir carpeta"),
        (Hit::Website, "Ver en GitHub"),
    ] {
        secondary_button(res, pm, hit, label, pointer);
    }
    play_button(res, pm, st, pointer, time);
}

fn title_buttons(pm: &mut Pixmap, pointer: Pointer) {
    for hit in [Hit::Minimize, Hit::Close] {
        let (x, y, w, h) = hit_rect(hit);
        if pointer.hovered == Some(hit) {
            let color = if hit == Hit::Close {
                [196, 43, 28, 255]
            } else {
                [255, 255, 255, 22]
            };
            fill_rect(pm, x, y, w, h - 1.0, solid(color));
        }
        let (cx, cy) = (x + w / 2.0, y + h / 2.0);
        let mut pb = PathBuilder::new();
        if hit == Hit::Close {
            pb.move_to(cx - 5.0, cy - 5.0);
            pb.line_to(cx + 5.0, cy + 5.0);
            pb.move_to(cx + 5.0, cy - 5.0);
            pb.line_to(cx - 5.0, cy + 5.0);
        } else {
            pb.move_to(cx - 5.5, cy);
            pb.line_to(cx + 5.5, cy);
        }
        if let Some(path) = pb.finish() {
            stroke_path(pm, &path, [226, 232, 240, 230], 1.3);
        }
    }
}

fn news_panel(res: &Resources, pm: &mut Pixmap, news: Option<&[NewsItem]>) {
    let Some(news) = news else {
        draw_text(
            pm,
            &res.body_font,
            13.5,
            NEWS_X + 20.0,
            PANEL_Y + 62.0,
            MUTED,
            "Cargando novedades...",
        );
        return;
    };
    for (i, item) in news.iter().take(4).enumerate() {
        let baseline = PANEL_Y + 58.0 + i as f32 * 22.0;
        // Rombo dorado como viñeta
        let (bx, by) = (NEWS_X + 24.0, baseline - 4.5);
        let mut pb = PathBuilder::new();
        pb.move_to(bx, by - 3.5);
        pb.line_to(bx + 3.5, by);
        pb.line_to(bx, by + 3.5);
        pb.line_to(bx - 3.5, by);
        pb.close();
        if let Some(path) = pb.finish() {
            fill_path(pm, &path, solid(GOLD_DARK));
        }
        let mut text_x = NEWS_X + 36.0;
        if !item.date.is_empty() {
            draw_text(pm, &res.body_font, 12.5, text_x, baseline, MUTED, &item.date);
            text_x += 46.0;
        }
        let max_w = NEWS_X + NEWS_W - 18.0 - text_x;
        let text = ellipsize(&res.body_font, 13.5, &item.text, max_w);
        draw_text(pm, &res.body_font, 13.5, text_x, baseline, TEXT, &text);
    }
}

fn info_panel(res: &Resources, pm: &mut Pixmap, st: &AppState) {
    let (connection, connection_color) = match st.status {
        UpdateStatus::Checking => ("Comprobando...", BLUE),
        UpdateStatus::Offline => ("Sin conexión", MUTED),
        UpdateStatus::Error(_) => ("Con errores", RED),
        _ => ("En línea", GREEN),
    };
    let rows: [(&str, &str, [u8; 4]); 4] = [
        ("Versión instalada", &st.local_version, TEXT),
        ("Última versión", &st.remote_version, TEXT),
        ("Servidor de actualizaciones", connection, connection_color),
        ("Mundo", "Azeria · 4 continentes", TEXT),
    ];
    let right = INFO_X + INFO_W - 18.0;
    for (i, (label, value, color)) in rows.iter().enumerate() {
        let baseline = PANEL_Y + 58.0 + i as f32 * 22.0;
        draw_text(pm, &res.body_font, 13.0, INFO_X + 18.0, baseline, MUTED, label);
        let value = ellipsize(&res.body_font, 13.0, value, 170.0);
        let value_x = draw_text_right(pm, &res.body_font, 13.0, right, baseline, *color, &value);
        if i == 2 {
            // Punto de color del estado de la conexión
            let path = PathBuilder::from_circle(value_x - 9.0, baseline - 4.5, 3.5);
            if let Some(path) = path {
                fill_path(pm, &path, solid(*color));
            }
        }
    }
}

fn status_and_progress(res: &Resources, pm: &mut Pixmap, st: &AppState, time: f32) {
    let no_game = st.game_binary.is_none();
    let (text, color): (String, [u8; 4]) = match &st.status {
        UpdateStatus::Checking => ("Buscando actualizaciones...".into(), BLUE),
        UpdateStatus::UpdateAvailable { remote_tag, .. } => {
            (format!("Nueva versión disponible: {remote_tag}"), GOLD)
        },
        UpdateStatus::Downloading { .. } => ("Descargando actualización...".into(), GOLD),
        UpdateStatus::Extracting => ("Instalando archivos del juego...".into(), GOLD),
        UpdateStatus::Error(err) => (format!("Error: {err}"), RED),
        _ if no_game => (
            "No se encontró el juego en esta carpeta.".into(),
            RED,
        ),
        UpdateStatus::Offline => (
            "Sin conexión con el servidor de actualizaciones. Puedes jugar igualmente.".into(),
            MUTED,
        ),
        UpdateStatus::UpToDate => ("Todo listo. Tu juego está actualizado.".into(), GREEN),
    };
    let text = ellipsize(&res.body_font, 14.5, &text, PROG_W - 230.0);
    draw_text(pm, &res.body_font, 14.5, PROG_X, STATUS_Y, color, &text);

    // Detalle a la derecha: tamaño, velocidad y tiempo restante
    let detail = match &st.status {
        UpdateStatus::Downloading {
            downloaded,
            total,
            speed,
        } => {
            let mut detail = format!("{} / {}", format_mb(*downloaded), format_mb(*total));
            if *speed > 1.0 {
                detail.push_str(&format!("  ·  {}/s", format_mb(*speed as u64)));
                if *total > *downloaded {
                    let secs = ((*total - *downloaded) as f64 / speed) as u64;
                    detail.push_str(&format!("  ·  {:02}:{:02}", secs / 60, secs % 60));
                }
            }
            detail
        },
        UpdateStatus::UpdateAvailable { total_bytes, .. } if *total_bytes > 0 => {
            format!("Tamaño: {}", format_mb(*total_bytes))
        },
        _ => String::new(),
    };
    draw_text_right(pm, &res.body_font, 12.5, PROG_X + PROG_W, STATUS_Y, MUTED, &detail);

    // Barra de progreso
    if let Some(track) = rounded_rect(PROG_X, PROG_Y, PROG_W, PROG_H, PROG_H / 2.0) {
        fill_path(pm, &track, solid([255, 255, 255, 14]));
        stroke_path(pm, &track, [255, 255, 255, 22], 1.0);
    }
    let gold_fill = |x0: f32, x1: f32| {
        hgrad(x0, x1, &[(0.0, [168, 116, 31, 255]), (1.0, [243, 213, 138, 255])])
    };
    match &st.status {
        // Indeterminada: un tramo azul que va y viene
        UpdateStatus::Checking => {
            let seg = PROG_W * 0.28;
            let phase = (time * 1.4).sin() * 0.5 + 0.5;
            let x = PROG_X + phase * (PROG_W - seg);
            if let Some(path) = rounded_rect(x, PROG_Y, seg, PROG_H, PROG_H / 2.0) {
                fill_path(pm, &path, hgrad(x, x + seg, &[
                    (0.0, [60, 120, 255, 0]),
                    (0.5, [127, 179, 255, 255]),
                    (1.0, [60, 120, 255, 0]),
                ]));
            }
        },
        status => {
            let (fraction, shader) = match status {
                UpdateStatus::Downloading {
                    downloaded, total, ..
                } if *total > 0 => (
                    *downloaded as f32 / *total as f32,
                    gold_fill(PROG_X, PROG_X + PROG_W),
                ),
                UpdateStatus::Downloading { .. } | UpdateStatus::Extracting => {
                    (1.0, gold_fill(PROG_X, PROG_X + PROG_W))
                },
                UpdateStatus::Error(_) => (1.0, solid([185, 28, 28, 200])),
                UpdateStatus::UpdateAvailable { .. } => (0.0, solid([0, 0, 0, 0])),
                UpdateStatus::Offline => (1.0, solid([147, 164, 189, 90])),
                _ => (1.0, hgrad(PROG_X, PROG_X + PROG_W, &[
                    (0.0, [47, 133, 90, 255]),
                    (1.0, [111, 208, 140, 255]),
                ])),
            };
            let fill_w = (PROG_W * fraction.clamp(0.0, 1.0)).max(0.0);
            if fill_w >= PROG_H
                && let Some(path) = rounded_rect(PROG_X, PROG_Y, fill_w, PROG_H, PROG_H / 2.0)
            {
                fill_path(pm, &path, shader);
                // Brillo que recorre la barra mientras se descarga o instala
                if matches!(
                    status,
                    UpdateStatus::Downloading { .. } | UpdateStatus::Extracting
                ) {
                    let shine_x = PROG_X + (time * 0.45).fract() * (fill_w + 120.0) - 60.0;
                    fill_path(pm, &path, hgrad(shine_x - 60.0, shine_x + 60.0, &[
                        (0.0, [255, 255, 255, 0]),
                        (0.5, [255, 255, 255, 110]),
                        (1.0, [255, 255, 255, 0]),
                    ]));
                }
            }
        },
    }
}

fn secondary_button(res: &Resources, pm: &mut Pixmap, hit: Hit, label: &str, pointer: Pointer) {
    let (x, y, w, h) = hit_rect(hit);
    let hovered = pointer.hovered == Some(hit);
    let pressed = hovered && pointer.pressed == Some(hit);
    if let Some(path) = rounded_rect(x, y, w, h, 6.0) {
        let fill = if pressed {
            [255, 255, 255, 8]
        } else if hovered {
            [255, 255, 255, 26]
        } else {
            [255, 255, 255, 12]
        };
        fill_path(pm, &path, solid(fill));
        stroke_path(
            pm,
            &path,
            [216, 169, 72, if hovered { 190 } else { 85 }],
            1.0,
        );
    }
    let color = if hovered { GOLD } else { TEXT };
    draw_text_centered(pm, &res.body_font, 13.0, x + w / 2.0, y + h / 2.0 + 4.5, color, label);
}

fn play_button(res: &Resources, pm: &mut Pixmap, st: &AppState, pointer: Pointer, time: f32) {
    let (x, y, w, h) = hit_rect(Hit::Play);
    let hovered = pointer.hovered == Some(Hit::Play);
    let pressed = hovered && pointer.pressed == Some(Hit::Play);

    let (label, sublabel, enabled) = match &st.status {
        UpdateStatus::UpdateAvailable {
            download_url: Some(_),
            ..
        } => ("ACTUALIZAR", String::new(), true),
        UpdateStatus::Downloading {
            downloaded, total, ..
        } => {
            let pct = if *total > 0 {
                format!("{:.0}%", *downloaded as f32 / *total as f32 * 100.0)
            } else {
                String::new()
            };
            ("DESCARGANDO", pct, false)
        },
        UpdateStatus::Extracting => ("INSTALANDO", String::new(), false),
        _ if st.game_binary.is_none() => ("NO DISPONIBLE", String::new(), false),
        _ => ("JUGAR", String::new(), true),
    };

    // Halo dorado: late suavemente cuando el botón está listo y crece al pasar el ratón
    if enabled {
        let pulse = if hovered {
            1.0
        } else {
            0.55 + 0.45 * ((time * 2.2).sin() * 0.5 + 0.5)
        };
        for (grow, alpha) in [(12.0, 18.0), (7.0, 30.0), (3.0, 45.0)] {
            if let Some(path) =
                rounded_rect(x - grow, y - grow, w + grow * 2.0, h + grow * 2.0, 10.0 + grow)
            {
                fill_path(pm, &path, solid([233, 180, 70, (alpha * pulse) as u8]));
            }
        }
    }

    let Some(body) = rounded_rect(x, y, w, h, 10.0) else {
        return;
    };
    let stops: [(f32, [u8; 4]); 3] = if !enabled {
        [
            (0.0, [70, 78, 94, 255]),
            (0.5, [52, 58, 72, 255]),
            (1.0, [38, 43, 54, 255]),
        ]
    } else if pressed {
        [
            (0.0, [214, 170, 88, 255]),
            (0.5, [184, 132, 48, 255]),
            (1.0, [140, 96, 26, 255]),
        ]
    } else if hovered {
        [
            (0.0, [255, 236, 170, 255]),
            (0.5, [236, 185, 86, 255]),
            (1.0, [190, 136, 42, 255]),
        ]
    } else {
        [
            (0.0, [247, 220, 138, 255]),
            (0.5, [217, 164, 65, 255]),
            (1.0, [168, 116, 31, 255]),
        ]
    };
    fill_path(pm, &body, vgrad(y, y + h, &stops));
    // Reflejo en la mitad superior
    if let Some(gloss) = rounded_rect(x + 3.0, y + 3.0, w - 6.0, h / 2.0 - 3.0, 8.0) {
        fill_path(pm, &gloss, vgrad(y, y + h / 2.0, &[
            (0.0, [255, 255, 255, if enabled { 70 } else { 25 }]),
            (1.0, [255, 255, 255, 0]),
        ]));
    }
    stroke_path(
        pm,
        &body,
        if enabled {
            [255, 240, 200, 220]
        } else {
            [110, 120, 140, 200]
        },
        1.5,
    );

    let text_color = if enabled { [43, 24, 5, 255] } else { [180, 188, 204, 255] };
    let size = if label.len() > 8 { 22.0 } else { 32.0 };
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    if sublabel.is_empty() {
        draw_text_centered(pm, &res.title_font, size, cx, cy + size * 0.36, text_color, label);
    } else {
        draw_text_centered(pm, &res.title_font, size, cx, cy + 2.0, text_color, label);
        draw_text_centered(pm, &res.body_font, 14.0, cx, cy + 24.0, text_color, &sublabel);
    }
}

// ----------------------------------------------------------------------------
// Utilidades de dibujo
// ----------------------------------------------------------------------------

fn rgba(c: [u8; 4]) -> Color { Color::from_rgba8(c[0], c[1], c[2], c[3]) }

fn solid(c: [u8; 4]) -> Shader<'static> { Shader::SolidColor(rgba(c)) }

fn gradient(start: Point, end: Point, stops: &[(f32, [u8; 4])]) -> Shader<'static> {
    LinearGradient::new(
        start,
        end,
        stops
            .iter()
            .map(|(pos, c)| GradientStop::new(*pos, rgba(*c)))
            .collect(),
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap_or_else(|| solid(stops[0].1))
}

/// Degradado vertical entre `y0` e `y1`
fn vgrad(y0: f32, y1: f32, stops: &[(f32, [u8; 4])]) -> Shader<'static> {
    gradient(Point::from_xy(0.0, y0), Point::from_xy(0.0, y1), stops)
}

/// Degradado horizontal entre `x0` y `x1`
fn hgrad(x0: f32, x1: f32, stops: &[(f32, [u8; 4])]) -> Shader<'static> {
    gradient(Point::from_xy(x0, 0.0), Point::from_xy(x1, 0.0), stops)
}

fn radial(cx: f32, cy: f32, radius: f32, stops: &[(f32, [u8; 4])]) -> Shader<'static> {
    let center = Point::from_xy(cx, cy);
    RadialGradient::new(
        center,
        center,
        radius,
        stops
            .iter()
            .map(|(pos, c)| GradientStop::new(*pos, rgba(*c)))
            .collect(),
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap_or_else(|| solid(stops[0].1))
}

fn paint(shader: Shader<'_>) -> Paint<'_> {
    Paint {
        shader,
        anti_alias: true,
        ..Default::default()
    }
}

fn fill_rect(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, shader: Shader<'_>) {
    if let Some(rect) = Rect::from_xywh(x, y, w, h) {
        pm.fill_rect(rect, &paint(shader), Transform::identity(), None);
    }
}

fn fill_path(pm: &mut Pixmap, path: &Path, shader: Shader<'_>) {
    pm.fill_path(path, &paint(shader), FillRule::Winding, Transform::identity(), None);
}

fn stroke_path(pm: &mut Pixmap, path: &Path, color: [u8; 4], width: f32) {
    let stroke = Stroke {
        width,
        ..Default::default()
    };
    pm.stroke_path(path, &paint(solid(color)), &stroke, Transform::identity(), None);
}

/// Rectángulo con esquinas redondeadas (curvas cúbicas)
fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<Path> {
    let r = r.min(w / 2.0).min(h / 2.0);
    // Distancia de los puntos de control para aproximar un cuarto de círculo
    let k = r * 0.552_284_8;
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish()
}

// ----------------------------------------------------------------------------
// Texto
// ----------------------------------------------------------------------------

fn text_width(font: &FontRef<'_>, size: f32, text: &str) -> f32 {
    let scaled = font.as_scaled(PxScale::from(size));
    let mut width = 0.0;
    let mut prev = None;
    for c in text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(prev) = prev {
            width += scaled.kern(prev, id);
        }
        width += scaled.h_advance(id);
        prev = Some(id);
    }
    width
}

/// Recorta el texto con "..." para que quepa en `max_w`
fn ellipsize(font: &FontRef<'_>, size: f32, text: &str, max_w: f32) -> String {
    if text_width(font, size, text) <= max_w {
        return text.to_string();
    }
    let mut out = String::new();
    for c in text.chars() {
        out.push(c);
        if text_width(font, size, &format!("{out}...")) > max_w {
            out.pop();
            break;
        }
    }
    format!("{}...", out.trim_end())
}

/// Dibuja texto con la línea base en `baseline` y devuelve dónde termina
fn draw_text(
    pm: &mut Pixmap,
    font: &FontRef<'_>,
    size: f32,
    x: f32,
    baseline: f32,
    color: [u8; 4],
    text: &str,
) -> f32 {
    let scale = PxScale::from(size);
    let scaled = font.as_scaled(scale);
    let (width, height) = (pm.width() as i32, pm.height() as i32);
    let data = pm.data_mut();
    let alpha = color[3] as f32 / 255.0;

    let mut caret = x;
    let mut prev = None;
    for c in text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(prev) = prev {
            caret += scaled.kern(prev, id);
        }
        let glyph = id.with_scale_and_position(scale, point(caret, baseline));
        caret += scaled.h_advance(id);
        prev = Some(id);

        let Some(outlined) = font.outline_glyph(glyph) else {
            continue;
        };
        let bounds = outlined.px_bounds();
        outlined.draw(|gx, gy, coverage| {
            let px = bounds.min.x as i32 + gx as i32;
            let py = bounds.min.y as i32 + gy as i32;
            if px < 0 || py < 0 || px >= width || py >= height {
                return;
            }
            let a = coverage.clamp(0.0, 1.0) * alpha;
            if a <= 0.0 {
                return;
            }
            // Mezcla sobre píxeles con alfa premultiplicado
            let i = ((py * width + px) * 4) as usize;
            for k in 0..3 {
                let blended = color[k] as f32 * a + data[i + k] as f32 * (1.0 - a);
                data[i + k] = blended.round().clamp(0.0, 255.0) as u8;
            }
            let blended = 255.0 * a + data[i + 3] as f32 * (1.0 - a);
            data[i + 3] = blended.round().clamp(0.0, 255.0) as u8;
        });
    }
    caret
}

/// Texto alineado a la derecha en `right`; devuelve dónde empieza
fn draw_text_right(
    pm: &mut Pixmap,
    font: &FontRef<'_>,
    size: f32,
    right: f32,
    baseline: f32,
    color: [u8; 4],
    text: &str,
) -> f32 {
    let x = right - text_width(font, size, text);
    draw_text(pm, font, size, x, baseline, color, text);
    x
}

fn draw_text_centered(
    pm: &mut Pixmap,
    font: &FontRef<'_>,
    size: f32,
    center: f32,
    baseline: f32,
    color: [u8; 4],
    text: &str,
) {
    let x = center - text_width(font, size, text) / 2.0;
    draw_text(pm, font, size, x, baseline, color, text);
}

fn format_mb(bytes: u64) -> String { format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) }
