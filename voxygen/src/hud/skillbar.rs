use super::{
    BLACK, BarNumbers, CRITICAL_HP_COLOR, HP_COLOR, HudInfo, LOW_HP_COLOR, POISE_COLOR,
    POISEBAR_TICK_COLOR, QUALITY_EPIC, QUALITY_LEGENDARY, STAMINA_COLOR, ShortcutNumbers,
    TEXT_COLOR, TEXT_VELORITE, UI_HIGHLIGHT_0, XP_COLOR, hotbar,
    img_ids::{Imgs, ImgsRot},
    item_imgs::ItemImgs,
    slots, util,
};
use crate::{
    GlobalState,
    game_input::GameInput,
    hud::{
        ComboFloater, Position, PositionSpecifier, animation::animation_timer,
        controller_icons as icon_utils,
    },
    key_state::GIVE_UP_HOLD_TIME,
    ui::{
        ImageFrame, ItemTooltip, ItemTooltipManager, ItemTooltipable, Tooltip, TooltipManager,
        Tooltipable,
        fonts::Fonts,
        slot::{ContentSize, SlotMaker},
    },
    window::{KeyMouse, LastInput},
};
use i18n::Localization;

use client::{self, Client};
use common::{
    comp::{
        self, Ability, ActiveAbilities, Body, Buffs, CharacterState, Combo, Energy, Hardcore,
        Health, Inventory, Poise, PoiseState, SkillSet, Stats,
        ability::{AbilityInput, Stance},
        is_downed,
        item::{ItemDesc, ItemI18n, MaterialStatManifest, tool::ToolKind},
        skillset::SkillGroupKind,
    },
    recipe::RecipeBookManifest,
};
use conrod_core::{
    Color, Colorable, Positionable, Sizeable, UiCell, Widget, WidgetCommon, color,
    widget::{self, Button, Image, Rectangle, RoundedRectangle, Text},
    widget_ids,
};
use specs::{Join, WorldExt};
use vek::*;

widget_ids! {
    struct Ids {
        // Death message
        death_message_1,
        death_message_2,
        death_message_1_bg,
        death_message_2_bg,
        death_message_3,
        death_message_3_bg,
        riding_prompt,
        riding_prompt_bg,
        // WoW Player Frame (Arriba a la izquierda)
        player_frame_border,
        player_frame_bg,
        player_portrait_border,
        player_portrait_bg,
        player_portrait,
        player_level_badge_border,
        player_level_badge_bg,
        player_level_txt_bg,
        player_level_txt,
        player_name_txt_bg,
        player_name_txt,
        player_hp_bg,
        player_hp_fill,
        player_hp_decay,
        player_hp_txt_bg,
        player_hp_txt,
        player_mana_bg,
        player_mana_fill,
        player_mana_txt_bg,
        player_mana_txt,
        player_xp_bg,
        player_xp_fill,
        // Skillbar
        frame,
        bg_health,
        frame_health,
        bg_energy,
        frame_energy,
        bg_poise,
        frame_poise,
        m1_ico,
        m2_ico,
        // HP-Bar
        hp_alignment,
        hp_filling,
        hp_decayed,
        hp_txt_bg,
        hp_txt,
        decay_overlay,
        // Energy-Bar
        energy_alignment,
        energy_filling,
        energy_txt_bg,
        energy_txt,
        // Poise-Bar
        poise_alignment,
        poise_filling,
        poise_ticks[],
        poise_txt_bg,
        poise_txt,
        // Exp-Bar
        exp_frame_bg,
        exp_frame,
        exp_filling,
        exp_img_frame_bg,
        exp_img_frame,
        exp_img,
        exp_lvl,
        diary_txt_bg,
        diary_txt,
        sp_arrow,
        sp_arrow_txt_bg,
        sp_arrow_txt,
        // Bag Button
        bag_frame,
        bag_filling,
        bag_img_frame_bg,
        bag_img_frame,
        bag_img,
        bag_space_bg,
        bag_space,
        bag_numbers_alignment,
        bag_text_bg,
        bag_text,
        // Combo Counter
        combo_align,
        combo_bg,
        combo,
        // Slots
        m1_slot_bg,
        m1_content,
        m2_slot_bg,
        m2_content,
        slot1,
        slot1_text,
        slot1_text_bg,
        slot2,
        slot2_text,
        slot2_text_bg,
        slot3,
        slot3_text,
        slot3_text_bg,
        slot4,
        slot4_text,
        slot4_text_bg,
        slot5,
        slot5_text,
        slot5_text_bg,
        slot6,
        slot6_text,
        slot6_text_bg,
        slot7,
        slot7_text,
        slot7_text_bg,
        slot8,
        slot8_text,
        slot8_text_bg,
        slot9,
        slot9_text,
        slot9_text_bg,
        slot10,
        slot10_text,
        slot10_text_bg,
        slot11,
        slot11_text,
        slot11_text_bg,
        slot12,
        slot12_text,
        slot12_text_bg,
        slot13,
        slot13_text,
        slot13_text_bg,
        slot14,
        slot14_text,
        slot14_text_bg,
        slot15,
        slot15_text,
        slot15_text_bg,
        slot16,
        slot16_text,
        slot16_text_bg,
        slot17,
        slot17_text,
        slot17_text_bg,
        slot18,
        slot18_text,
        slot18_text_bg,
        slot19,
        slot19_text,
        slot19_text_bg,
        slot20,
        slot20_text,
        slot20_text_bg,
        slot_highlight,
        // WoW Pet Control Bar
        pet_info_name_bg,
        pet_info_name,

        pet_btn_summon,
        pet_btn_summon_icon,
        pet_btn_summon_border,
        pet_btn_summon_sc_bg,
        pet_btn_summon_sc,

        pet_btn_attack,
        pet_btn_attack_icon,
        pet_btn_attack_border,
        pet_btn_attack_sc_bg,
        pet_btn_attack_sc,

        pet_btn_follow,
        pet_btn_follow_icon,
        pet_btn_follow_border,
        pet_btn_follow_sc_bg,
        pet_btn_follow_sc,

        pet_btn_stay,
        pet_btn_stay_icon,
        pet_btn_stay_border,
        pet_btn_stay_sc_bg,
        pet_btn_stay_sc,

        pet_btn_heal,
        pet_btn_heal_icon,
        pet_btn_heal_border,
        pet_btn_heal_sc_bg,
        pet_btn_heal_sc,

        pet_btn_aggro,
        pet_btn_aggro_icon,
        pet_btn_aggro_border,
        pet_btn_aggro_sc_bg,
        pet_btn_aggro_sc,

        pet_btn_def,
        pet_btn_def_icon,
        pet_btn_def_border,
        pet_btn_def_sc_bg,
        pet_btn_def_sc,

        pet_btn_passive,
        pet_btn_passive_icon,
        pet_btn_passive_border,
        pet_btn_passive_sc_bg,
        pet_btn_passive_sc,
    }
}

#[derive(Clone, Copy)]
struct SlotEntry {
    slot: hotbar::Slot,
    widget_id: widget::Id,
    position: PositionSpecifier,
    game_input: Option<GameInput>,
    custom_shortcut: Option<&'static str>,
    shortcut_position: PositionSpecifier,
    shortcut_position_bg: PositionSpecifier,
    shortcut_widget_ids: (widget::Id, widget::Id),
}

