use common::comp::{
    self,
    inventory::item::{Item, item_key::ItemKey},
};
use serde::{Deserialize, Serialize};

use super::HudInfo;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Slot {
    #[default]
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5,
    Seven = 6,
    Eight = 7,
    Nine = 8,
    Ten = 9,
    // Fila superior (Barra secundaria WoW)
    Eleven = 10,
    Twelve = 11,
    Thirteen = 12,
    Fourteen = 13,
    Fifteen = 14,
    Sixteen = 15,
    Seventeen = 16,
    Eighteen = 17,
    Nineteen = 18,
    Twenty = 19,
}

pub const HOTBAR_SLOT_COUNT: usize = 20;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SlotContents {
    Inventory(u64, ItemKey),
    Ability(usize),
    PrimaryAbility,
    SecondaryAbility,
}

#[derive(Clone, Default)]
pub struct State {
    pub slots: [Option<SlotContents>; HOTBAR_SLOT_COUNT],
    inputs: [bool; HOTBAR_SLOT_COUNT],
    pub currently_selected_slot: Slot,
}

impl State {
    pub fn new(slots: [Option<SlotContents>; HOTBAR_SLOT_COUNT]) -> Self {
        Self {
            slots,
            inputs: [false; HOTBAR_SLOT_COUNT],
            currently_selected_slot: Slot::default(),
        }
    }

    /// Returns true if the button was just pressed
    pub fn process_input(&mut self, slot: Slot, state: bool) -> bool {
        let slot = slot as usize;
        let just_pressed = !self.inputs[slot] && state;
        self.inputs[slot] = state;
        just_pressed
    }

    pub fn get(&self, slot: Slot) -> Option<SlotContents> { self.slots[slot as usize].clone() }

    pub fn swap(&mut self, a: Slot, b: Slot) { self.slots.swap(a as usize, b as usize); }

    pub fn clear_slot(&mut self, slot: Slot) { self.slots[slot as usize] = None; }

    pub fn add_inventory_link(&mut self, slot: Slot, item: &Item) {
        self.slots[slot as usize] = Some(SlotContents::Inventory(
            item.item_hash(),
            ItemKey::from(item),
        ));
    }

    // Adds ability slots if missing and should be present
    // Removes ability slots if not there and shouldn't be present
    pub fn maintain_abilities(&mut self, client: &client::Client, info: &HudInfo) {
        use specs::WorldExt;
        if let Some(active_abilities) = client
            .state()
            .ecs()
            .read_storage::<comp::ActiveAbilities>()
            .get(info.viewpoint_entity)
        {
            // Casilla 1 por defecto al poder de Click Izquierdo (Primario)
            if self.slots[0].is_none() {
                self.slots[0] = Some(SlotContents::PrimaryAbility);
            }
            // Casilla 2 por defecto al poder de Click Derecho (Secundario)
            if self.slots[1].is_none() {
                self.slots[1] = Some(SlotContents::SecondaryAbility);
            }

            use common::comp::ability::AuxiliaryAbility;
            let aux = active_abilities.auxiliary_set(
                client.inventories().get(info.viewpoint_entity),
                client
                    .state()
                    .read_storage::<comp::SkillSet>()
                    .get(info.viewpoint_entity),
            );

            // Casillas 3 en adelante (índice 2..) para poderes auxiliares
            for (i, ability) in aux.iter().enumerate() {
                let slot_idx = i + 2;
                if slot_idx < 10 {
                    let hotbar_slot = &mut self.slots[slot_idx];
                    if matches!(ability, AuxiliaryAbility::Empty) {
                        if matches!(hotbar_slot, Some(SlotContents::Ability(_))) {
                            *hotbar_slot = None;
                        }
                    } else if hotbar_slot.is_none() || matches!(hotbar_slot, Some(SlotContents::Ability(_))) {
                        *hotbar_slot = Some(SlotContents::Ability(i));
                    }
                }
            }
        } else {
            self.slots
                .iter_mut()
                .filter(|slot| matches!(slot, Some(SlotContents::Ability(_))))
                .for_each(|slot| *slot = None)
        }
    }
}

impl Slot {
    const SLOTS: [Slot; HOTBAR_SLOT_COUNT] = [
        Slot::One,
        Slot::Two,
        Slot::Three,
        Slot::Four,
        Slot::Five,
        Slot::Six,
        Slot::Seven,
        Slot::Eight,
        Slot::Nine,
        Slot::Ten,
        Slot::Eleven,
        Slot::Twelve,
        Slot::Thirteen,
        Slot::Fourteen,
        Slot::Fifteen,
        Slot::Sixteen,
        Slot::Seventeen,
        Slot::Eighteen,
        Slot::Nineteen,
        Slot::Twenty,
    ];

    pub fn next_slot(&mut self) {
        let current_slot = *self as usize;
        let next_slot = (current_slot + 1) % Self::SLOTS.len();
        *self = Self::SLOTS[next_slot];
    }

    pub fn previous_slot(&mut self) {
        let current_slot = *self as usize;
        let previous_slot = (current_slot + Self::SLOTS.len() - 1) % Self::SLOTS.len();
        *self = Self::SLOTS[previous_slot];
    }
}
