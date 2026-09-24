use crate::comp::body::humanoid::Species;
use serde::{Deserialize, Serialize};
use vek::*;

/// Facciones jugables del MMORPG
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionId {
    Alliance,
    Horde,
}

impl FactionId {
    pub fn name(&self) -> &'static str {
        match self {
            FactionId::Alliance => "Alianza",
            FactionId::Horde => "Horda",
        }
    }

    /// Determina la facción a partir de la raza/especie del personaje
    pub fn from_species(species: Species) -> Self {
        match species {
            Species::Human | Species::Dwarf | Species::Elf => FactionId::Alliance,
            Species::Orc | Species::Draugr | Species::Danari => FactionId::Horde,
        }
    }
}

/// Zonas del mundo: cada una de ellas es uno de los 4 continentes
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneId {
    AllianceContinent,
    HordeContinent,
    ContestedContinent,
    HighLevelContinent,
}

/// Definición de una zona de aventura con rangos de nivel y puntos de spawn
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZoneDefinition {
    pub id: ZoneId,
    pub name: &'static str,
    pub level_min: u8,
    pub level_max: u8,
    pub faction: Option<FactionId>,
    /// Centro del continente. Cada punto del mundo pertenece al continente
    /// cuyo centro tiene más cerca, igual que la máscara que los generó.
    pub center_wpos: Vec2<i32>,
    pub graveyard_wpos: Vec2<i32>,
}

/// Tamaño en bloques del mapa por defecto (`world.map.azeria_4continentes`,
/// 2048 x 2048 chunks de 32 bloques)
pub const WORLD_SIZE_BLOCKS: i32 = 65536;

/// Centro de un continente a partir de su posición relativa en el mapa. Son las
/// mismas posiciones que usa la máscara de continentes al generar el mundo
/// (`continent_centers(4)` en `world/src/sim/util.rs`).
const fn continent_center(x_frac: f32, y_frac: f32) -> Vec2<i32> {
    Vec2::new(
        (x_frac * WORLD_SIZE_BLOCKS as f32) as i32,
        (y_frac * WORLD_SIZE_BLOCKS as f32) as i32,
    )
}

// En el mapa (norte arriba): Alianza al sureste, Horda al noreste, Tierras
// Disputadas al suroeste y el continente de alto nivel al noroeste.
pub const ALLIANCE_CAPITAL_WPOS: Vec2<i32> = continent_center(0.74, 0.24);
pub const HORDE_CAPITAL_WPOS: Vec2<i32> = continent_center(0.76, 0.76);
pub const CONTESTED_CENTER_WPOS: Vec2<i32> = continent_center(0.26, 0.27);
pub const HIGH_LEVEL_CENTER_WPOS: Vec2<i32> = continent_center(0.24, 0.73);

pub const ZONES: [ZoneDefinition; 4] = [
    ZoneDefinition {
        id: ZoneId::AllianceContinent,
        name: "Reinos de Valdoria",
        level_min: 1,
        level_max: 20,
        faction: Some(FactionId::Alliance),
        center_wpos: ALLIANCE_CAPITAL_WPOS,
        graveyard_wpos: ALLIANCE_CAPITAL_WPOS,
    },
    ZoneDefinition {
        id: ZoneId::HordeContinent,
        name: "Dominios de Kargath",
        level_min: 1,
        level_max: 20,
        faction: Some(FactionId::Horde),
        center_wpos: HORDE_CAPITAL_WPOS,
        graveyard_wpos: HORDE_CAPITAL_WPOS,
    },
    ZoneDefinition {
        id: ZoneId::ContestedContinent,
        name: "Tierras Disputadas de Morvenia",
        level_min: 20,
        level_max: 40,
        faction: None,
        center_wpos: CONTESTED_CENTER_WPOS,
        graveyard_wpos: CONTESTED_CENTER_WPOS,
    },
    ZoneDefinition {
        id: ZoneId::HighLevelContinent,
        name: "Picos Helados de Nordheim",
        level_min: 40,
        level_max: 60,
        faction: None,
        center_wpos: HIGH_LEVEL_CENTER_WPOS,
        graveyard_wpos: HIGH_LEVEL_CENTER_WPOS,
    },
];

