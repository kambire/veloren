use serde::{Deserialize, Serialize};
use specs::{Component, DerefFlaggedStorage};
use uuid::Uuid;

use crate::resources::{BattleMode, Time};

pub const MAX_ALIAS_LEN: usize = 32;

#[derive(Debug)]
pub enum DisconnectReason {
    Kicked,
    NewerLogin,
    NetworkError,
    Timeout,
    ClientRequested,
    InvalidClientType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub alias: String,
    pub battle_mode: BattleMode,
    pub last_battlemode_change: Option<Time>,
    #[serde(default)]
    pub faction: Option<crate::zone::FactionId>,
    uuid: Uuid,
}

impl BattleMode {
    pub fn may_harm(self, other: Self) -> bool {
        matches!((self, other), (BattleMode::PvP, BattleMode::PvP))
    }
}

impl Player {
    pub fn new(
        alias: String,
        battle_mode: BattleMode,
        uuid: Uuid,
        last_battlemode_change: Option<Time>,
    ) -> Self {
        Self {
            alias,
            battle_mode,
            last_battlemode_change,
            faction: None,
            uuid,
        }
    }

    pub fn with_faction(mut self, faction: Option<crate::zone::FactionId>) -> Self {
        self.faction = faction;
        self
    }

    /// Reglas de combate MMORPG entre jugadores:
    /// - Miembros de la misma facción (Alianza con Alianza, Horda con Horda) son aliados y NUNCA pueden dañarse.
    /// - Miembros de facciones rivales (Alianza vs Horda) pueden entrar en combate abierto.
    /// - Si no tienen facciones asignadas, aplica el consentimiento mutuo de BattleMode::PvP.
    pub fn may_harm(&self, other: &Player) -> bool {
        if let (Some(f1), Some(f2)) = (self.faction, other.faction) {
            if f1 == f2 {
                // Mismo bando: Fuego amigo deshabilitado
                return false;
            } else {
                // Bando rival: Combate entre facciones habilitado
                return true;
            }
        }

        self.battle_mode.may_harm(other.battle_mode)
    }

    pub fn is_valid(&self) -> bool { Self::alias_validate(&self.alias).is_ok() }

    pub fn alias_validate(alias: &str) -> Result<(), AliasError> {
        // TODO: Expose auth name validation and use it here.
        // See https://gitlab.com/veloren/auth/-/blob/master/server/src/web.rs#L20
        if !alias
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            Err(AliasError::ForbiddenCharacters)
        } else if alias.len() > MAX_ALIAS_LEN {
            Err(AliasError::TooLong)
        } else {
            Ok(())
        }
    }

    /// Not to be confused with uid
    pub fn uuid(&self) -> Uuid { self.uuid }
}

impl Component for Player {
    type Storage = DerefFlaggedStorage<Self, specs::DenseVecStorage<Self>>;
}

pub enum AliasError {
    ForbiddenCharacters,
    TooLong,
}

#[expect(clippy::to_string_trait_impl)]
impl ToString for AliasError {
    fn to_string(&self) -> String {
        match *self {
            AliasError::ForbiddenCharacters => "Alias contains illegal characters.",
            AliasError::TooLong => "Alias is too long.",
        }
        .to_string()
    }
}
