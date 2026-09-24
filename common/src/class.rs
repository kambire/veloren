use crate::comp::{
    item::tool::ToolKind,
    skills::Skill,
    skillset::{SkillGroupKind, SkillSet},
};
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
    Tamer,
}

impl Component for CharacterClass {
    type Storage = VecStorage<Self>;
}

/// Arma inicial exclusiva del Entrenador. Es un cetro, así que su id también
/// contiene "sceptre" y debe comprobarse antes que el del Sacerdote.
pub const TAMER_WEAPON: &str = "common.items.weapons.sceptre.cayado_entrenador";

/// Fracción del daño que hacen las mascotas de las clases que no son Entrenador.
/// Cualquier clase puede domar bestias, pero solo las del Entrenador pegan al 100 %.
pub const NON_TAMER_PET_DAMAGE: f32 = 0.2;

impl CharacterClass {
    pub const ALL: [CharacterClass; 7] = [
        CharacterClass::Warrior,
        CharacterClass::Priest,
        CharacterClass::Mage,
        CharacterClass::Hunter,
        CharacterClass::Rogue,
        CharacterClass::Paladin,
        CharacterClass::Tamer,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Guerrero",
            CharacterClass::Priest => "Sacerdote",
            CharacterClass::Mage => "Mago",
            CharacterClass::Hunter => "Cazador",
            CharacterClass::Rogue => "Pícaro",
            CharacterClass::Paladin => "Paladín",
            CharacterClass::Tamer => "Entrenador",
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
            CharacterClass::Tamer => {
                "Domador de bestias que lucha junto a sus mascotas. Captura criaturas salvajes \
                 con collares, las potencia y cura un poco a su grupo; su propio daño es leve."
            },
        }
    }

    pub fn role(&self) -> CombatRole {
        match self {
            CharacterClass::Warrior | CharacterClass::Paladin => CombatRole::Tank,
            CharacterClass::Priest => CombatRole::Healer,
            // El Entrenador hace su daño a través de sus mascotas
            CharacterClass::Mage
            | CharacterClass::Hunter
            | CharacterClass::Rogue
            | CharacterClass::Tamer => CombatRole::Dps,
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
            CharacterClass::Tamer => (Some(TAMER_WEAPON), None),
        }
    }

    /// Deducción de la clase según las armas que empuña el personaje
    pub fn from_weapons(mainhand: Option<&str>, offhand: Option<&str>) -> Self {
        match (mainhand, offhand) {
            (Some(m), _) if m == TAMER_WEAPON => CharacterClass::Tamer,
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
            CharacterClass::Tamer => 0.95,
            CharacterClass::Priest => 0.90,
            CharacterClass::Mage => 0.85,
        }
    }

    /// Único árbol de talentos que puede usar la clase. Se desbloquea al crear el
    /// personaje y, como los jugadores ya no pueden desbloquear otros árboles,
    /// queda guardado como la clase del personaje.
    pub fn skill_group(&self) -> SkillGroupKind {
        match self {
            // El Pícaro no tiene árbol de dagas en Veloren: usa la rama ágil de la espada
            CharacterClass::Warrior | CharacterClass::Rogue => SkillGroupKind::Weapon(ToolKind::Sword),
            CharacterClass::Paladin => SkillGroupKind::Weapon(ToolKind::Hammer),
            CharacterClass::Priest => SkillGroupKind::Weapon(ToolKind::Sceptre),
            CharacterClass::Mage => SkillGroupKind::Weapon(ToolKind::Staff),
            CharacterClass::Hunter => SkillGroupKind::Weapon(ToolKind::Bow),
            CharacterClass::Tamer => SkillGroupKind::Tamer,
        }
    }

    /// Árboles de clase: todos menos el general y el de minería
    pub fn is_class_skill_group(group: SkillGroupKind) -> bool {
        Self::ALL.iter().any(|class| class.skill_group() == group)
    }

    /// Indica si el personaje ya tiene desbloqueado algún árbol de clase
    pub fn has_class_tree(skill_set: &SkillSet) -> bool {
        Self::ALL
            .iter()
            .any(|class| skill_set.skill_group_accessible(class.skill_group()))
    }

    /// Desbloquea gratis el árbol de la clase. Se regala el punto general que
    /// cuesta el desbloqueo y se gasta de forma normal, para que al cargar el
    /// personaje de la base de datos el desbloqueo se reproduzca sin errores.
    pub fn unlock_class_tree(&self, skill_set: &mut SkillSet) {
        let unlock = Skill::UnlockGroup(self.skill_group());
        if skill_set.has_skill(unlock) {
            return;
        }
        skill_set.add_skill_points(SkillGroupKind::General, unlock.skill_cost(1));
        if let Err(err) = skill_set.unlock_skill(unlock) {
            tracing::warn!(?err, class = self.name(), "No se pudo desbloquear el árbol de clase");
        }
    }
}