/// Devuelve el centro del continente de la facción, usado como aparición de
/// respaldo si no se encuentra su pueblo inicial
pub fn get_faction_spawn_wpos(faction: FactionId) -> Vec2<i32> {
    match faction {
        FactionId::Alliance => ALLIANCE_CAPITAL_WPOS,
        FactionId::Horde => HORDE_CAPITAL_WPOS,
    }
}

/// Información del hogar y asentamiento natal de cada raza jugable
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaceStartingInfo {
    pub species: Species,
    pub race_name: &'static str,
    pub homeland_name: &'static str,
    pub town_name: &'static str,
    pub faction: FactionId,
    pub spawn_wpos: Vec2<i32>,
    pub graveyard_wpos: Vec2<i32>,
}

pub const RACE_STARTING_INFOS: [RaceStartingInfo; 6] = [
    // Alianza
    RaceStartingInfo {
        species: Species::Human,
        race_name: "Humano",
        homeland_name: "Valle de Bosquedorado",
        town_name: "Villa Bosquedorado",
        faction: FactionId::Alliance,
        spawn_wpos: ALLIANCE_CAPITAL_WPOS,
        graveyard_wpos: ALLIANCE_CAPITAL_WPOS,
    },
    RaceStartingInfo {
        species: Species::Dwarf,
        race_name: "Enano",
        homeland_name: "Picos de Dun Kahr",
        town_name: "Fortaleza Forjahierro",
        faction: FactionId::Alliance,
        spawn_wpos: Vec2::new(ALLIANCE_CAPITAL_WPOS.x - 3000, ALLIANCE_CAPITAL_WPOS.y + 2500),
        graveyard_wpos: Vec2::new(ALLIANCE_CAPITAL_WPOS.x - 3000, ALLIANCE_CAPITAL_WPOS.y + 2500),
    },
    RaceStartingInfo {
        species: Species::Elf,
        race_name: "Elfo",
        homeland_name: "Bosque Místico de Elveron",
        town_name: "Arboleda de las Estrellas",
        faction: FactionId::Alliance,
        spawn_wpos: Vec2::new(ALLIANCE_CAPITAL_WPOS.x + 3000, ALLIANCE_CAPITAL_WPOS.y + 2500),
        graveyard_wpos: Vec2::new(ALLIANCE_CAPITAL_WPOS.x + 3000, ALLIANCE_CAPITAL_WPOS.y + 2500),
    },
    // Horda
    RaceStartingInfo {
        species: Species::Orc,
        race_name: "Orco",
        homeland_name: "Valle Quebrantahuesos",
        town_name: "Bastión Ogron",
        faction: FactionId::Horde,
        spawn_wpos: HORDE_CAPITAL_WPOS,
        graveyard_wpos: HORDE_CAPITAL_WPOS,
    },
    RaceStartingInfo {
        species: Species::Draugr,
        race_name: "Draugr",
        homeland_name: "Criptas de la Desolación",
        town_name: "Sepulcro Sombrío",
        faction: FactionId::Horde,
        spawn_wpos: Vec2::new(HORDE_CAPITAL_WPOS.x + 3000, HORDE_CAPITAL_WPOS.y - 2500),
        graveyard_wpos: Vec2::new(HORDE_CAPITAL_WPOS.x + 3000, HORDE_CAPITAL_WPOS.y - 2500),
    },
    RaceStartingInfo {
        species: Species::Danari,
        race_name: "Danari",
        homeland_name: "Oasis del Sol Silencioso",
        town_name: "Santuario de las Arenas",
        faction: FactionId::Horde,
        spawn_wpos: Vec2::new(HORDE_CAPITAL_WPOS.x - 3000, HORDE_CAPITAL_WPOS.y - 2500),
        graveyard_wpos: Vec2::new(HORDE_CAPITAL_WPOS.x - 3000, HORDE_CAPITAL_WPOS.y - 2500),
    },
];

