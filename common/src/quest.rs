use crate::zone::FactionId;
use hashbrown::HashSet;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

/// Identificador único de una misión
pub type QuestId = String;

/// Estado de una misión para un personaje
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestState {
    NotStarted,
    InProgress,
    ReadyToTurnIn,
    Completed,
}

/// Marcador visual sobre la cabeza del NPC (estilo World of Warcraft)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestMarker {
    /// Misión disponible para aceptar: signo '!' dorado
    Available,
    /// Misión completada lista para entregar: signo '?' dorado
    ReadyToTurnIn,
    /// Misión en curso pero con objetivos pendientes: signo '?' plateado/gris
    InProgress,
}

impl QuestMarker {
    pub fn icon(&self) -> &'static str {
        match self {
            QuestMarker::Available => "!",
            QuestMarker::ReadyToTurnIn => "?",
            QuestMarker::InProgress => "?",
        }
    }

    /// Color en formato RGBA (estilo WoW: dorado para disponible/entregar, gris para en progreso)
    pub fn color_rgba(&self) -> (u8, u8, u8, u8) {
        match self {
            QuestMarker::Available => (255, 215, 0, 255),      // Dorado
            QuestMarker::ReadyToTurnIn => (255, 215, 0, 255),   // Dorado
            QuestMarker::InProgress => (170, 170, 170, 220),   // Plateado/Gris
        }
    }
}

/// Tipo de objetivo para completar una misión
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestObjective {
    /// Eliminar una cantidad de enemigos específicos
    Kill {
        target_name: String,
        required: u32,
    },
    /// Recolectar o entregar una cantidad de objetos
    Collect {
        item_id: String,
        item_name: String,
        required: u32,
    },
    /// Explorar o descubrir un lugar o campamento
    Explore {
        location_name: String,
    },
}

/// Recompensas otorgadas al completar la misión
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestReward {
    pub xp: u32,
    pub copper_coins: u32,
    pub items: Vec<String>,
}

/// Definición estática de una misión (formato .ron)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestDefinition {
    pub id: QuestId,
    pub title: String,
    pub description: String,
    pub giver_name: String,
    pub min_level: u8,
    pub faction: Option<FactionId>,
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestReward,
}

/// Progreso individual de una misión activa en el jugador
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestProgress {
    pub quest_id: QuestId,
    pub state: QuestState,
    pub current_progress: Vec<u32>,
}

impl QuestProgress {
    pub fn new(quest: &QuestDefinition) -> Self {
        Self {
            quest_id: quest.id.clone(),
            state: QuestState::InProgress,
            current_progress: vec![0; quest.objectives.len()],
        }
    }

    /// Comprueba si todos los objetivos de la misión han alcanzado la cantidad requerida
    pub fn check_completion(&mut self, quest: &QuestDefinition) -> bool {
        let is_complete = quest.objectives.iter().enumerate().all(|(idx, obj)| {
            let current = self.current_progress.get(idx).copied().unwrap_or(0);
            match obj {
                QuestObjective::Kill { required, .. } => current >= *required,
                QuestObjective::Collect { required, .. } => current >= *required,
                QuestObjective::Explore { .. } => current >= 1,
            }
        });

        if is_complete && self.state == QuestState::InProgress {
            self.state = QuestState::ReadyToTurnIn;
        }

        is_complete
    }

    /// Suma progreso a un objetivo de eliminación de criatura
    pub fn on_kill(&mut self, quest: &QuestDefinition, killed_name: &str) -> bool {
        let mut changed = false;
        for (idx, obj) in quest.objectives.iter().enumerate() {
            if let QuestObjective::Kill { target_name, required } = obj {
                if killed_name.to_lowercase().contains(&target_name.to_lowercase()) {
                    let current = self.current_progress.get_mut(idx).unwrap();
                    if *current < *required {
                        *current += 1;
                        changed = true;
                    }
                }
            }
        }
        if changed {
            self.check_completion(quest);
        }
        changed
    }
}

/// Componente ECS del jugador para almacenar sus misiones activas y completadas
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActiveQuests {
    pub active: Vec<QuestProgress>,
    pub completed: HashSet<QuestId>,
}

impl Component for ActiveQuests {
    type Storage = VecStorage<Self>;
}

impl ActiveQuests {
    pub fn is_completed(&self, quest_id: &str) -> bool {
        self.completed.contains(quest_id)
    }

    pub fn get_active_mut(&mut self, quest_id: &str) -> Option<&mut QuestProgress> {
        self.active.iter_mut().find(|q| q.quest_id == quest_id)
    }

    pub fn accept_quest(&mut self, quest: &QuestDefinition) -> bool {
        if self.is_completed(&quest.id) || self.active.iter().any(|q| q.quest_id == quest.id) {
            return false;
        }
        self.active.push(QuestProgress::new(quest));
        true
    }

    pub fn complete_quest(&mut self, quest_id: &str) -> bool {
        if let Some(pos) = self.active.iter().position(|q| q.quest_id == quest_id) {
            let q = self.active.remove(pos);
            if q.state == QuestState::ReadyToTurnIn {
                self.completed.insert(q.quest_id);
                return true;
            }
        }
        false
    }
}
