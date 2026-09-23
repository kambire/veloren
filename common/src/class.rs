use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

/// Rol principal en la Trinidad de Combate (estilo World of Warcraft)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatRole {
    Tank,
    Healer,
    Dps,
}

impl CombatRole {
    pub fn name(&self) -> &'static str {
        match self {
            CombatRole::Tank => "Tanque",
            CombatRole::Healer => "Sanador",
            CombatRole::Dps => "DPS",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            CombatRole::Tank => "🛡️",
            CombatRole::Healer => "➕",
            CombatRole::Dps => "⚔️",
        }
    }
}

/// Clases fijas de personaje estilo World of Warcraft
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterClass {
    Warrior,
    Priest,
    Mage,
    Hunter,
    Rogue,
    Paladin,
}

impl Component for CharacterClass {
    type Storage = VecStorage<Self>;
}

impl CharacterClass {
    pub const ALL: [CharacterClass; 6] = [
        CharacterClass::Warrior,
        CharacterClass::Priest,
        CharacterClass::Mage,
        CharacterClass::Hunter,
        CharacterClass::Rogue,
        CharacterClass::Paladin,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Guerrero",
            CharacterClass::Priest => "Sacerdote",
            CharacterClass::Mage => "Mago",
            CharacterClass::Hunter => "Cazador",
            CharacterClass::Rogue => "Pícaro",
            CharacterClass::Paladin => "Paladín",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => {
                "Defensor inquebrantable de primera línea. Especialista en espada y escudo, \
                 gran mitigación de daño y control de amenaza (aggro)."
            },
            CharacterClass::Priest => {
                "Canalizador de energías vitales. Mantiene con vida a su grupo con cetros \
                 sanadores, auras regenerativas y barreras protectoras."
            },
            CharacterClass::Mage => {
                "Maestro de las artes arcanas e ígneas. Inflige un daño masivo a distancia con \
                 bastones de fuego y controla las masas."
            },
            CharacterClass::Hunter => {
                "Tirador letal a distancia con arco. Combina disparos precisos, gran movilidad y \
                 trampas de control."
            },
            CharacterClass::Rogue => {
                "Asesino sigiloso con dagas duales. Aprovecha las sombras para realizar ataques \
                 críticos sorpresivos y desangrar a sus víctimas."
            },
            CharacterClass::Paladin => {
                "Campeón de la luz bendita. Híbrido protector que combate con martillo sagrado y \
                 escudo, asistiendo con auras a sus aliados."
            },
        }
    }

    pub fn role(&self) -> CombatRole {
        match self {
            CharacterClass::Warrior | CharacterClass::Paladin => CombatRole::Tank,
            CharacterClass::Priest => CombatRole::Healer,
            CharacterClass::Mage | CharacterClass::Hunter | CharacterClass::Rogue => CombatRole::Dps,
        }
    }

    /// Armas iniciales por defecto según la clase seleccionada
    pub fn starter_weapons(&self) -> (Option<&'static str>, Option<&'static str>) {
        match self {
            CharacterClass::Warrior => (
                Some("common.items.weapons.sword_1h.starter"),
                Some("common.items.weapons.shield.starter_shield"),
            ),
            CharacterClass::Priest => (
                Some("common.items.weapons.sceptre.starter_sceptre"),
                None,
            ),
            CharacterClass::Mage => (
                Some("common.items.weapons.staff.starter_staff"),
                None,
            ),
            CharacterClass::Hunter => (
                Some("common.items.weapons.bow.starter"),
                None,
            ),
            CharacterClass::Rogue => (
                Some("common.items.weapons.dagger.starter_dagger"),
                Some("common.items.weapons.dagger.starter_dagger"),
            ),
            CharacterClass::Paladin => (
                Some("common.items.weapons.hammer.starter_hammer"),
                Some("common.items.weapons.shield.starter_shield"),
            ),
        }
    }

    /// Deducción de la clase según las armas que empuña el personaje
    pub fn from_weapons(mainhand: Option<&str>, offhand: Option<&str>) -> Self {
        match (mainhand, offhand) {
            (Some(m), _) if m.contains("sceptre") => CharacterClass::Priest,
            (Some(m), _) if m.contains("staff") => CharacterClass::Mage,
            (Some(m), _) if m.contains("bow") => CharacterClass::Hunter,
            (Some(m), _) if m.contains("dagger") => CharacterClass::Rogue,
            (Some(m), Some(o)) if m.contains("hammer") && o.contains("shield") => CharacterClass::Paladin,
            (Some(m), _) if m.contains("hammer") => CharacterClass::Paladin,
            (Some(_m), Some(o)) if o.contains("shield") => CharacterClass::Warrior,
            (Some(m), _) if m.contains("sword") || m.contains("axe") => CharacterClass::Warrior,
            _ => CharacterClass::Warrior, // Guerrero por defecto
        }
    }

    /// Deducción de la clase a partir del inventario y armas equipadas
    pub fn from_inventory(inventory: &crate::comp::Inventory) -> Self {
        let mut mainhand: Option<String> = None;
        let mut offhand: Option<String> = None;

        for (slot, item) in inventory.equipped_items_with_slot() {
            match slot {
                crate::comp::inventory::slot::EquipSlot::ActiveMainhand => {
                    mainhand = item.item_definition_id().itemdef_id().map(|s| s.to_string());
                },
                crate::comp::inventory::slot::EquipSlot::ActiveOffhand => {
                    offhand = item.item_definition_id().itemdef_id().map(|s| s.to_string());
                },
                _ => {},
            }
        }

        Self::from_weapons(
            mainhand.as_deref(),
            offhand.as_deref(),
        )
    }

    /// Multiplicador de amenaza para la gestión de aggro en combate
    pub fn threat_multiplier(&self) -> f32 {
        match self.role() {
            CombatRole::Tank => 2.5,   // +150% de amenaza para mantener el aggro de los monstruos
            CombatRole::Healer => 0.5, // 50% de reducción de amenaza al curar
            CombatRole::Dps => 1.0,    // Amenaza estándar
        }
    }

    /// Modificador de salud máxima según la especialidad de la clase
    pub fn health_multiplier(&self) -> f32 {
        match self {
            CharacterClass::Warrior => 1.30,
            CharacterClass::Paladin => 1.25,
            CharacterClass::Hunter => 1.05,
            CharacterClass::Rogue => 1.00,
            CharacterClass::Priest => 0.90,
            CharacterClass::Mage => 0.85,
        }
    }
}