fn slot_entries(state: &State, slot_offset: f64) -> [SlotEntry; 20] {
    use PositionSpecifier::*;

    [
        // 1th - 5th slots (Fila inferior izquierda)
        SlotEntry {
            slot: hotbar::Slot::One,
            widget_id: state.ids.slot1,
            position: BottomLeftWithMarginsOn(state.ids.frame, 0.0, 0.0),
            game_input: Some(GameInput::Slot1),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot1_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot1, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot1_text, state.ids.slot1_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Two,
            widget_id: state.ids.slot2,
            position: RightFrom(state.ids.slot1, slot_offset),
            game_input: Some(GameInput::Slot2),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot2_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot2, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot2_text, state.ids.slot2_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Three,
            widget_id: state.ids.slot3,
            position: RightFrom(state.ids.slot2, slot_offset),
            game_input: Some(GameInput::Slot3),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot3_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot3, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot3_text, state.ids.slot3_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Four,
            widget_id: state.ids.slot4,
            position: RightFrom(state.ids.slot3, slot_offset),
            game_input: Some(GameInput::Slot4),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot4_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot4, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot4_text, state.ids.slot4_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Five,
            widget_id: state.ids.slot5,
            position: RightFrom(state.ids.slot4, slot_offset),
            game_input: Some(GameInput::Slot5),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot5_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot5, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot5_text, state.ids.slot5_text_bg),
        },
        // 6th - 10th slots (Fila inferior derecha)
        SlotEntry {
            slot: hotbar::Slot::Six,
            widget_id: state.ids.slot6,
            position: RightFrom(state.ids.m2_slot_bg, slot_offset),
            game_input: Some(GameInput::Slot6),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot6_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot6, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot6_text, state.ids.slot6_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Seven,
            widget_id: state.ids.slot7,
            position: RightFrom(state.ids.slot6, slot_offset),
            game_input: Some(GameInput::Slot7),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot7_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot7, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot7_text, state.ids.slot7_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Eight,
            widget_id: state.ids.slot8,
            position: RightFrom(state.ids.slot7, slot_offset),
            game_input: Some(GameInput::Slot8),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot8_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot8, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot8_text, state.ids.slot8_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Nine,
            widget_id: state.ids.slot9,
            position: RightFrom(state.ids.slot8, slot_offset),
            game_input: Some(GameInput::Slot9),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot9_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot9, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot9_text, state.ids.slot9_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Ten,
            widget_id: state.ids.slot10,
            position: RightFrom(state.ids.slot9, slot_offset),
            game_input: Some(GameInput::Slot10),
            custom_shortcut: None,
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot10_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot10, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot10_text, state.ids.slot10_text_bg),
        },
        // 11th - 15th slots (Fila superior izquierda estilo WoW)
        SlotEntry {
            slot: hotbar::Slot::Eleven,
            widget_id: state.ids.slot11,
            position: UpFrom(state.ids.slot1, 4.0),
            game_input: None,
            custom_shortcut: Some("S1"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot11_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot11, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot11_text, state.ids.slot11_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Twelve,
            widget_id: state.ids.slot12,
            position: RightFrom(state.ids.slot11, slot_offset),
            game_input: None,
            custom_shortcut: Some("S2"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot12_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot12, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot12_text, state.ids.slot12_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Thirteen,
            widget_id: state.ids.slot13,
            position: RightFrom(state.ids.slot12, slot_offset),
            game_input: None,
            custom_shortcut: Some("S3"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot13_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot13, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot13_text, state.ids.slot13_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Fourteen,
            widget_id: state.ids.slot14,
            position: RightFrom(state.ids.slot13, slot_offset),
            game_input: None,
            custom_shortcut: Some("S4"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot14_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot14, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot14_text, state.ids.slot14_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Fifteen,
            widget_id: state.ids.slot15,
            position: RightFrom(state.ids.slot14, slot_offset),
            game_input: None,
            custom_shortcut: Some("S5"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot15_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot15, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot15_text, state.ids.slot15_text_bg),
        },
        // 16th - 20th slots (Fila superior derecha estilo WoW)
        SlotEntry {
            slot: hotbar::Slot::Sixteen,
            widget_id: state.ids.slot16,
            position: UpFrom(state.ids.slot6, 4.0),
            game_input: None,
            custom_shortcut: Some("S6"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot16_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot16, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot16_text, state.ids.slot16_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Seventeen,
            widget_id: state.ids.slot17,
            position: RightFrom(state.ids.slot16, slot_offset),
            game_input: None,
            custom_shortcut: Some("S7"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot17_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot17, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot17_text, state.ids.slot17_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Eighteen,
            widget_id: state.ids.slot18,
            position: RightFrom(state.ids.slot17, slot_offset),
            game_input: None,
            custom_shortcut: Some("S8"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot18_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot18, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot18_text, state.ids.slot18_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Nineteen,
            widget_id: state.ids.slot19,
            position: RightFrom(state.ids.slot18, slot_offset),
            game_input: None,
            custom_shortcut: Some("S9"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot19_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot19, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot19_text, state.ids.slot19_text_bg),
        },
        SlotEntry {
            slot: hotbar::Slot::Twenty,
            widget_id: state.ids.slot20,
            position: RightFrom(state.ids.slot19, slot_offset),
            game_input: None,
            custom_shortcut: Some("S0"),
            shortcut_position: BottomLeftWithMarginsOn(state.ids.slot20_text_bg, 1.0, 1.0),
            shortcut_position_bg: TopRightWithMarginsOn(state.ids.slot20, 3.0, 5.0),
            shortcut_widget_ids: (state.ids.slot20_text, state.ids.slot20_text_bg),
        },
    ]
}

pub enum Event {
    OpenDiary(SkillGroupKind),
    OpenBag,
    CommandPet(comp::PetCommand),
    UseHotbarSlot(hotbar::Slot),
    UsePrimaryAbility,
    UseSecondaryAbility,
}

#[derive(WidgetCommon)]
pub struct Skillbar<'a> {
    client: &'a Client,
    info: &'a HudInfo<'a>,
    global_state: &'a GlobalState,
    imgs: &'a Imgs,
    item_imgs: &'a ItemImgs,
    fonts: &'a Fonts,
    rot_imgs: &'a ImgsRot,
    health: &'a Health,
    inventory: &'a Inventory,
    energy: &'a Energy,
    poise: &'a Poise,
    skillset: &'a SkillSet,
    active_abilities: Option<&'a ActiveAbilities>,
    body: &'a Body,
    hotbar: &'a hotbar::State,
    tooltip_manager: &'a mut TooltipManager,
    item_tooltip_manager: &'a mut ItemTooltipManager,
    slot_manager: &'a mut slots::SlotManager,
    localized_strings: &'a Localization,
    item_i18n: &'a ItemI18n,
    pulse: f32,
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    msm: &'a MaterialStatManifest,
    rbm: &'a RecipeBookManifest,
    combo_floater: Option<ComboFloater>,
    combo: Option<&'a Combo>,
    char_state: Option<&'a CharacterState>,
    stance: Option<&'a Stance>,
    stats: Option<&'a Stats>,
    buffs: Option<&'a Buffs>,
}

impl<'a> Skillbar<'a> {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        client: &'a Client,
        info: &'a HudInfo,
        global_state: &'a GlobalState,
        imgs: &'a Imgs,
        item_imgs: &'a ItemImgs,
        fonts: &'a Fonts,
        rot_imgs: &'a ImgsRot,
        health: &'a Health,
        inventory: &'a Inventory,
        energy: &'a Energy,
        poise: &'a Poise,
        skillset: &'a SkillSet,
        active_abilities: Option<&'a ActiveAbilities>,
        body: &'a Body,
        pulse: f32,
        hotbar: &'a hotbar::State,
        tooltip_manager: &'a mut TooltipManager,
        item_tooltip_manager: &'a mut ItemTooltipManager,
        slot_manager: &'a mut slots::SlotManager,
        localized_strings: &'a Localization,
        item_i18n: &'a ItemI18n,
        msm: &'a MaterialStatManifest,
        rbm: &'a RecipeBookManifest,
        combo_floater: Option<ComboFloater>,
        combo: Option<&'a Combo>,
        char_state: Option<&'a CharacterState>,
        stance: Option<&'a Stance>,
        stats: Option<&'a Stats>,
        buffs: Option<&'a Buffs>,
    ) -> Self {
        Self {
            client,
            info,
            global_state,
            imgs,
            item_imgs,
            fonts,
            rot_imgs,
            health,
            inventory,
            energy,
            poise,
            skillset,
            active_abilities,
            body,
            common: widget::CommonBuilder::default(),
            pulse,
            hotbar,
            tooltip_manager,
            item_tooltip_manager,
            slot_manager,
            localized_strings,
            item_i18n,
            msm,
            rbm,
            combo_floater,
            combo,
            char_state,
            stance,
            stats,
            buffs,
        }
    }

    fn create_new_button_with_shadow(
        &self,
        ui: &mut UiCell,
        key_mouse: &KeyMouse,
        button_identifier: widget::Id,
        text_background: widget::Id,
        text: widget::Id,
    ) {
        let key_desc = key_mouse.display_shortest();

        //Create shadow
        Text::new(&key_desc)
            .bottom_right_with_margins_on(button_identifier, 0.0, 0.0)
            .font_size(10)
            .font_id(self.fonts.cyri.conrod_id)
            .color(BLACK)
            .set(text_background, ui);

        //Create button
        Text::new(&key_desc)
            .bottom_right_with_margins_on(text_background, 1.0, 1.0)
            .font_size(10)
            .font_id(self.fonts.cyri.conrod_id)
            .color(TEXT_COLOR)
            .set(text, ui);
    }

    fn show_give_up_message(&self, state: &State, ui: &mut UiCell) {
        let localized_strings = self.localized_strings;
        let hardcore = self.client.current::<Hardcore>().is_some();

        if let Some(key) = self
            .global_state
            .settings
            .controls
            .get_binding(GameInput::GiveUp)
        {
            let respawn_msg =
                localized_strings.get_msg_ctx("hud-press_key_to_give_up", &i18n::fluent_args! {
                    "key" => key.display_string()
                });
            let penalty_msg = if hardcore {
                self.localized_strings
                    .get_msg("hud-hardcore_will_char_deleted")
            } else {
                self.localized_strings.get_msg("hud-items_will_lose_dur")
            };

            let recieving_help_msg = localized_strings.get_msg("hud-downed_recieving_help");
            Text::new(&penalty_msg)
                .mid_bottom_with_margin_on(ui.window, 180.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.death_message_3_bg, ui);
            Text::new(&respawn_msg)
                .mid_top_with_margin_on(state.ids.death_message_3_bg, -50.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.death_message_2_bg, ui);
            Text::new(&penalty_msg)
                .bottom_left_with_margins_on(state.ids.death_message_3_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.death_message_3, ui);
            Text::new(&respawn_msg)
                .bottom_left_with_margins_on(state.ids.death_message_2_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.death_message_2, ui);
            if self
                .client
                .state()
                .read_storage::<common::interaction::Interactors>()
                .get(self.client.entity())
                .is_some_and(|interactors| {
                    interactors.has_interaction(common::interaction::InteractionKind::HelpDowned)
                })
            {
                Text::new(&recieving_help_msg)
                    .mid_top_with_margin_on(state.ids.death_message_2_bg, -50.0)
                    .font_size(self.fonts.cyri.scale(24))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                    .set(state.ids.death_message_1_bg, ui);
                Text::new(&recieving_help_msg)
                    .bottom_left_with_margins_on(state.ids.death_message_1_bg, 2.0, 2.0)
                    .font_size(self.fonts.cyri.scale(24))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(HP_COLOR)
                    .set(state.ids.death_message_1, ui);
            }
        }
    }

    fn show_riding_message(&self, state: &State, ui: &mut UiCell) {
        let prompt = if self.client.is_volume_controller() {
            "[E] Soltar timÃ³n"
        } else if self.client.is_volume_rider() {
            "[E] Levantarse del asiento"
        } else {
            "[E] Desmontar"
        };

        Text::new(prompt)
            .mid_bottom_with_margin_on(ui.window, 185.0)
            .font_size(self.fonts.cyri.scale(20))
            .font_id(self.fonts.cyri.conrod_id)
            .color(Color::Rgba(0.0, 0.0, 0.0, 0.85))
            .set(state.ids.riding_prompt_bg, ui);

        Text::new(prompt)
            .bottom_left_with_margins_on(state.ids.riding_prompt_bg, 2.0, 2.0)
            .font_size(self.fonts.cyri.scale(20))
            .font_id(self.fonts.cyri.conrod_id)
            .color(TEXT_COLOR)
            .set(state.ids.riding_prompt, ui);
    }

    fn show_death_message(&self, state: &State, ui: &mut UiCell) {
        let localized_strings = self.localized_strings;
        let hardcore = self.client.current::<Hardcore>().is_some();

        if let Some(key) = self
            .global_state
            .settings
            .controls
            .get_binding(GameInput::Respawn)
        {
            Text::new(&self.localized_strings.get_msg("hud-you_died"))
                .middle_of(ui.window)
                .font_size(self.fonts.cyri.scale(50))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.death_message_1_bg, ui);
            let respawn_msg = if hardcore {
                localized_strings.get_msg_ctx(
                    "hud-press_key_to_return_to_char_menu",
                    &i18n::fluent_args! {
                        "key" => key.display_string()
                    },
                )
            } else {
                localized_strings.get_msg_ctx("hud-press_key_to_respawn", &i18n::fluent_args! {
                    "key" => key.display_string()
                })
            };
            let penalty_msg = if hardcore {
                self.localized_strings.get_msg("hud-hardcore_char_deleted")
            } else {
                self.localized_strings.get_msg("hud-items_lost_dur")
            };
            Text::new(&respawn_msg)
                .mid_bottom_with_margin_on(state.ids.death_message_1_bg, -120.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.death_message_2_bg, ui);
            Text::new(&penalty_msg)
                .mid_bottom_with_margin_on(state.ids.death_message_2_bg, -50.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.death_message_3_bg, ui);
            Text::new(&self.localized_strings.get_msg("hud-you_died"))
                .bottom_left_with_margins_on(state.ids.death_message_1_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(50))
                .font_id(self.fonts.cyri.conrod_id)
                .color(CRITICAL_HP_COLOR)
                .set(state.ids.death_message_1, ui);
            Text::new(&respawn_msg)
                .bottom_left_with_margins_on(state.ids.death_message_2_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(CRITICAL_HP_COLOR)
                .set(state.ids.death_message_2, ui);
            Text::new(&penalty_msg)
                .bottom_left_with_margins_on(state.ids.death_message_3_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(CRITICAL_HP_COLOR)
                .set(state.ids.death_message_3, ui);
        }
    }

    fn show_wow_player_frame(&self, state: &State, ui: &mut UiCell) {
        let (hp_percentage, energy_percentage) = if self.health.is_dead {
            (0.0, 0.0)
        } else {
            let max_hp = f64::from(self.health.base_max().max(self.health.maximum()));
            let current_hp = f64::from(self.health.current());
            (
                (current_hp / max_hp * 100.0).clamp(0.0, 100.0),
                f64::from(self.energy.fraction() * 100.0).clamp(0.0, 100.0),
            )
        };

        let hp_ani = (self.pulse * 4.0).cos() * 0.5 + 0.8;
        let crit_hp_color = Color::Rgba(0.79, 0.19, 0.17, hp_ani);
        let health_col = match hp_percentage as u8 {
            0..=20 => crit_hp_color,
            21..=40 => LOW_HP_COLOR,
            _ => HP_COLOR,
        };

        let selected_experience = &self
            .global_state
            .settings
            .interface
            .xp_bar_skillgroup
            .unwrap_or(SkillGroupKind::General);
        let current_exp = self.skillset.available_experience(*selected_experience) as f64;
        let max_exp = self.skillset.skill_point_cost(*selected_experience) as f64;
        let exp_percentage = (current_exp / max_exp.max(1.0)).clamp(0.0, 1.0);
        let level = (1 + self.skillset.earned_sp(*selected_experience) as u32).min(60);
        let level_txt = level.to_string();

        let char_name = self.stats.map_or_else(
            || "Aventurero".to_string(),
            |s| self.localized_strings.get_content(&s.name),
        );

        let portrait_img = match self.body {
            comp::Body::Humanoid(h) => {
                use comp::humanoid::{BodyType, Species};
                match (h.species, h.body_type) {
                    (Species::Human, BodyType::Male) => self.imgs.portrait_human_m,
                    (Species::Human, BodyType::Female) => self.imgs.portrait_human_f,
                    (Species::Orc, BodyType::Male) => self.imgs.portrait_orc_m,
                    (Species::Orc, BodyType::Female) => self.imgs.portrait_orc_f,
                    (Species::Dwarf, BodyType::Male) => self.imgs.portrait_dwarf_m,
                    (Species::Dwarf, BodyType::Female) => self.imgs.portrait_dwarf_f,
                    (Species::Draugr, BodyType::Male) => self.imgs.portrait_draugr_m,
                    (Species::Draugr, BodyType::Female) => self.imgs.portrait_draugr_f,
                    (Species::Elf, BodyType::Male) => self.imgs.portrait_elf_m,
                    (Species::Elf, BodyType::Female) => self.imgs.portrait_elf_f,
                    (Species::Danari, BodyType::Male) => self.imgs.portrait_danari_m,
                    (Species::Danari, BodyType::Female) => self.imgs.portrait_danari_f,
                }
            },
            _ => self.imgs.portrait_human_m,
        };

        let hp_txt = if self.health.is_dead {
            "MUERTO".to_string()
        } else {
            format!(
                "{}/{}",
                self.health.current().round() as u32,
                self.health.maximum().round() as u32
            )
        };
        let mana_txt = format!(
            "{}/{}",
            self.energy.current().round() as u32,
            self.energy.maximum().round() as u32
        );

        // 1. Marco exterior con borde dorado/metÃ¡lico estilo WoW
        RoundedRectangle::fill_with([244.0, 70.0], 8.0, Color::Rgba(0.55, 0.45, 0.20, 0.95))
            .top_left_with_margins_on(ui.window, 16.0, 16.0)
            .set(state.ids.player_frame_border, ui);

        // Fondo interior oscuro del panel
        RoundedRectangle::fill_with([240.0, 66.0], 6.0, Color::Rgba(0.07, 0.08, 0.11, 0.90))
            .middle_of(state.ids.player_frame_border)
            .set(state.ids.player_frame_bg, ui);

        // 2. MedallÃ³n Circular de Retrato
        RoundedRectangle::fill_with([56.0, 56.0], 28.0, Color::Rgba(0.85, 0.70, 0.22, 1.0))
            .top_left_with_margins_on(state.ids.player_frame_bg, 5.0, 6.0)
            .set(state.ids.player_portrait_border, ui);

        RoundedRectangle::fill_with([52.0, 52.0], 26.0, Color::Rgba(0.05, 0.05, 0.07, 1.0))
            .middle_of(state.ids.player_portrait_border)
            .set(state.ids.player_portrait_bg, ui);

        Image::new(portrait_img)
            .w_h(48.0, 48.0)
            .middle_of(state.ids.player_portrait_bg)
            .set(state.ids.player_portrait, ui);

        // 3. Insignia Circular de Nivel (Esquina inferior izquierda del retrato)
        RoundedRectangle::fill_with([22.0, 22.0], 11.0, Color::Rgba(0.95, 0.82, 0.25, 1.0))
            .bottom_left_with_margins_on(state.ids.player_portrait_border, -3.0, -3.0)
            .set(state.ids.player_level_badge_border, ui);

        RoundedRectangle::fill_with([18.0, 18.0], 9.0, Color::Rgba(0.12, 0.10, 0.04, 1.0))
            .middle_of(state.ids.player_level_badge_border)
            .set(state.ids.player_level_badge_bg, ui);

        Text::new(&level_txt)
            .middle_of(state.ids.player_level_badge_bg)
            .font_size(11)
            .font_id(self.fonts.cyri.conrod_id)
            .color(BLACK)
            .set(state.ids.player_level_txt_bg, ui);
        Text::new(&level_txt)
            .bottom_right_with_margins_on(state.ids.player_level_txt_bg, 1.0, 1.0)
            .font_size(11)
            .font_id(self.fonts.cyri.conrod_id)
            .color(Color::Rgba(1.0, 0.95, 0.40, 1.0))
            .set(state.ids.player_level_txt, ui);

        // 4. Nombre del Personaje
        Text::new(&char_name)
            .top_left_with_margins_on(state.ids.player_frame_bg, 5.0, 68.0)
            .font_size(13)
            .font_id(self.fonts.cyri.conrod_id)
            .color(BLACK)
            .set(state.ids.player_name_txt_bg, ui);
        Text::new(&char_name)
            .bottom_right_with_margins_on(state.ids.player_name_txt_bg, 1.0, 1.0)
            .font_size(13)
            .font_id(self.fonts.cyri.conrod_id)
            .color(Color::Rgba(1.0, 0.95, 0.85, 1.0))
            .set(state.ids.player_name_txt, ui);

        // 5. Barra de Vida (Health Bar)
        RoundedRectangle::fill_with([160.0, 15.0], 2.0, Color::Rgba(0.04, 0.04, 0.04, 0.95))
            .top_left_with_margins_on(state.ids.player_frame_bg, 22.0, 68.0)
            .set(state.ids.player_hp_bg, ui);
        Image::new(self.imgs.bar_content)
            .w_h(158.0 * (hp_percentage / 100.0).clamp(0.0, 1.0), 13.0)
            .color(Some(health_col))
            .top_left_with_margins_on(state.ids.player_hp_bg, 1.0, 1.0)
            .set(state.ids.player_hp_fill, ui);
        let decayed_health = 1.0 - self.health.maximum() as f64 / self.health.base_max() as f64;
        if decayed_health > 0.0 {
            Image::new(self.imgs.bar_content)
                .w_h(158.0 * decayed_health.clamp(0.0, 1.0), 13.0)
                .color(Some(QUALITY_EPIC))
                .top_right_with_margins_on(state.ids.player_hp_bg, 1.0, 1.0)
                .set(state.ids.player_hp_decay, ui);
        }
        Text::new(&hp_txt)
            .middle_of(state.ids.player_hp_bg)
            .font_size(10)
            .font_id(self.fonts.cyri.conrod_id)
            .color(BLACK)
            .set(state.ids.player_hp_txt_bg, ui);
        Text::new(&hp_txt)
            .bottom_right_with_margins_on(state.ids.player_hp_txt_bg, 1.0, 1.0)
            .font_size(10)
            .font_id(self.fonts.cyri.conrod_id)
            .color(Color::Rgba(1.0, 1.0, 1.0, 0.95))
            .set(state.ids.player_hp_txt, ui);

        // 6. Barra de ManÃ¡ / EnergÃ­a (Mana Bar)
        RoundedRectangle::fill_with([160.0, 12.0], 2.0, Color::Rgba(0.04, 0.04, 0.04, 0.95))
            .top_left_with_margins_on(state.ids.player_frame_bg, 39.0, 68.0)
            .set(state.ids.player_mana_bg, ui);
        Image::new(self.imgs.bar_content)
            .w_h(158.0 * (energy_percentage / 100.0).clamp(0.0, 1.0), 10.0)
            .color(Some(Color::Rgba(0.18, 0.48, 0.92, 1.0)))
            .top_left_with_margins_on(state.ids.player_mana_bg, 1.0, 1.0)
            .set(state.ids.player_mana_fill, ui);
        Text::new(&mana_txt)
            .middle_of(state.ids.player_mana_bg)
            .font_size(9)
            .font_id(self.fonts.cyri.conrod_id)
            .color(BLACK)
            .set(state.ids.player_mana_txt_bg, ui);
        Text::new(&mana_txt)
            .bottom_right_with_margins_on(state.ids.player_mana_txt_bg, 1.0, 1.0)
            .font_size(9)
            .font_id(self.fonts.cyri.conrod_id)
            .color(Color::Rgba(0.9, 0.95, 1.0, 0.95))
            .set(state.ids.player_mana_txt, ui);

        // 7. PequeÃ±a Barra de Nivel / Exp (Mini XP Bar)
        RoundedRectangle::fill_with([160.0, 6.0], 1.0, Color::Rgba(0.04, 0.04, 0.04, 0.95))
            .top_left_with_margins_on(state.ids.player_frame_bg, 53.0, 68.0)
            .set(state.ids.player_xp_bg, ui);
        Image::new(self.imgs.bar_content)
            .w_h(158.0 * exp_percentage, 4.0)
            .color(Some(Color::Rgba(0.68, 0.32, 0.88, 1.0)))
            .top_left_with_margins_on(state.ids.player_xp_bg, 1.0, 1.0)
            .set(state.ids.player_xp_fill, ui);
    }

    fn show_stat_bars(&self, state: &State, ui: &mut UiCell, events: &mut Vec<Event>) {
        let (hp_percentage, energy_percentage, poise_percentage): (f64, f64, f64) =
            if self.health.is_dead {
                (0.0, 0.0, 0.0)
            } else {
                let max_hp = f64::from(self.health.base_max().max(self.health.maximum()));
                let current_hp = f64::from(self.health.current());
                (
                    current_hp / max_hp * 100.0,
                    f64::from(self.energy.fraction() * 100.0),
                    f64::from(self.poise.fraction() * 100.0),
                )
            };

        // Animation timer
        let hp_ani = (self.pulse * 4.0/* speed factor */).cos() * 0.5 + 0.8;
        let crit_hp_color: Color = Color::Rgba(0.79, 0.19, 0.17, hp_ani);
        let bar_values = self.global_state.settings.interface.bar_numbers;
        let is_downed = is_downed(Some(self.health), self.char_state);
        let show_health = self.global_state.settings.interface.always_show_bars
            || is_downed
            || (self.health.current() - self.health.maximum()).abs() > Health::HEALTH_EPSILON;
        let show_energy = self.global_state.settings.interface.always_show_bars
            || (self.energy.current() - self.energy.maximum()).abs() > Energy::ENERGY_EPSILON;
        let show_poise = self.global_state.settings.interface.enable_poise_bar
            && (self.global_state.settings.interface.always_show_bars
                || (self.poise.current() - self.poise.maximum()).abs() > Poise::POISE_EPSILON);
        let decayed_health = 1.0 - self.health.maximum() as f64 / self.health.base_max() as f64;

        if show_health && !self.health.is_dead || decayed_health > 0.0 {
            let offset = 1.0;
            let hp_percentage = if is_downed {
                100.0
                    * (1.0 - self.info.key_state.give_up.unwrap_or(0.0) / GIVE_UP_HOLD_TIME)
                        .clamp(0.0, 1.0) as f64
            } else {
                hp_percentage
            };

            Image::new(self.imgs.health_bg)
                .w_h(484.0, 24.0)
                .mid_top_with_margin_on(state.ids.frame, -offset)
                .set(state.ids.bg_health, ui);
            Rectangle::fill_with([480.0, 18.0], color::TRANSPARENT)
                .top_left_with_margins_on(state.ids.bg_health, 2.0, 2.0)
                .set(state.ids.hp_alignment, ui);
            let health_col = match hp_percentage as u8 {
                _ if is_downed => crit_hp_color,
                0..=20 => crit_hp_color,
                21..=40 => LOW_HP_COLOR,
                _ => HP_COLOR,
            };
            Image::new(self.imgs.bar_content)
                .w_h(480.0 * hp_percentage / 100.0, 18.0)
                .color(Some(health_col))
                .top_left_with_margins_on(state.ids.hp_alignment, 0.0, 0.0)
                .set(state.ids.hp_filling, ui);

            if decayed_health > 0.0 {
                let decay_bar_len = 480.0 * decayed_health;
                Image::new(self.imgs.bar_content)
                    .w_h(decay_bar_len, 18.0)
                    .color(Some(QUALITY_EPIC))
                    .top_right_with_margins_on(state.ids.hp_alignment, 0.0, 0.0)
                    .crop_kids()
                    .set(state.ids.hp_decayed, ui);

                Image::new(self.imgs.decayed_bg)
                    .w_h(480.0, 18.0)
                    .color(Some(Color::Rgba(0.58, 0.29, 0.93, (hp_ani + 0.6).min(1.0))))
                    .top_left_with_margins_on(state.ids.hp_alignment, 0.0, 0.0)
                    .parent(state.ids.hp_decayed)
                    .set(state.ids.decay_overlay, ui);
            }
            Image::new(self.imgs.health_frame)
                .w_h(484.0, 24.0)
                .color(Some(UI_HIGHLIGHT_0))
                .middle_of(state.ids.bg_health)
                .set(state.ids.frame_health, ui);
        }
        if show_energy && !self.health.is_dead {
            let offset = if show_health || decayed_health > 0.0 {
                33.0
            } else {
                1.0
            };
            Image::new(self.imgs.energy_bg)
                .w_h(323.0, 16.0)
                .mid_top_with_margin_on(state.ids.frame, -offset)
                .set(state.ids.bg_energy, ui);
            Rectangle::fill_with([319.0, 10.0], color::TRANSPARENT)
                .top_left_with_margins_on(state.ids.bg_energy, 2.0, 2.0)
                .set(state.ids.energy_alignment, ui);
            Image::new(self.imgs.bar_content)
                .w_h(319.0 * energy_percentage / 100.0, 10.0)
                .color(Some(STAMINA_COLOR))
                .top_left_with_margins_on(state.ids.energy_alignment, 0.0, 0.0)
                .set(state.ids.energy_filling, ui);
            Image::new(self.imgs.energy_frame)
                .w_h(323.0, 16.0)
                .color(Some(UI_HIGHLIGHT_0))
                .middle_of(state.ids.bg_energy)
                .set(state.ids.frame_energy, ui);
        }
        if show_poise && !self.health.is_dead {
            let offset = 16.0;

            let poise_colour = match self.poise.previous_state {
                self::PoiseState::KnockedDown => BLACK,
                self::PoiseState::Dazed => Color::Rgba(0.25, 0.0, 0.15, 1.0),
                self::PoiseState::Stunned => Color::Rgba(0.40, 0.0, 0.30, 1.0),
                self::PoiseState::Interrupted => Color::Rgba(0.55, 0.0, 0.45, 1.0),
                _ => POISE_COLOR,
            };

            Image::new(self.imgs.poise_bg)
                .w_h(323.0, 14.0)
                .mid_top_with_margin_on(state.ids.frame, -offset)
                .set(state.ids.bg_poise, ui);
            Rectangle::fill_with([319.0, 10.0], color::TRANSPARENT)
                .top_left_with_margins_on(state.ids.bg_poise, 2.0, 2.0)
                .set(state.ids.poise_alignment, ui);
            Image::new(self.imgs.bar_content)
                .w_h(319.0 * poise_percentage / 100.0, 10.0)
                .color(Some(poise_colour))
                .top_left_with_margins_on(state.ids.poise_alignment, 0.0, 0.0)
                .set(state.ids.poise_filling, ui);
            for i in 0..state.ids.poise_ticks.len() {
                Image::new(self.imgs.poise_tick)
                    .w_h(3.0, 10.0)
                    .color(Some(POISEBAR_TICK_COLOR))
                    .top_left_with_margins_on(
                        state.ids.poise_alignment,
                        0.0,
                        319.0f64 * (self::Poise::POISE_THRESHOLDS[i] / self.poise.maximum()) as f64,
                    )
                    .set(state.ids.poise_ticks[i], ui);
            }
            Image::new(self.imgs.poise_frame)
                .w_h(323.0, 16.0)
                .color(Some(UI_HIGHLIGHT_0))
                .middle_of(state.ids.bg_poise)
                .set(state.ids.frame_poise, ui);
        }
        // Bag button and indicator
        Image::new(self.imgs.selected_exp_bg)
            .w_h(34.0, 38.0)
            .bottom_right_with_margins_on(state.ids.slot10, 0.0, -37.0)
            .color(Some(Color::Rgba(1.0, 1.0, 1.0, 1.0)))
            .set(state.ids.bag_img_frame_bg, ui);

        if Button::image(self.imgs.bag_frame)
            .w_h(34.0, 38.0)
            .middle_of(state.ids.bag_img_frame_bg)
            .set(state.ids.bag_img_frame, ui)
            .was_clicked()
        {
            events.push(Event::OpenBag);
        }
        let inventory = self.inventory;

        let space_used = inventory.populated_slots();
        let space_max = inventory.slots().count();
        let bag_space = format!("{}/{}", space_used, space_max);
        let bag_space_percentage = space_used as f64 / space_max as f64;

        // bag filling indicator bar
        Image::new(self.imgs.bar_content)
            .w_h(1.0, 21.0 * bag_space_percentage)
            .color(if bag_space_percentage < 0.6 {
                Some(TEXT_VELORITE)
            } else if bag_space_percentage < 1.0 {
                Some(LOW_HP_COLOR)
            } else {
                Some(CRITICAL_HP_COLOR)
            })
            .graphics_for(state.ids.bag_img_frame)
            .bottom_left_with_margins_on(state.ids.bag_img_frame, 14.0, 2.0)
            .set(state.ids.bag_filling, ui);

        // bag filling text
        Rectangle::fill_with([32.0, 11.0], color::TRANSPARENT)
            .bottom_left_with_margins_on(state.ids.bag_img_frame_bg, 1.0, 2.0)
            .graphics_for(state.ids.bag_img_frame)
            .set(state.ids.bag_numbers_alignment, ui);
        Text::new(&bag_space)
            .middle_of(state.ids.bag_numbers_alignment)
            .font_size(if bag_space.len() < 6 { 9 } else { 8 })
            .font_id(self.fonts.cyri.conrod_id)
            .color(BLACK)
            .graphics_for(state.ids.bag_img_frame)
            .set(state.ids.bag_space_bg, ui);
        Text::new(&bag_space)
            .bottom_right_with_margins_on(state.ids.bag_space_bg, 1.0, 1.0)
            .font_size(if bag_space.len() < 6 { 9 } else { 8 })
            .font_id(self.fonts.cyri.conrod_id)
            .color(if bag_space_percentage < 0.6 {
                TEXT_VELORITE
            } else if bag_space_percentage < 1.0 {
                LOW_HP_COLOR
            } else {
                CRITICAL_HP_COLOR
            })
            .graphics_for(state.ids.bag_img_frame)
            .set(state.ids.bag_space, ui);

        Image::new(self.imgs.bag_ico)
            .w_h(24.0, 24.0)
            .graphics_for(state.ids.bag_img_frame)
            .mid_bottom_with_margin_on(state.ids.bag_img_frame, 13.0)
            .set(state.ids.bag_img, ui);

        if let Some(bag) = &self
            .global_state
            .settings
            .controls
            .get_binding(GameInput::Inventory)
        {
            self.create_new_button_with_shadow(
                ui,
                bag,
                state.ids.bag_img,
                state.ids.bag_text_bg,
                state.ids.bag_text,
            );
        }

        // Exp Type and Level Display

        // Unspent SP indicator (only show if we can spend SP)
        let unspent_sp = self.skillset.can_unlock_any_skill();
        if unspent_sp {
            let arrow_ani = animation_timer(self.pulse); //Animation timer
            Image::new(self.imgs.sp_indicator_arrow)
                .w_h(20.0, 11.0)
                .graphics_for(state.ids.exp_img_frame)
                .mid_top_with_margin_on(state.ids.exp_img_frame, -12.0 + arrow_ani as f64)
                .color(Some(QUALITY_LEGENDARY))
                .set(state.ids.sp_arrow, ui);
            Text::new(&self.localized_strings.get_msg("hud-sp_arrow_txt"))
                .mid_top_with_margin_on(state.ids.sp_arrow, -18.0)
                .graphics_for(state.ids.exp_img_frame)
                .font_id(self.fonts.cyri.conrod_id)
                .font_size(self.fonts.cyri.scale(14))
                .color(BLACK)
                .set(state.ids.sp_arrow_txt_bg, ui);
            Text::new(&self.localized_strings.get_msg("hud-sp_arrow_txt"))
                .graphics_for(state.ids.exp_img_frame)
                .bottom_right_with_margins_on(state.ids.sp_arrow_txt_bg, 1.0, 1.0)
                .font_id(self.fonts.cyri.conrod_id)
                .font_size(self.fonts.cyri.scale(14))
                .color(QUALITY_LEGENDARY)
                .set(state.ids.sp_arrow_txt, ui);
        }

        if self
            .global_state
            .settings
            .interface
            .xp_bar_skillgroup
            .is_some()
        {
            let offset = -81.0;
            let selected_experience = &self
                .global_state
                .settings
                .interface
                .xp_bar_skillgroup
                .unwrap_or(SkillGroupKind::General);
            let current_exp = self.skillset.available_experience(*selected_experience) as f64;
            let max_exp = self.skillset.skill_point_cost(*selected_experience) as f64;
            let exp_percentage = current_exp / max_exp.max(1.0);
            let level = (1 + self.skillset.earned_sp(*selected_experience) as u32).min(60);
            let level_txt = level.to_string();

            // Exp Bar
            Image::new(self.imgs.exp_frame_bg)
                .w_h(594.0, 8.0)
                .mid_top_with_margin_on(state.ids.frame, -offset)
                .color(Some(Color::Rgba(1.0, 1.0, 1.0, 0.9)))
                .set(state.ids.exp_frame_bg, ui);
            Image::new(self.imgs.exp_frame)
                .w_h(594.0, 8.0)
                .middle_of(state.ids.exp_frame_bg)
                .set(state.ids.exp_frame, ui);

            Image::new(self.imgs.bar_content)
                .w_h(590.0 * exp_percentage, 4.0)
                .color(Some(XP_COLOR))
                .top_left_with_margins_on(state.ids.exp_frame, 2.0, 2.0)
                .set(state.ids.exp_filling, ui);
            // Exp Type and Level Display
            Image::new(self.imgs.selected_exp_bg)
                .w_h(34.0, 38.0)
                .top_left_with_margins_on(state.ids.exp_frame, -39.0, 3.0)
                .color(Some(Color::Rgba(1.0, 1.0, 1.0, 1.0)))
                .set(state.ids.exp_img_frame_bg, ui);

            if Button::image(self.imgs.selected_exp)
                .w_h(34.0, 38.0)
                .middle_of(state.ids.exp_img_frame_bg)
                .set(state.ids.exp_img_frame, ui)
                .was_clicked()
            {
                events.push(Event::OpenDiary(*selected_experience));
            }

            Text::new(&level_txt)
                .mid_bottom_with_margin_on(state.ids.exp_img_frame, 2.0)
                .font_size(11)
                .font_id(self.fonts.cyri.conrod_id)
                .color(QUALITY_LEGENDARY)
                .graphics_for(state.ids.exp_img_frame)
                .set(state.ids.exp_lvl, ui);

            Image::new(match selected_experience {
                SkillGroupKind::General => self.imgs.swords_crossed,
                SkillGroupKind::Weapon(ToolKind::Sword) => self.imgs.sword,
                SkillGroupKind::Weapon(ToolKind::Hammer) => self.imgs.hammer,
                SkillGroupKind::Weapon(ToolKind::Axe) => self.imgs.axe,
                SkillGroupKind::Weapon(ToolKind::Sceptre) => self.imgs.sceptre,
                SkillGroupKind::Weapon(ToolKind::Bow) => self.imgs.bow,
                SkillGroupKind::Weapon(ToolKind::Staff) => self.imgs.staff,
                SkillGroupKind::Weapon(ToolKind::Pick) => self.imgs.mining,
                SkillGroupKind::Tamer => self.imgs.tamer_class,
                _ => self.imgs.nothing,
            })
            .w_h(24.0, 24.0)
            .graphics_for(state.ids.exp_img_frame)
            .mid_bottom_with_margin_on(state.ids.exp_img_frame, 13.0)
            .set(state.ids.exp_img, ui);

            // Show Shortcut
            if let Some(diary) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Diary)
            {
                self.create_new_button_with_shadow(
                    ui,
                    diary,
                    state.ids.exp_img,
                    state.ids.diary_txt_bg,
                    state.ids.diary_txt,
                );
            }
        } else {
            // Only show Spellbook ico
            Image::new(self.imgs.selected_exp_bg)
                .w_h(34.0, 38.0)
                .bottom_left_with_margins_on(state.ids.slot1, 0.0, -37.0)
                .color(Some(Color::Rgba(1.0, 1.0, 1.0, 1.0)))
                .set(state.ids.exp_img_frame_bg, ui);

            if Button::image(self.imgs.selected_exp)
                .w_h(34.0, 38.0)
                .middle_of(state.ids.exp_img_frame_bg)
                .set(state.ids.exp_img_frame, ui)
                .was_clicked()
            {
                events.push(Event::OpenDiary(SkillGroupKind::General));
            }

            Image::new(self.imgs.spellbook_ico0)
                .w_h(24.0, 24.0)
                .graphics_for(state.ids.exp_img_frame)
                .mid_bottom_with_margin_on(state.ids.exp_img_frame, 13.0)
                .set(state.ids.exp_img, ui);

            // Show Shortcut
            if let Some(diary) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Diary)
            {
                self.create_new_button_with_shadow(
                    ui,
                    diary,
                    state.ids.exp_img,
                    state.ids.diary_txt_bg,
                    state.ids.diary_txt,
                );
            }
        }

        // Bar Text
        let bar_text = if self.health.is_dead {
            Some((
                self.localized_strings
                    .get_msg("hud-group-dead")
                    .into_owned(),
                self.localized_strings
                    .get_msg("hud-group-dead")
                    .into_owned(),
                self.localized_strings
                    .get_msg("hud-group-dead")
                    .into_owned(),
            ))
        } else if let BarNumbers::Values = bar_values {
            Some((
                format!(
                    "{}/{}",
                    self.health.current().round().max(1.0) as u32, /* Don't show 0 health for
                                                                    * living players */
                    self.health.maximum().round() as u32
                ),
                format!(
                    "{}/{}",
                    self.energy.current().round() as u32,
                    self.energy.maximum().round() as u32
                ),
                String::new(), // Don't obscure the tick mark
            ))
        } else if let BarNumbers::Percent = bar_values {
            Some((
                format!("{}%", hp_percentage as u32),
                format!("{}%", energy_percentage as u32),
                String::new(), // Don't obscure the tick mark
            ))
        } else {
            None
        };
        if let Some((hp_txt, energy_txt, poise_txt)) = bar_text {
            let hp_txt = if is_downed { String::new() } else { hp_txt };

            Text::new(&hp_txt)
                .middle_of(state.ids.frame_health)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.hp_txt_bg, ui);
            Text::new(&hp_txt)
                .bottom_left_with_margins_on(state.ids.hp_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.hp_txt, ui);

            Text::new(&energy_txt)
                .middle_of(state.ids.frame_energy)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.energy_txt_bg, ui);
            Text::new(&energy_txt)
                .bottom_left_with_margins_on(state.ids.energy_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.energy_txt, ui);

            Text::new(&poise_txt)
                .middle_of(state.ids.frame_poise)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.poise_txt_bg, ui);
            Text::new(&poise_txt)
                .bottom_left_with_margins_on(state.ids.poise_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.poise_txt, ui);
        }
    }

    fn show_slotbar(
        &mut self,
        state: &State,
        ui: &mut UiCell,
        slot_offset: f64,
        events: &mut Vec<Event>,
    ) {
        let shortcuts = self.global_state.settings.interface.shortcut_numbers;

        // TODO: avoid this
        let content_source = (
            self.hotbar,
            self.inventory,
            self.energy,
            self.skillset,
            self.active_abilities,
            self.body,
            self.combo,
            self.char_state,
            self.stance,
            self.stats,
            self.buffs,
        );

        let image_source = (self.item_imgs, self.imgs);
        let mut slot_maker = SlotMaker {
            // TODO: is a separate image needed for the frame?
            empty_slot: self.imgs.skillbar_slot,
            hovered_slot: self.imgs.skillbar_index,
            filled_slot: self.imgs.skillbar_slot,
            selected_slot: self.imgs.inv_slot_sel,
            background_color: None,
            content_size: ContentSize {
                width_height_ratio: 1.0,
                max_fraction: 0.9, /* Changes the item image size by setting a maximum fraction
                                    * of either the width or height */
            },
            selected_content_scale: 1.0,
            amount_font: self.fonts.cyri.conrod_id,
            amount_margins: Vec2::new(1.0, 1.0),
            amount_font_size: self.fonts.cyri.scale(12),
            amount_text_color: TEXT_COLOR,
            content_source: &content_source,
            image_source: &image_source,
            slot_manager: Some(self.slot_manager),
            global_state: self.global_state,
            pulse: self.pulse,
        };

        // Tooltips
        let tooltip = Tooltip::new({
            // Edge images [t, b, r, l]
            // Corner images [tr, tl, br, bl]
            let edge = &self.rot_imgs.tt_side;
            let corner = &self.rot_imgs.tt_corner;
            ImageFrame::new(
                [edge.cw180, edge.none, edge.cw270, edge.cw90],
                [corner.none, corner.cw270, corner.cw90, corner.cw180],
                Color::Rgba(0.08, 0.07, 0.04, 1.0),
                5.0,
            )
        })
        .title_font_size(self.fonts.cyri.scale(15))
        .parent(ui.window)
        .desc_font_size(self.fonts.cyri.scale(12))
        .font_id(self.fonts.cyri.conrod_id)
        .desc_text_color(TEXT_COLOR);

        let item_tooltip = ItemTooltip::new(
            {
                // Edge images [t, b, r, l]
                // Corner images [tr, tl, br, bl]
                let edge = &self.rot_imgs.tt_side;
                let corner = &self.rot_imgs.tt_corner;
                ImageFrame::new(
                    [edge.cw180, edge.none, edge.cw270, edge.cw90],
                    [corner.none, corner.cw270, corner.cw90, corner.cw180],
                    Color::Rgba(0.08, 0.07, 0.04, 1.0),
                    5.0,
                )
            },
            self.client,
            self.info,
            self.imgs,
            self.item_imgs,
            self.pulse,
            self.msm,
            self.rbm,
            Some(self.inventory),
            self.localized_strings,
            self.item_i18n,
        )
        .title_font_size(self.fonts.cyri.scale(20))
        .parent(ui.window)
        .desc_font_size(self.fonts.cyri.scale(12))
        .font_id(self.fonts.cyri.conrod_id)
        .desc_text_color(TEXT_COLOR);

        let slot_content = |slot| {
            let (hotbar, inventory, ..) = content_source;
            hotbar.get(slot).and_then(|content| match content {
                hotbar::SlotContents::Inventory(i, _) => inventory.get_by_hash(i),
                _ => None,
            })
        };

        // Helper
        let tooltip_text = |slot| {
            let (hotbar, inventory, _, skill_set, active_abilities, _, combo, _, stance, _, buffs) =
                content_source;
            hotbar.get(slot).and_then(|content| match content {
                hotbar::SlotContents::Inventory(i, _) => inventory.get_by_hash(i).map(|item| {
                    let (title, desc) =
                        util::item_text(item, self.localized_strings, self.item_i18n);

                    (title.into(), desc.into())
                }),
                hotbar::SlotContents::PrimaryAbility => active_abilities
                    .and_then(|a| {
                        Ability::from(a.primary).ability_id(
                            self.char_state,
                            Some(inventory),
                            Some(skill_set),
                            stance,
                            combo,
                            buffs,
                        )
                    })
                    .map(|id| util::ability_description(id, self.localized_strings)),
                hotbar::SlotContents::SecondaryAbility => active_abilities
                    .and_then(|a| {
                        Ability::from(a.secondary).ability_id(
                            self.char_state,
                            Some(inventory),
                            Some(skill_set),
                            stance,
                            combo,
                            buffs,
                        )
                    })
                    .map(|id| util::ability_description(id, self.localized_strings)),
                hotbar::SlotContents::Ability(i) => active_abilities
                    .and_then(|a| {
                        a.auxiliary_set(Some(inventory), Some(skill_set))
                            .get(i)
                            .and_then(|a| {
                                Ability::from(*a).ability_id(
                                    self.char_state,
                                    Some(inventory),
                                    Some(skill_set),
                                    stance,
                                    combo,
                                    buffs,
                                )
                            })
                    })
                    .map(|id| util::ability_description(id, self.localized_strings)),
            })
        };

        slot_maker.empty_slot = self.imgs.skillbar_slot;
        slot_maker.selected_slot = self.imgs.skillbar_slot;

        let slots = slot_entries(state, slot_offset);
        for entry in slots {
            let slot = slot_maker
                .fabricate(entry.slot, [40.0; 2], false, false)
                .filled_slot(self.imgs.skillbar_slot)
                .position(entry.position);
            // if there is an item attached, show item tooltip
            if let Some(item) = slot_content(entry.slot) {
                slot.with_item_tooltip(
                    self.item_tooltip_manager,
                    core::iter::once(item as &dyn ItemDesc),
                    &None,
                    &item_tooltip,
                )
                .set(entry.widget_id, ui);
            // if we can gather some text to display, show it
            } else if let Some((title, desc)) = tooltip_text(entry.slot) {
                slot.with_tooltip(self.tooltip_manager, &title, &desc, &tooltip, TEXT_COLOR)
                    .set(entry.widget_id, ui);
            // if not, just set slot
            } else {
                slot.set(entry.widget_id, ui);
            }

            if ui.widget_input(entry.widget_id).clicks().left().next().is_some() {
                events.push(Event::UseHotbarSlot(entry.slot));
            }

            // selection box around current hotbar index
            match self.global_state.window.last_input() {
                LastInput::Controller => {
                    // enable UI if gamepad binding is set for CurrentSlot
                    if self
                        .global_state
                        .settings
                        .controller
                        .get_game_button_binding(GameInput::CurrentSlot)
                        .is_some()
                        || self
                            .global_state
                            .settings
                            .controller
                            .get_layer_button_binding(GameInput::CurrentSlot)
                            .is_some()
                    {
                        let current_hotbar_selection =
                            self.hotbar.currently_selected_slot == entry.slot;
                        if current_hotbar_selection {
                            let selection_image = self.imgs.skillbar_index;

                            Image::new(selection_image)
                                .w_h(42.0, 42.0)
                                .middle_of(entry.widget_id)
                                .graphics_for(entry.widget_id)
                                .set(state.ids.slot_highlight, ui);
                        }
                    }
                },
                LastInput::Keyboard | LastInput::Mouse => {
                    // enable UI if keyboard binding is set for CurrentSlot
                    if self
                        .global_state
                        .settings
                        .controls
                        .get_binding(GameInput::CurrentSlot)
                        .is_some()
                    {
                        let current_hotbar_selection =
                            self.hotbar.currently_selected_slot == entry.slot;
                        if current_hotbar_selection {
                            let selection_image = self.imgs.skillbar_index;

                            Image::new(selection_image)
                                .w_h(42.0, 42.0)
                                .middle_of(entry.widget_id)
                                .graphics_for(entry.widget_id)
                                .set(state.ids.slot_highlight, ui);
                        }
                    }
                },
            }

            // shortcuts
            if let ShortcutNumbers::On = shortcuts {
                let key_desc = entry
                    .game_input
                    .and_then(|input| self.global_state.settings.controls.get_binding(input))
                    .map(|key| key.display_shortest())
                    .or_else(|| entry.custom_shortcut.map(|s| s.to_string()));

                if let Some(key_desc) = key_desc {
                    let position = entry.shortcut_position;
                    let position_bg = entry.shortcut_position_bg;
                    let (id, id_bg) = entry.shortcut_widget_ids;

                    // shortcut text
                    Text::new(&key_desc)
                        .position(position)
                        .font_size(self.fonts.cyri.scale(8))
                        .font_id(self.fonts.cyri.conrod_id)
                        .color(TEXT_COLOR)
                        .set(id, ui);
                    // shortcut background
                    Text::new(&key_desc)
                        .position(position_bg)
                        .font_size(self.fonts.cyri.scale(8))
                        .font_id(self.fonts.cyri.conrod_id)
                        .color(BLACK)
                        .set(id_bg, ui);
                }
            }
        }
        // M1 is primary slot on mouse, M2 is primary slot on controller
        let (primary_id, primary_bg, secondary_id, secondary_bg) =
            match self.global_state.window.last_input() {
                LastInput::Keyboard | LastInput::Mouse => (
                    state.ids.m1_content,
                    state.ids.m1_slot_bg,
                    state.ids.m2_content,
                    state.ids.m2_slot_bg,
                ),
                LastInput::Controller => (
                    state.ids.m2_content,
                    state.ids.m2_slot_bg,
                    state.ids.m1_content,
                    state.ids.m1_slot_bg,
                ),
            };

        // Slot M1
        Image::new(self.imgs.skillbar_slot)
            .w_h(40.0, 40.0)
            .right_from(state.ids.slot5, slot_offset)
            .set(state.ids.m1_slot_bg, ui);

        let primary_ability_id = self.active_abilities.and_then(|a| {
            Ability::from(a.primary).ability_id(
                self.char_state,
                Some(self.inventory),
                Some(self.skillset),
                self.stance,
                self.combo,
                self.buffs,
            )
        });

        let (primary_ability_title, primary_ability_desc) =
            util::ability_description(primary_ability_id.unwrap_or(""), self.localized_strings);

        let m1_clicked = Button::image(
            primary_ability_id.map_or(self.imgs.nothing, |id| util::ability_image(self.imgs, id)),
        )
        .hover_image(self.imgs.skillbar_index)
        .w_h(36.0, 36.0)
        .middle_of(primary_bg)
        .with_tooltip(
            self.tooltip_manager,
            &primary_ability_title,
            &primary_ability_desc,
            &tooltip,
            TEXT_COLOR,
        )
        .set(primary_id, ui)
        .was_clicked()
        || ui.widget_input(primary_id).clicks().left().next().is_some();
        if m1_clicked {
            events.push(Event::UsePrimaryAbility);
        }

        // Slot M2
        Image::new(self.imgs.skillbar_slot)
            .w_h(40.0, 40.0)
            .right_from(state.ids.m1_slot_bg, slot_offset)
            .set(state.ids.m2_slot_bg, ui);

        let secondary_ability_id = self.active_abilities.and_then(|a| {
            Ability::from(a.secondary).ability_id(
                self.char_state,
                Some(self.inventory),
                Some(self.skillset),
                self.stance,
                self.combo,
                self.buffs,
            )
        });

        let (secondary_ability_title, secondary_ability_desc) =
            util::ability_description(secondary_ability_id.unwrap_or(""), self.localized_strings);

        let m2_clicked = Button::image(
            secondary_ability_id.map_or(self.imgs.nothing, |id| util::ability_image(self.imgs, id)),
        )
        .hover_image(self.imgs.skillbar_index)
        .w_h(36.0, 36.0)
        .middle_of(secondary_bg)
        .image_color(
            if self
                .active_abilities
                .and_then(|a| {
                    a.activate_ability(
                        AbilityInput::Secondary,
                        Some(self.inventory),
                        self.skillset,
                        Some(self.body),
                        self.char_state,
                        self.stance,
                        self.combo,
                        self.stats,
                        self.buffs,
                    )
                })
                .is_some_and(|(a, _, _)| {
                    self.energy.current() >= a.energy_cost()
                        && self.combo.is_some_and(|c| c.counter() >= a.combo_cost())
                        && a.ability_meta()
                            .requirements
                            .requirements_met(self.stance, Some(self.inventory))
                })
            {
                Color::Rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::Rgba(0.3, 0.3, 0.3, 0.8)
            },
        )
        .with_tooltip(
            self.tooltip_manager,
            &secondary_ability_title,
            &secondary_ability_desc,
            &tooltip,
            TEXT_COLOR,
        )
        .set(secondary_id, ui)
        .was_clicked()
        || ui.widget_input(secondary_id).clicks().left().next().is_some();
        if m2_clicked {
            events.push(Event::UseSecondaryAbility);
        }

        // M1 and M2 icons
        match self.global_state.window.last_input() {
            LastInput::Keyboard | LastInput::Mouse => {
                Image::new(self.imgs.m1_ico)
                    .w_h(16.0, 18.0)
                    .mid_bottom_with_margin_on(state.ids.m1_content, -11.0)
                    .set(state.ids.m1_ico, ui);
                Image::new(self.imgs.m2_ico)
                    .w_h(16.0, 18.0)
                    .mid_bottom_with_margin_on(state.ids.m2_content, -11.0)
                    .set(state.ids.m2_ico, ui);
            },
            LastInput::Controller => {
                Image::new(icon_utils::fetch_skillbar_gamepad_left(
                    self.global_state.window.controller_type(),
                    self.imgs,
                ))
                .w_h(18.0, 18.0)
                .mid_bottom_with_margin_on(state.ids.m1_content, -11.0)
                .set(state.ids.m1_ico, ui);
                Image::new(icon_utils::fetch_skillbar_gamepad_right(
                    self.global_state.window.controller_type(),
                    self.imgs,
                ))
                .w_h(18.0, 18.0)
                .mid_bottom_with_margin_on(state.ids.m2_content, -11.0)
                .set(state.ids.m2_ico, ui);
            },
        }
    }

    fn show_combo_counter(&self, combo_floater: ComboFloater, state: &State, ui: &mut UiCell) {
        if combo_floater.combo > 0 {
            let combo_txt = format!("{} Combo", combo_floater.combo);
            let combo_cnt = combo_floater.combo as f32;
            let time_since_last_update = comp::combo::COMBO_DECAY_START - combo_floater.timer;
            let alpha = (1.0 - time_since_last_update * 0.2).min(1.0) as f32;
            let fnt_col = Color::Rgba(
                // White -> Yellow -> Red text color gradient depending on count
                (1.0 - combo_cnt / (combo_cnt + 20.0)).max(0.79),
                (1.0 - combo_cnt / (combo_cnt + 80.0)).max(0.19),
                (1.0 - combo_cnt / (combo_cnt + 5.0)).max(0.17),
                alpha,
            );
            // Increase size for higher counts,
            // "flash" on update by increasing the font size by 2.
            let fnt_size = ((14.0 + combo_floater.timer as f32 * 0.8).min(30.0)) as u32
                + if (time_since_last_update) < 0.1 { 2 } else { 0 };

            Rectangle::fill_with([10.0, 10.0], color::TRANSPARENT)
                .middle_of(ui.window)
                .set(state.ids.combo_align, ui);

            Text::new(combo_txt.as_str())
                .mid_bottom_with_margin_on(
                    state.ids.combo_align,
                    -350.0 + time_since_last_update * -8.0,
                )
                .font_size(self.fonts.cyri.scale(fnt_size))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, alpha))
                .set(state.ids.combo_bg, ui);
            Text::new(combo_txt.as_str())
                .bottom_right_with_margins_on(state.ids.combo_bg, 1.0, 1.0)
                .font_size(self.fonts.cyri.scale(fnt_size))
                .font_id(self.fonts.cyri.conrod_id)
                .color(fnt_col)
                .set(state.ids.combo, ui);
        }
    }

    fn show_pet_bar(&mut self, state: &State, ui: &mut UiCell, events: &mut Vec<Event>) {
        let client_entity = self.client.entity();
        let ecs = self.client.state().ecs();
        let alignments = ecs.read_storage::<comp::Alignment>();
        let entities = ecs.entities();
        let client_uid = match self.client.uid() {
            Some(u) => u,
            None => return,
        };

        let pet_entity = (&entities, &alignments).join().find_map(|(e, align)| {
            if e != client_entity {
                if let comp::Alignment::Owned(owner) = align {
                    if *owner == client_uid {
                        return Some(e);
                    }
                }
            }
            None
        });

        let has_pet = pet_entity.is_some();
        let activities = ecs.read_storage::<comp::CharacterActivity>();
        let activity = pet_entity.and_then(|p| activities.get(p));
        let is_staying = activity.map_or(false, |a| a.is_pet_staying);
        let pet_mode = activity.map_or(comp::PetMode::Defensive, |a| a.pet_mode);

        let tooltip = Tooltip::new({
            let edge = &self.rot_imgs.tt_side;
            let corner = &self.rot_imgs.tt_corner;
            ImageFrame::new(
                [edge.cw180, edge.none, edge.cw270, edge.cw90],
                [corner.none, corner.cw270, corner.cw90, corner.cw180],
                Color::Rgba(0.08, 0.07, 0.04, 1.0),
                5.0,
            )
        })
        .title_font_size(self.fonts.cyri.scale(13))
        .parent(ui.window)
        .desc_font_size(self.fonts.cyri.scale(11))
        .font_id(self.fonts.cyri.conrod_id)
        .desc_text_color(TEXT_COLOR);

        let btn_size = 28.0;
        let icon_size = 18.0;
        let border_size = 30.0;

        // 0. Summon / Call Pet Button (Ctrl+0)
        let (summon_title, summon_desc) = (
            "Invocar Mascota (Ctrl+0)",
            "Llama a tu mascota a tu lado, curándola y reviviéndola si ha caído.",
        );
        let clicked_summon = Button::image(self.imgs.skillbar_slot)
            .hover_image(self.imgs.skillbar_index)
            .press_image(self.imgs.skillbar_slot)
            .w_h(btn_size, btn_size)
            .up_from(state.ids.slot11, 5.0)
            .with_tooltip(self.tooltip_manager, summon_title, summon_desc, &tooltip, TEXT_COLOR)
            .set(state.ids.pet_btn_summon, ui)
            .was_clicked()
            || ui.widget_input(state.ids.pet_btn_summon).clicks().left().next().is_some();
        if clicked_summon {
            events.push(Event::CommandPet(comp::PetCommand::Summon));
        }
        Image::new(self.imgs.tamer_class)
            .w_h(icon_size, icon_size)
            .color(Some(Color::Rgba(0.35, 1.0, 0.45, 1.0)))
            .middle_of(state.ids.pet_btn_summon)
            .graphics_for(state.ids.pet_btn_summon)
            .set(state.ids.pet_btn_summon_icon, ui);
        Text::new("^0")
            .top_left_with_margins_on(state.ids.pet_btn_summon, 1.0, 2.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_summon)
            .color(BLACK)
            .set(state.ids.pet_btn_summon_sc_bg, ui);
        Text::new("^0")
            .bottom_left_with_margins_on(state.ids.pet_btn_summon_sc_bg, 1.0, 1.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_summon)
            .color(QUALITY_LEGENDARY)
            .set(state.ids.pet_btn_summon_sc, ui);

        // Header Label: [ CONTROL DE MASCOTA ]
        let header_txt = if has_pet { "[ CONTROL DE MASCOTA ]" } else { "[ MASCOTA (INACTIVA) ]" };
        let header_col = if has_pet {
            Color::Rgba(1.0, 0.85, 0.35, 1.0)
        } else {
            Color::Rgba(0.65, 0.65, 0.65, 0.75)
        };
        Text::new(header_txt)
            .up_from(state.ids.pet_btn_summon, 4.0)
            .font_size(self.fonts.cyri.scale(8))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_summon)
            .color(BLACK)
            .set(state.ids.pet_info_name_bg, ui);
        Text::new(header_txt)
            .bottom_left_with_margins_on(state.ids.pet_info_name_bg, 1.0, 1.0)
            .font_size(self.fonts.cyri.scale(8))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_summon)
            .color(header_col)
            .set(state.ids.pet_info_name, ui);

        // 1. Attack Button (Ctrl+1)
        let (atk_title, atk_desc) = (
            "Atacar (Ctrl+1)",
            if has_pet {
                "Ordena a tu mascota atacar a tu objetivo actual o al enemigo más cercano."
            } else {
                "Sin mascota activa. Invoca o domestica una criatura para usar los controles."
            },
        );
        let clicked_attack = Button::image(self.imgs.skillbar_slot)
            .hover_image(self.imgs.skillbar_index)
            .press_image(self.imgs.skillbar_slot)
            .w_h(btn_size, btn_size)
            .right_from(state.ids.pet_btn_summon, 3.0)
            .with_tooltip(self.tooltip_manager, atk_title, atk_desc, &tooltip, TEXT_COLOR)
            .set(state.ids.pet_btn_attack, ui)
            .was_clicked()
            || ui.widget_input(state.ids.pet_btn_attack).clicks().left().next().is_some();
        if clicked_attack {
            let target_uid = self.info.target_entity.and_then(|e| {
                self.client.state().read_component_copied::<common::uid::Uid>(e)
            });
            events.push(Event::CommandPet(comp::PetCommand::Attack(target_uid)));
        }
        Image::new(self.imgs.swords_crossed)
            .w_h(icon_size, icon_size)
            .color(Some(if has_pet {
                Color::Rgba(1.0, 0.85, 0.35, 1.0)
            } else {
                Color::Rgba(0.6, 0.6, 0.6, 0.45)
            }))
            .middle_of(state.ids.pet_btn_attack)
            .graphics_for(state.ids.pet_btn_attack)
            .set(state.ids.pet_btn_attack_icon, ui);
        Text::new("^1")
            .top_left_with_margins_on(state.ids.pet_btn_attack, 1.0, 2.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_attack)
            .color(BLACK)
            .set(state.ids.pet_btn_attack_sc_bg, ui);
        Text::new("^1")
            .bottom_left_with_margins_on(state.ids.pet_btn_attack_sc_bg, 1.0, 1.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_attack)
            .color(QUALITY_LEGENDARY)
            .set(state.ids.pet_btn_attack_sc, ui);

        // 2. Follow Button (Ctrl+2)
        let (follow_title, follow_desc) = (
            "Sígueme (Ctrl+2)",
            if has_pet {
                "Ordena a tu mascota dejar de combatir y seguirte de inmediato."
            } else {
                "Sin mascota activa. Invoca o domestica una criatura para usar los controles."
            },
        );
        let clicked_follow = Button::image(self.imgs.skillbar_slot)
            .hover_image(self.imgs.skillbar_index)
            .press_image(self.imgs.skillbar_slot)
            .w_h(btn_size, btn_size)
            .right_from(state.ids.pet_btn_attack, 3.0)
            .with_tooltip(self.tooltip_manager, follow_title, follow_desc, &tooltip, TEXT_COLOR)
            .set(state.ids.pet_btn_follow, ui)
            .was_clicked()
            || ui.widget_input(state.ids.pet_btn_follow).clicks().left().next().is_some();
        if clicked_follow {
            events.push(Event::CommandPet(comp::PetCommand::Follow));
        }
        Image::new(self.imgs.utility_speed_skill)
            .w_h(icon_size, icon_size)
            .color(Some(if has_pet {
                Color::Rgba(0.35, 0.95, 0.35, 1.0)
            } else {
                Color::Rgba(0.6, 0.6, 0.6, 0.45)
            }))
            .middle_of(state.ids.pet_btn_follow)
            .graphics_for(state.ids.pet_btn_follow)
            .set(state.ids.pet_btn_follow_icon, ui);
        Text::new("^2")
            .top_left_with_margins_on(state.ids.pet_btn_follow, 1.0, 2.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_follow)
            .color(BLACK)
            .set(state.ids.pet_btn_follow_sc_bg, ui);
        Text::new("^2")
            .bottom_left_with_margins_on(state.ids.pet_btn_follow_sc_bg, 1.0, 1.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_follow)
            .color(QUALITY_LEGENDARY)
            .set(state.ids.pet_btn_follow_sc, ui);

        // 3. Stay Button (Ctrl+3)
        let (stay_title, stay_desc) = (
            "Quedarse (Ctrl+3)",
            if has_pet {
                "Ordena a tu mascota mantenerse quieta en su lugar actual."
            } else {
                "Sin mascota activa. Invoca o domestica una criatura para usar los controles."
            },
        );
        let clicked_stay = Button::image(self.imgs.skillbar_slot)
            .hover_image(self.imgs.skillbar_index)
            .press_image(self.imgs.skillbar_slot)
            .w_h(btn_size, btn_size)
            .right_from(state.ids.pet_btn_follow, 3.0)
            .with_tooltip(self.tooltip_manager, stay_title, stay_desc, &tooltip, TEXT_COLOR)
            .set(state.ids.pet_btn_stay, ui)
            .was_clicked()
            || ui.widget_input(state.ids.pet_btn_stay).clicks().left().next().is_some();
        if clicked_stay {
            events.push(Event::CommandPet(comp::PetCommand::Stay));
        }
        if has_pet && is_staying {
            Image::new(self.imgs.inv_slot_sel)
                .w_h(border_size, border_size)
                .middle_of(state.ids.pet_btn_stay)
                .graphics_for(state.ids.pet_btn_stay)
                .set(state.ids.pet_btn_stay_border, ui);
        }
        Image::new(self.imgs.lock)
            .w_h(15.0, 15.0)
            .color(Some(if is_staying {
                Color::Rgba(1.0, 0.85, 0.2, 1.0)
            } else if has_pet {
                Color::Rgba(0.9, 0.85, 0.7, 0.9)
            } else {
                Color::Rgba(0.6, 0.6, 0.6, 0.45)
            }))
            .middle_of(state.ids.pet_btn_stay)
            .graphics_for(state.ids.pet_btn_stay)
            .set(state.ids.pet_btn_stay_icon, ui);
        Text::new("^3")
            .top_left_with_margins_on(state.ids.pet_btn_stay, 1.0, 2.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_stay)
            .color(BLACK)
            .set(state.ids.pet_btn_stay_sc_bg, ui);
        Text::new("^3")
            .bottom_left_with_margins_on(state.ids.pet_btn_stay_sc_bg, 1.0, 1.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_stay)
            .color(QUALITY_LEGENDARY)
            .set(state.ids.pet_btn_stay_sc, ui);

        // 4. Heal / Life Transfer Button (Ctrl+7)
        let (heal_title, heal_desc) = (
            "Transfusión Vital (Ctrl+7)",
            if has_pet {
                "Transfiere un 20 % de tu salud a tu mascota para sanarla intensamente (cura 2.5x el sacrificio). Requiere más de 15 de vida."
            } else {
                "Sin mascota activa. Invoca o domestica una criatura para usar los controles."
            },
        );
        let clicked_heal = Button::image(self.imgs.skillbar_slot)
            .hover_image(self.imgs.skillbar_index)
            .press_image(self.imgs.skillbar_slot)
            .w_h(btn_size, btn_size)
            .right_from(state.ids.pet_btn_stay, 3.0)
            .with_tooltip(self.tooltip_manager, heal_title, heal_desc, &tooltip, TEXT_COLOR)
            .set(state.ids.pet_btn_heal, ui)
            .was_clicked()
            || ui.widget_input(state.ids.pet_btn_heal).clicks().left().next().is_some();
        if clicked_heal {
            events.push(Event::CommandPet(comp::PetCommand::Heal));
        }
        Image::new(self.imgs.health_ico)
            .w_h(icon_size, icon_size)
            .color(Some(if has_pet {
                Color::Rgba(1.0, 0.28, 0.28, 1.0)
            } else {
                Color::Rgba(0.6, 0.6, 0.6, 0.45)
            }))
            .middle_of(state.ids.pet_btn_heal)
            .graphics_for(state.ids.pet_btn_heal)
            .set(state.ids.pet_btn_heal_icon, ui);
        Text::new("^7")
            .top_left_with_margins_on(state.ids.pet_btn_heal, 1.0, 2.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_heal)
            .color(BLACK)
            .set(state.ids.pet_btn_heal_sc_bg, ui);
        Text::new("^7")
            .bottom_left_with_margins_on(state.ids.pet_btn_heal_sc_bg, 1.0, 1.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_heal)
            .color(QUALITY_LEGENDARY)
            .set(state.ids.pet_btn_heal_sc, ui);

        // 5. Aggressive Mode Button (Ctrl+4)
        let (aggro_title, aggro_desc) = (
            "Modo Agresivo (Ctrl+4)",
            if has_pet {
                "Tu mascota atacará automáticamente a cualquier enemigo que se acerque a 25 metros."
            } else {
                "Sin mascota activa. Invoca o domestica una criatura para usar los controles."
            },
        );
        let clicked_aggro = Button::image(self.imgs.skillbar_slot)
            .hover_image(self.imgs.skillbar_index)
            .press_image(self.imgs.skillbar_slot)
            .w_h(btn_size, btn_size)
            .right_from(state.ids.pet_btn_heal, 6.0)
            .with_tooltip(self.tooltip_manager, aggro_title, aggro_desc, &tooltip, TEXT_COLOR)
            .set(state.ids.pet_btn_aggro, ui)
            .was_clicked()
            || ui.widget_input(state.ids.pet_btn_aggro).clicks().left().next().is_some();
        if clicked_aggro {
            events.push(Event::CommandPet(comp::PetCommand::SetMode(comp::PetMode::Aggressive)));
        }
        if has_pet && pet_mode == comp::PetMode::Aggressive {
            Image::new(self.imgs.inv_slot_sel)
                .w_h(border_size, border_size)
                .middle_of(state.ids.pet_btn_aggro)
                .graphics_for(state.ids.pet_btn_aggro)
                .set(state.ids.pet_btn_aggro_border, ui);
        }
        Image::new(self.imgs.combat_rating_ico)
            .w_h(icon_size, icon_size)
            .color(Some(if pet_mode == comp::PetMode::Aggressive {
                Color::Rgba(1.0, 0.35, 0.35, 1.0)
            } else if has_pet {
                Color::Rgba(0.85, 0.5, 0.5, 0.8)
            } else {
                Color::Rgba(0.6, 0.6, 0.6, 0.45)
            }))
            .middle_of(state.ids.pet_btn_aggro)
            .graphics_for(state.ids.pet_btn_aggro)
            .set(state.ids.pet_btn_aggro_icon, ui);
        Text::new("^4")
            .top_left_with_margins_on(state.ids.pet_btn_aggro, 1.0, 2.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_aggro)
            .color(BLACK)
            .set(state.ids.pet_btn_aggro_sc_bg, ui);
        Text::new("^4")
            .bottom_left_with_margins_on(state.ids.pet_btn_aggro_sc_bg, 1.0, 1.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_aggro)
            .color(QUALITY_LEGENDARY)
            .set(state.ids.pet_btn_aggro_sc, ui);

        // 5. Defensive Mode Button (Ctrl+5)
        let (def_title, def_desc) = (
            "Modo Defensivo (Ctrl+5)",
            if has_pet {
                "Tu mascota sólo atacará a los enemigos que te ataquen a ti o a los que ataques tú."
            } else {
                "Sin mascota activa. Invoca o domestica una criatura para usar los controles."
            },
        );
        let clicked_def = Button::image(self.imgs.skillbar_slot)
            .hover_image(self.imgs.skillbar_index)
            .press_image(self.imgs.skillbar_slot)
            .w_h(btn_size, btn_size)
            .right_from(state.ids.pet_btn_aggro, 3.0)
            .with_tooltip(self.tooltip_manager, def_title, def_desc, &tooltip, TEXT_COLOR)
            .set(state.ids.pet_btn_def, ui)
            .was_clicked()
            || ui.widget_input(state.ids.pet_btn_def).clicks().left().next().is_some();
        if clicked_def {
            events.push(Event::CommandPet(comp::PetCommand::SetMode(comp::PetMode::Defensive)));
        }
        if has_pet && pet_mode == comp::PetMode::Defensive {
            Image::new(self.imgs.inv_slot_sel)
                .w_h(border_size, border_size)
                .middle_of(state.ids.pet_btn_def)
                .graphics_for(state.ids.pet_btn_def)
                .set(state.ids.pet_btn_def_border, ui);
        }
        Image::new(self.imgs.protection_ico)
            .w_h(icon_size, icon_size)
            .color(Some(if pet_mode == comp::PetMode::Defensive {
                Color::Rgba(0.45, 0.75, 1.0, 1.0)
            } else if has_pet {
                Color::Rgba(0.6, 0.75, 0.9, 0.8)
            } else {
                Color::Rgba(0.6, 0.6, 0.6, 0.45)
            }))
            .middle_of(state.ids.pet_btn_def)
            .graphics_for(state.ids.pet_btn_def)
            .set(state.ids.pet_btn_def_icon, ui);
        Text::new("^5")
            .top_left_with_margins_on(state.ids.pet_btn_def, 1.0, 2.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_def)
            .color(BLACK)
            .set(state.ids.pet_btn_def_sc_bg, ui);
        Text::new("^5")
            .bottom_left_with_margins_on(state.ids.pet_btn_def_sc_bg, 1.0, 1.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_def)
            .color(QUALITY_LEGENDARY)
            .set(state.ids.pet_btn_def_sc, ui);

        // 6. Passive Mode Button (Ctrl+6)
        let (passive_title, passive_desc) = (
            "Modo Pasivo (Ctrl+6)",
            if has_pet {
                "Tu mascota no atacará bajo ninguna circunstancia, incluso si recibe daño."
            } else {
                "Sin mascota activa. Invoca o domestica una criatura para usar los controles."
            },
        );
        let clicked_passive = Button::image(self.imgs.skillbar_slot)
            .hover_image(self.imgs.skillbar_index)
            .press_image(self.imgs.skillbar_slot)
            .w_h(btn_size, btn_size)
            .right_from(state.ids.pet_btn_def, 3.0)
            .with_tooltip(self.tooltip_manager, passive_title, passive_desc, &tooltip, TEXT_COLOR)
            .set(state.ids.pet_btn_passive, ui)
            .was_clicked()
            || ui.widget_input(state.ids.pet_btn_passive).clicks().left().next().is_some();
        if clicked_passive {
            events.push(Event::CommandPet(comp::PetCommand::SetMode(comp::PetMode::Passive)));
        }
        if has_pet && pet_mode == comp::PetMode::Passive {
            Image::new(self.imgs.inv_slot_sel)
                .w_h(border_size, border_size)
                .middle_of(state.ids.pet_btn_passive)
                .graphics_for(state.ids.pet_btn_passive)
                .set(state.ids.pet_btn_passive_border, ui);
        }
        Image::new(self.imgs.skill_sceptre_heal)
            .w_h(icon_size, icon_size)
            .color(Some(if pet_mode == comp::PetMode::Passive {
                Color::Rgba(0.95, 0.95, 1.0, 1.0)
            } else if has_pet {
                Color::Rgba(0.7, 0.7, 0.75, 0.8)
            } else {
                Color::Rgba(0.6, 0.6, 0.6, 0.45)
            }))
            .middle_of(state.ids.pet_btn_passive)
            .graphics_for(state.ids.pet_btn_passive)
            .set(state.ids.pet_btn_passive_icon, ui);
        Text::new("^6")
            .top_left_with_margins_on(state.ids.pet_btn_passive, 1.0, 2.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_passive)
            .color(BLACK)
            .set(state.ids.pet_btn_passive_sc_bg, ui);
        Text::new("^6")
            .bottom_left_with_margins_on(state.ids.pet_btn_passive_sc_bg, 1.0, 1.0)
            .font_size(self.fonts.cyri.scale(7))
            .font_id(self.fonts.cyri.conrod_id)
            .graphics_for(state.ids.pet_btn_passive)
            .color(QUALITY_LEGENDARY)
            .set(state.ids.pet_btn_passive_sc, ui);
    }
}

