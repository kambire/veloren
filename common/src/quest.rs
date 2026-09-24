//! Misiones estilo World of Warcraft: cadenas de misiones que dan los NPC de
//! los pueblos según su profesión y el continente en el que viven.
//!
//! Las definiciones están en `assets/common/quests/quest_manifest.ron`. El
//! progreso de cada personaje está en el componente [`ActiveQuests`], que se
//! guarda en la base de datos y se sincroniza con su cliente para mostrar los
//! marcadores `!` y `?`.

use crate::{
    assets::{AssetExt, Ron},
    class::CharacterClass,
    comp::skillset::{SkillGroupKind, SkillSet},
    rtsim::Profession,
    zone::{FactionId, ZoneId},
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use specs::{Component, DenseVecStorage, DerefFlaggedStorage, VecStorage};
use std::collections::BTreeSet;

/// Identificador único de una misión
pub type QuestId = String;

lazy_static! {
    /// Todas las misiones del juego
    pub static ref QUESTS: Vec<QuestDefinition> = {
        let quests: Vec<QuestDefinition> =
            Ron::load_expect_cloned("common.quests.quest_manifest").into_inner();
        quests
    };
}

/// Busca la definición de una misión por su identificador
pub fn quest(id: &str) -> Option<&'static QuestDefinition> {
    QUESTS.iter().find(|quest| quest.id == id)
}

/// Da la experiencia de una recompensa, repartida como la de combate: entre el
/// árbol general y el árbol de la clase del personaje
pub fn grant_xp(skill_set: &mut SkillSet, xp: u32) {
    let mut pools = vec![SkillGroupKind::General];
    for class in CharacterClass::ALL {
        let group = class.skill_group();
        if skill_set.skill_group_accessible(group) && !pools.contains(&group) {
            pools.push(group);
        }
    }
    let share = (xp as f32 / pools.len() as f32).ceil() as u32;
    for pool in pools {
        skill_set.add_experience(pool, share);
    }
}

/// Tipo de NPC que da una misión. Es la profesión de los aldeanos de Veloren
/// que pueden dar misiones.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestGiver {
    Guard,
    Hunter,
    Farmer,
    Herbalist,
    Blacksmith,
    Merchant,
    Chef,
    Alchemist,
}

impl QuestGiver {
    pub fn from_profession(profession: &Profession) -> Option<Self> {
        Some(match profession {
            Profession::Guard => QuestGiver::Guard,
            Profession::Hunter => QuestGiver::Hunter,
            Profession::Farmer => QuestGiver::Farmer,
            Profession::Herbalist => QuestGiver::Herbalist,
            Profession::Blacksmith => QuestGiver::Blacksmith,
            Profession::Merchant => QuestGiver::Merchant,
            Profession::Chef => QuestGiver::Chef,
            Profession::Alchemist => QuestGiver::Alchemist,
            _ => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            QuestGiver::Guard => "guardia",
            QuestGiver::Hunter => "cazador",
            QuestGiver::Farmer => "granjero",
            QuestGiver::Herbalist => "herbolario",
            QuestGiver::Blacksmith => "herrero",
            QuestGiver::Merchant => "mercader",
            QuestGiver::Chef => "cocinero",
            QuestGiver::Alchemist => "alquimista",
        }
    }
}

/// Componente de los NPC de pueblo que pueden dar misiones. Se sincroniza con
/// los clientes para dibujar los marcadores sobre su cabeza.
impl Component for QuestGiver {
    type Storage = DerefFlaggedStorage<Self, DenseVecStorage<Self>>;
}

/// Marcador sobre la cabeza de una entidad (estilo World of Warcraft)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestMarker {
    /// NPC con una misión disponible: '!' dorado
    Available,
    /// NPC al que entregar una misión completada: '?' dorado
    ReadyToTurnIn,
    /// NPC con una misión tuya aún sin terminar: '?' gris
    InProgress,
    /// Criatura que cuenta para una misión activa: '!' naranja
    Target,
}

impl QuestMarker {
    pub fn icon(&self) -> &'static str {
        match self {
            QuestMarker::Available | QuestMarker::Target => "!",
            QuestMarker::ReadyToTurnIn | QuestMarker::InProgress => "?",
        }
    }

    /// Color en RGBA: dorado para dar/entregar, gris en progreso y naranja
    /// para las criaturas objetivo
    pub fn color_rgba(&self) -> (u8, u8, u8, u8) {
        match self {
            QuestMarker::Available | QuestMarker::ReadyToTurnIn => (255, 215, 0, 255),
            QuestMarker::InProgress => (170, 170, 170, 220),
            QuestMarker::Target => (255, 130, 20, 255),
        }
    }

    /// Tamaño de letra del marcador: el de las criaturas es más pequeño
    pub fn font_size(&self) -> u32 {
        match self {
            QuestMarker::Target => 20,
            _ => 28,
        }
    }
}

