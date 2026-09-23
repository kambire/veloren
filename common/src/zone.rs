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

/// Devuelve la zona en la que se encuentra una coordenada del mundo
pub fn get_zone_at(wpos: Vec2<f32>) -> &'static ZoneDefinition {
    // Buscar la zona más cercana o que contenga la coordenada dentro de su radio
    ZONES
        .iter()
        .filter(|z| (z.center_wpos.as_::<f32>() - wpos).magnitude() <= z.radius)
        .min_by(|a, b| a.radius.partial_cmp(&b.radius).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap_or(&ZONES[4]) // Tierras Disputadas por defecto si está en zonas abiertas
}
