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

/// Identificadores de las Zonas del Mundo Estático
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneId {
    AllianceCapital,
    AllianceStartingZone,
    HordeCapital,
    HordeStartingZone,
    ContestedWilds,
}

/// Definición de una zona de aventura con rangos de nivel y puntos de spawn
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZoneDefinition {
    pub id: ZoneId,
    pub name: &'static str,
    pub level_min: u8,
    pub level_max: u8,
    pub faction: Option<FactionId>,
    pub center_wpos: Vec2<i32>,
    pub radius: f32,
    pub graveyard_wpos: Vec2<i32>,
}

// Coordenadas base en el mapa estático (1024x1024 chunks -> 32768 x 32768 bloques)
// Ubicadas en cuadrantes de llanura/valle con terreno estable y plano.
pub const ALLIANCE_CAPITAL_WPOS: Vec2<i32> = Vec2::new(14000, 14000);
pub const ALLIANCE_GRAVEYARD_WPOS: Vec2<i32> = Vec2::new(14020, 14010);

pub const HORDE_CAPITAL_WPOS: Vec2<i32> = Vec2::new(18000, 18000);
pub const HORDE_GRAVEYARD_WPOS: Vec2<i32> = Vec2::new(18020, 18010);

pub const ZONES: [ZoneDefinition; 5] = [
    ZoneDefinition {
        id: ZoneId::AllianceCapital,
        name: "Ciudadela de la Luz (Capital)",
        level_min: 1,
        level_max: 60,
        faction: Some(FactionId::Alliance),
        center_wpos: ALLIANCE_CAPITAL_WPOS,
        radius: 1200.0,
        graveyard_wpos: ALLIANCE_GRAVEYARD_WPOS,
    },
    ZoneDefinition {
        id: ZoneId::AllianceStartingZone,
        name: "Valle Dorado",
        level_min: 1,
        level_max: 10,
        faction: Some(FactionId::Alliance),
        center_wpos: ALLIANCE_CAPITAL_WPOS,
        radius: 3500.0,
        graveyard_wpos: ALLIANCE_GRAVEYARD_WPOS,
    },
    ZoneDefinition {
        id: ZoneId::HordeCapital,
        name: "Bastión de Ogron (Capital)",
        level_min: 1,
        level_max: 60,
        faction: Some(FactionId::Horde),
        center_wpos: HORDE_CAPITAL_WPOS,
        radius: 1200.0,
        graveyard_wpos: HORDE_GRAVEYARD_WPOS,
    },
    ZoneDefinition {
        id: ZoneId::HordeStartingZone,
        name: "Estepas Áridas",
        level_min: 1,
        level_max: 10,
        faction: Some(FactionId::Horde),
        center_wpos: HORDE_CAPITAL_WPOS,
        radius: 3500.0,
        graveyard_wpos: HORDE_GRAVEYARD_WPOS,
    },
    ZoneDefinition {
        id: ZoneId::ContestedWilds,
        name: "Tierras Disputadas",
        level_min: 10,
        level_max: 25,
        faction: None,
        center_wpos: Vec2::new(16000, 16000),
        radius: 8000.0,
        graveyard_wpos: Vec2::new(16000, 16000),
    },
];

/// Devuelve la posición inicial plana en la plaza principal de la facción correspondiente
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
        spawn_wpos: Vec2::new(14000, 14000),
        graveyard_wpos: Vec2::new(14020, 14010),
    },
    RaceStartingInfo {
        species: Species::Dwarf,
        race_name: "Enano",
        homeland_name: "Picos de Dun Kahr",
        town_name: "Fortaleza Forjahierro",
        faction: FactionId::Alliance,
        spawn_wpos: Vec2::new(13200, 14800),
        graveyard_wpos: Vec2::new(13220, 14810),
    },
    RaceStartingInfo {
        species: Species::Elf,
        race_name: "Elfo",
        homeland_name: "Bosque Místico de Elveron",
        town_name: "Arboleda de las Estrellas",
        faction: FactionId::Alliance,
        spawn_wpos: Vec2::new(14800, 13200),
        graveyard_wpos: Vec2::new(14820, 13210),
    },
    // Horda
    RaceStartingInfo {
        species: Species::Orc,
        race_name: "Orco",
        homeland_name: "Valle Quebrantahuesos",
        town_name: "Bastión Ogron",
        faction: FactionId::Horde,
        spawn_wpos: Vec2::new(18000, 18000),
        graveyard_wpos: Vec2::new(18020, 18010),
    },
    RaceStartingInfo {
        species: Species::Draugr,
        race_name: "Draugr",
        homeland_name: "Criptas de la Desolación",
        town_name: "Sepulcro Sombrío",
        faction: FactionId::Horde,
        spawn_wpos: Vec2::new(18800, 17200),
        graveyard_wpos: Vec2::new(18820, 17210),
    },
    RaceStartingInfo {
        species: Species::Danari,
        race_name: "Danari",
        homeland_name: "Oasis del Sol Silencioso",
        town_name: "Santuario de las Arenas",
        faction: FactionId::Horde,
        spawn_wpos: Vec2::new(17200, 18800),
        graveyard_wpos: Vec2::new(17220, 18810),
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

/// Devuelve la zona en la que se encuentra una coordenada del mundo
pub fn get_zone_at(wpos: Vec2<f32>) -> &'static ZoneDefinition {
    // Buscar la zona más cercana o que contenga la coordenada dentro de su radio
    ZONES
        .iter()
        .filter(|z| (z.center_wpos.as_::<f32>() - wpos).magnitude() <= z.radius)
        .min_by(|a, b| a.radius.partial_cmp(&b.radius).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap_or(&ZONES[4]) // Tierras Disputadas por defecto si está en zonas abiertas
}