pub struct State {
    ids: Ids,
}

impl Widget for Skillbar<'_> {
    type Event = Vec<Event>;
    type State = State;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {}

    fn update(mut self, args: widget::UpdateArgs<Self>) -> Self::Event {
        common_base::prof_span!("Skillbar::update");
        let widget::UpdateArgs { state, ui, .. } = args;

        let mut events = Vec::new();

        let slot_offset = 3.0;

        // Death message
        if self.health.is_dead {
            self.show_death_message(state, ui);
        }
        // Give up message
        else if comp::is_downed(Some(self.health), self.client.current().as_ref()) {
            self.show_give_up_message(state, ui);
        }
        // Riding / sitting on boat seat prompt
        else if self.client.is_riding() {
            self.show_riding_message(state, ui);
        }

        // Skillbar

        // Poise bar ticks
        state.update(|s| {
            s.ids.poise_ticks.resize(
                self::Poise::POISE_THRESHOLDS.len(),
                &mut ui.widget_id_generator(),
            )
        });

        // Alignment and BG
        let alignment_size = 40.0 * 12.0 + slot_offset * 11.0;
        Rectangle::fill_with([alignment_size, 160.0], color::TRANSPARENT)
            .mid_bottom_with_margin_on(ui.window, 10.0)
            .set(state.ids.frame, ui);

        // WoW Player Frame (Arriba a la izquierda)
        self.show_wow_player_frame(state, ui);

        // Health, Energy and Poise bars
        self.show_stat_bars(state, ui, &mut events);

        // Slots
        self.show_slotbar(state, ui, slot_offset, &mut events);

        // WoW Pet Action Bar (Encima de las habilidades)
        self.show_pet_bar(state, ui, &mut events);

        // Combo Counter
        if let Some(combo_floater) = self.combo_floater {
            self.show_combo_counter(combo_floater, state, ui);
        }
        events
    }
}