/// Objetivo de una misión
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestObjective {
    /// Matar criaturas. `targets` son palabras clave de especie de
    /// `common.npc_names` (por ejemplo `"wolf"` o `"goblin_thug"`); cuenta
    /// cualquiera de ellas.
    Kill {
        targets: Vec<String>,
        label: String,
        required: u32,
    },
}

impl QuestObjective {
    pub fn required(&self) -> u32 {
        match self {
            QuestObjective::Kill { required, .. } => *required,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            QuestObjective::Kill { label, .. } => label,
        }
    }

    /// Indica si matar una criatura con esta palabra clave cuenta para el objetivo
    pub fn counts_kill(&self, species_keyword: &str) -> bool {
        match self {
            QuestObjective::Kill { targets, .. } => targets.iter().any(|t| t == species_keyword),
        }
    }
}

/// Recompensas de una misión
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestReward {
    pub xp: u32,
    /// Monedas de cobre (100 cobre = 1 plata, 100 plata = 1 oro)
    pub coins: u32,
    /// Objetos: (identificador del objeto, cantidad)
    #[serde(default)]
    pub items: Vec<(String, u32)>,
}

/// Definición de una misión
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestDefinition {
    pub id: QuestId,
    /// Nombre de la cadena a la que pertenece
    pub chain: String,
    pub title: String,
    /// Lo que cuenta el NPC al ofrecer la misión
    pub offer_text: String,
    /// Lo que dice el NPC si vuelves sin haberla terminado
    pub progress_text: String,
    /// Lo que dice el NPC al entregarla
    pub complete_text: String,
    /// Profesión del NPC que da y recibe la misión
    pub giver: QuestGiver,
    /// Continente en el que se da la misión
    pub zone: ZoneId,
    /// Facción que puede hacerla (`None` = cualquiera)
    pub faction: Option<FactionId>,
    /// Misión anterior de la cadena, que hay que haber completado antes
    pub requires: Option<QuestId>,
    /// Nivel recomendado
    pub min_level: u8,
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestReward,
}

impl QuestDefinition {
    /// El NPC `giver` que vive en `zone` da y recibe esta misión
    pub fn given_by(&self, giver: QuestGiver, zone: ZoneId) -> bool {
        self.giver == giver && self.zone == zone
    }

    pub fn allowed_for(&self, faction: Option<FactionId>) -> bool {
        self.faction.is_none() || self.faction == faction
    }
}

/// Progreso de una misión aceptada
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestProgress {
    pub quest_id: QuestId,
    /// Progreso de cada objetivo, en el mismo orden que en la definición
    pub counts: Vec<u32>,
}

impl QuestProgress {
    pub fn new(quest: &QuestDefinition) -> Self {
        Self {
            quest_id: quest.id.clone(),
            counts: vec![0; quest.objectives.len()],
        }
    }

    pub fn definition(&self) -> Option<&'static QuestDefinition> { quest(&self.quest_id) }

    pub fn count(&self, objective: usize) -> u32 { self.counts.get(objective).copied().unwrap_or(0) }

    /// Todos los objetivos cumplidos: lista para entregar
    pub fn is_complete(&self) -> bool {
        self.definition().is_some_and(|quest| {
            quest
                .objectives
                .iter()
                .enumerate()
                .all(|(i, objective)| self.count(i) >= objective.required())
        })
    }
}

/// Aviso de progreso de una misión tras matar una criatura
pub struct KillProgress {
    pub quest_title: String,
    pub objective_label: String,
    pub count: u32,
    pub required: u32,
    /// La misión acaba de quedar lista para entregar
    pub quest_completed: bool,
}

/// Misiones de un personaje: las que tiene en curso y las que ya entregó
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveQuests {
    pub active: Vec<QuestProgress>,
    pub completed: BTreeSet<QuestId>,
}

impl Component for ActiveQuests {
    type Storage = DerefFlaggedStorage<Self, VecStorage<Self>>;
}