/// Obtiene la información del lugar de nacimiento natal de una raza
pub fn get_race_starting_info(species: Species) -> &'static RaceStartingInfo {
    match species {
        Species::Human => &RACE_STARTING_INFOS[0],
        Species::Dwarf => &RACE_STARTING_INFOS[1],
        Species::Elf => &RACE_STARTING_INFOS[2],
        Species::Orc => &RACE_STARTING_INFOS[3],
        Species::Draugr => &RACE_STARTING_INFOS[4],
        Species::Danari => &RACE_STARTING_INFOS[5],
    }
}

/// Obtiene la posición de spawn inicial de una raza
pub fn get_species_spawn_wpos(species: Species) -> Vec2<i32> {
    get_race_starting_info(species).spawn_wpos
}

/// Obtiene el índice determinista del sitio inicial asignado a la raza
pub fn species_starting_site_index(species: Species, total_sites: usize) -> usize {
    if total_sites == 0 {
        return 0;
    }
    let idx = match species {
        Species::Human => 0,
        Species::Dwarf => 1,
        Species::Elf => 2,
        Species::Orc => 3,
        Species::Draugr => 4,
        Species::Danari => 5,
    };
    idx % total_sites
}

/// Devuelve la zona (el continente) en la que se encuentra una coordenada del
/// mundo: la del centro de continente más cercano
pub fn get_zone_at(wpos: Vec2<f32>) -> &'static ZoneDefinition {
    ZONES
        .iter()
        .min_by(|a, b| {
            let da = a.center_wpos.as_::<f32>().distance_squared(wpos);
            let db = b.center_wpos.as_::<f32>().distance_squared(wpos);
            da.partial_cmp(&db).unwrap_or(core::cmp::Ordering::Equal)
        })
        .expect("Siempre hay al menos una zona")
}

/// Radio de protección en bloques/metros para las zonas seguras de inicio y asentamientos
pub const SAFE_ZONE_RADIUS: f32 = 250.0;

lazy_static::lazy_static! {
    static ref DYNAMIC_SAFE_ZONES: std::sync::RwLock<Vec<Vec2<i32>>> = std::sync::RwLock::new(Vec::new());
}

/// Registra una zona segura dinámica (ej. el pueblo natal generado en el mundo)
pub fn register_safe_zone(center: Vec2<i32>) {
    if let Ok(mut zones) = DYNAMIC_SAFE_ZONES.write() {
        if !zones.contains(&center) {
            zones.push(center);
        }
    }
}

/// Determina si una coordenada se encuentra protegida dentro de una zona segura
pub fn is_in_safe_zone(wpos2d: Vec2<i32>) -> bool {
    nearest_safe_zone_with_margin(wpos2d, 0.0).is_some()
}

/// Devuelve el centro y distancia a la zona segura más cercana si está dentro del radio + margen
pub fn nearest_safe_zone_with_margin(wpos2d: Vec2<i32>, margin: f32) -> Option<(Vec2<i32>, f32)> {
    let wpos_f = wpos2d.map(|e| e as f32);
    let max_dist = SAFE_ZONE_RADIUS + margin;
    let mut nearest = None;
    let mut min_dist = max_dist;

    // 1. Zonas de spawn de cada raza
    for info in &RACE_STARTING_INFOS {
        let spawn_f = info.spawn_wpos.map(|e| e as f32);
        let dist = wpos_f.distance(spawn_f);
        if dist <= min_dist {
            min_dist = dist;
            nearest = Some((info.spawn_wpos, dist));
        }
    }

    // 2. Capitales de facción
    for capital in &[ALLIANCE_CAPITAL_WPOS, HORDE_CAPITAL_WPOS] {
        let cap_f = capital.map(|e| e as f32);
        let dist = wpos_f.distance(cap_f);
        if dist <= min_dist {
            min_dist = dist;
            nearest = Some((*capital, dist));
        }
    }

    // 3. Asentamientos y pueblos iniciales dinámicos
    if let Ok(dyn_zones) = DYNAMIC_SAFE_ZONES.read() {
        for &center in dyn_zones.iter() {
            let center_f = center.map(|e| e as f32);
            let dist = wpos_f.distance(center_f);
            if dist <= min_dist {
                min_dist = dist;
                nearest = Some((center, dist));
            }
        }
    }

    nearest
}

