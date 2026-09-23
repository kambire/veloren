use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SiteKindMeta {
    Dungeon(DungeonKindMeta),
    Cave,
    Settlement(SettlementKindMeta),
    Castle,
    #[default]
    Void,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DungeonKindMeta {
    Gnarling,
    Adlet,
    Haniwa,
    SeaChapel,
    Terracotta,
    Cultist,
    Sahagin,
    Myrmidon,
    VampireCastle,
    DwarvenMine,
}

impl DungeonKindMeta {
    /// Rango de niveles recomendados estilo WoW (1-60)
    pub fn level_range(&self) -> (u32, u32) {
        match self {
            Self::Gnarling => (5, 12),
            Self::Adlet => (12, 20),
            Self::Haniwa => (18, 28),
            Self::SeaChapel => (20, 30),
            Self::Sahagin => (25, 35),
            Self::DwarvenMine => (32, 42),
            Self::Terracotta => (38, 48),
            Self::Myrmidon => (45, 54),
            Self::Cultist => (50, 58),
            Self::VampireCastle => (60, 60),
        }
    }

    /// Nombre de la mazmorra con rango de nivel en español
    pub fn display_name_es(&self) -> &'static str {
        match self {
            Self::Gnarling => "Cavernas Gnarling (Nvl 5-12)",
            Self::Adlet => "Templo Helado de Adlet (Nvl 12-20)",
            Self::Haniwa => "Catacumbas Ancestrales Haniwa (Nvl 18-28)",
            Self::SeaChapel => "Santuario de las Profundidades (Nvl 20-30)",
            Self::Sahagin => "Cuevas Acuáticas Sahagin (Nvl 25-35)",
            Self::DwarvenMine => "Minas Olvidadas de los Enanos (Nvl 32-42)",
            Self::Terracotta => "Mausoleo de Terracota (Nvl 38-48)",
            Self::Myrmidon => "Fortaleza Colmena Myrmidon (Nvl 45-54)",
            Self::Cultist => "Bastión del Vacío Cultista (Nvl 50-58)",
            Self::VampireCastle => "Castillo Sanguíneo del Conde [Raid Nvl 60]",
        }
    }

    /// Si es una incursión de banda (Raid)
    pub fn is_raid(&self) -> bool {
        matches!(self, Self::VampireCastle)
    }

    /// Tamaño recomendado de grupo (5 jugadores para mazmorra, 10 para raid)
    pub fn recommended_party_size(&self) -> u32 {
        if self.is_raid() {
            10
        } else {
            5
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SettlementKindMeta {
    Default,
    CliffTown,
    DesertCity,
    SavannahTown,
    CoastalTown,
}