impl ActiveQuests {
    pub fn is_completed(&self, quest_id: &str) -> bool { self.completed.contains(quest_id) }

    pub fn is_active(&self, quest_id: &str) -> bool {
        self.active.iter().any(|q| q.quest_id == quest_id)
    }

    pub fn progress(&self, quest_id: &str) -> Option<&QuestProgress> {
        self.active.iter().find(|q| q.quest_id == quest_id)
    }

    /// Misiones que el NPC puede ofrecer: son suyas, de tu facción, ya hiciste
    /// la anterior de la cadena y todavía no la has aceptado ni completado
    pub fn available_from(
        &self,
        giver: QuestGiver,
        zone: ZoneId,
        faction: Option<FactionId>,
    ) -> impl Iterator<Item = &'static QuestDefinition> + '_ {
        QUESTS.iter().filter(move |quest| {
            quest.given_by(giver, zone)
                && quest.allowed_for(faction)
                && quest.requires.as_ref().is_none_or(|req| self.is_completed(req))
                && !self.is_active(&quest.id)
                && !self.is_completed(&quest.id)
        })
    }

    /// Misiones en curso que se entregan a este NPC
    pub fn active_for(
        &self,
        giver: QuestGiver,
        zone: ZoneId,
    ) -> impl Iterator<Item = (&QuestProgress, &'static QuestDefinition)> + '_ {
        self.active.iter().filter_map(move |progress| {
            progress
                .definition()
                .filter(|quest| quest.given_by(giver, zone))
                .map(|quest| (progress, quest))
        })
    }

    /// Marcador que mostrar sobre un NPC de misiones para este personaje
    pub fn marker_for_giver(
        &self,
        giver: QuestGiver,
        zone: ZoneId,
        faction: Option<FactionId>,
    ) -> Option<QuestMarker> {
        let mut in_progress = false;
        for (progress, _) in self.active_for(giver, zone) {
            if progress.is_complete() {
                return Some(QuestMarker::ReadyToTurnIn);
            }
            in_progress = true;
        }
        if self.available_from(giver, zone, faction).next().is_some() {
            Some(QuestMarker::Available)
        } else if in_progress {
            Some(QuestMarker::InProgress)
        } else {
            None
        }
    }

    /// Indica si matar una criatura con esta palabra clave cuenta para alguna
    /// misión en curso con el objetivo aún sin cumplir
    pub fn is_target(&self, species_keyword: &str) -> bool {
        self.active.iter().any(|progress| {
            progress.definition().is_some_and(|quest| {
                quest.objectives.iter().enumerate().any(|(i, objective)| {
                    progress.count(i) < objective.required()
                        && objective.counts_kill(species_keyword)
                })
            })
        })
    }

    /// Acepta una misión. Devuelve `false` si ya estaba aceptada o completada.
    pub fn accept(&mut self, quest: &QuestDefinition) -> bool {
        if self.is_active(&quest.id) || self.is_completed(&quest.id) {
            return false;
        }
        self.active.push(QuestProgress::new(quest));
        true
    }

    /// Suma una muerte a los objetivos que correspondan y devuelve los avisos
    /// de progreso
    pub fn register_kill(&mut self, species_keyword: &str) -> Vec<KillProgress> {
        let mut updates = Vec::new();
        for progress in &mut self.active {
            let Some(quest) = progress.definition() else {
                continue;
            };
            let was_complete = progress.is_complete();
            let mut changed = Vec::new();
            for (i, objective) in quest.objectives.iter().enumerate() {
                if objective.counts_kill(species_keyword) && progress.count(i) < objective.required()
                {
                    if let Some(count) = progress.counts.get_mut(i) {
                        *count += 1;
                        changed.push((i, *count));
                    }
                }
            }
            let completed_now = !was_complete && progress.is_complete();
            for (i, count) in changed {
                updates.push(KillProgress {
                    quest_title: quest.title.clone(),
                    objective_label: quest.objectives[i].label().to_string(),
                    count,
                    required: quest.objectives[i].required(),
                    quest_completed: completed_now,
                });
            }
        }
        updates
    }

    /// Entrega una misión completada. Devuelve su definición si se entregó.
    pub fn turn_in(&mut self, quest_id: &str) -> Option<&'static QuestDefinition> {
        let pos = self
            .active
            .iter()
            .position(|q| q.quest_id == quest_id && q.is_complete())?;
        let progress = self.active.remove(pos);
        self.completed.insert(progress.quest_id.clone());
        progress.definition()
    }
}