/// Devuelve el centro y distancia a la zona segura más cercana si está dentro del radio normal
pub fn nearest_safe_zone_center(wpos2d: Vec2<i32>) -> Option<(Vec2<i32>, f32)> {
    nearest_safe_zone_with_margin(wpos2d, 0.0)
}

/// Devuelve la distancia al centro de la zona segura o ciudad de inicio más cercana
pub fn distance_to_nearest_safe_zone(wpos2d: Vec2<i32>) -> f32 {
    let wpos_f = wpos2d.map(|e| e as f32);
    let mut min_dist = f32::MAX;

    // 1. Zonas de spawn de cada raza
    for info in &RACE_STARTING_INFOS {
        let dist = wpos_f.distance(info.spawn_wpos.map(|e| e as f32));
        if dist < min_dist {
            min_dist = dist;
        }
    }

    // 2. Capitales de facción
    for capital in &[ALLIANCE_CAPITAL_WPOS, HORDE_CAPITAL_WPOS] {
        let dist = wpos_f.distance(capital.map(|e| e as f32));
        if dist < min_dist {
            min_dist = dist;
        }
    }

    // 3. Asentamientos y pueblos dinámicos
    if let Ok(dyn_zones) = DYNAMIC_SAFE_ZONES.read() {
        for &center in dyn_zones.iter() {
            let dist = wpos_f.distance(center.map(|e| e as f32));
            if dist < min_dist {
                min_dist = dist;
            }
        }
    }

    min_dist
}

/// Calcula el factor de escala de dificultad y poder (0.25 a 1.0+) para mobs hostiles
/// según la distancia a la ciudad o zona segura más cercana.
///
/// Progresión de niveles alrededor de ciudades:
/// - Inmediaciones (0m a 300m del límite de la ciudad):
///   Nivel 1 a 2 (factor ~0.25 - 0.35): vida y daño muy reducidos para no morir de un golpe.
/// - Alrededores cercanos (300m a 700m del límite):
///   Nivel 2 a 4 (factor ~0.35 - 0.55).
/// - Zona intermedia (700m a 1200m del límite):
///   Nivel 4 a 6 (factor ~0.55 - 0.75).
/// - Tierras salvajes lejanas (> 1200m del límite):
///   Nivel 7+ hasta el nivel estándar del área (factor 0.85 - 1.0+).
pub fn mob_difficulty_scaling_at(wpos2d: Vec2<i32>) -> f32 {
    let dist = distance_to_nearest_safe_zone(wpos2d);
    if dist <= SAFE_ZONE_RADIUS {
        return 0.25;
    }

    let dist_from_border = dist - SAFE_ZONE_RADIUS;

    if dist_from_border < 300.0 {
        // Inmediaciones de la ciudad: Nivel 1 a 2
        0.25 + (dist_from_border / 300.0) * 0.10
    } else if dist_from_border < 700.0 {
        // Alrededores cercanos: Nivel 2 a 4
        0.35 + ((dist_from_border - 300.0) / 400.0) * 0.20
    } else if dist_from_border < 1200.0 {
        // Zona intermedia: Nivel 4 a 6
        0.55 + ((dist_from_border - 700.0) / 500.0) * 0.20
    } else if dist_from_border < 1800.0 {
        // Transición a tierras salvajes: Nivel 6 a 7
        0.75 + ((dist_from_border - 1200.0) / 600.0) * 0.25
    } else {
        // Tierras salvajes: poder normal 1.0
        1.0
    }
}

