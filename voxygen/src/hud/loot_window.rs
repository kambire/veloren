use super::{
    get_quality_col,
    img_ids::{Imgs, ImgsRot},
    item_imgs::{animate_by_pulse, ItemImgs},
    util, HudInfo, TEXT_COLOR,
};
use crate::ui::{
    fonts::Fonts, ImageFrame, ItemTooltip, ItemTooltipManager, ItemTooltipable,
};
use client::Client;
use common::{
    comp::{
        inventory::item::{ItemDesc, ItemI18n, MaterialStatManifest, Quality},
        Inventory, PickupItem,
    },
    recipe::RecipeBookManifest,
};
use conrod_core::{
    widget::{self, Button, Image, RoundedRectangle, Text},
    widget_ids, Borderable, Color, Colorable, Labelable, Positionable, Sizeable, Widget,
    WidgetCommon,
};
use i18n::Localization;
use specs::Entity as EcsEntity;

widget_ids! {
    pub struct Ids {
        window_bg,
        window_border,
        header_bg,
        header_title,
        header_count,
        close_btn,
        page_prev_btn,
        page_next_btn,
        page_indicator,

        item_rows[],
        item_row_bgs[],
        item_slot_bgs[],
        item_slot_frames[],
        item_icons[],
        item_amount_bgs[],
        item_amounts[],
        item_names[],
        item_categories[],
        item_loot_btns[],

        footer_bg,
        loot_all_btn,
        close_btn_bottom,
    }
}

pub enum LootWindowEvent {
    PickUp(EcsEntity),
    PickUpAll,
    Close,
}

pub struct State {
    ids: Ids,
    page: usize,
}

const ITEMS_PER_PAGE: usize = 5;
const WINDOW_WIDTH: f64 = 340.0;
const ROW_HEIGHT: f64 = 48.0;
const ROW_SPACING: f64 = 4.0;
const HEADER_HEIGHT: f64 = 38.0;
const FOOTER_HEIGHT: f64 = 44.0;

#[derive(WidgetCommon)]
pub struct LootWindow<'a> {
    items: &'a [(EcsEntity, &'a PickupItem)],
    client: &'a Client,
    info: &'a HudInfo<'a>,
    imgs: &'a Imgs,
    item_imgs: &'a ItemImgs,
    rot_imgs: &'a ImgsRot,
    fonts: &'a Fonts,
    localized_strings: &'a Localization,
    item_i18n: &'a ItemI18n,
    msm: &'a MaterialStatManifest,
    rbm: &'a RecipeBookManifest,
    inventory: Option<&'a Inventory>,
    item_tooltip_manager: &'a mut ItemTooltipManager,
    pulse: f32,

    #[conrod(common_builder)]
    common: widget::CommonBuilder,
}

impl<'a> LootWindow<'a> {
    pub fn new(
        items: &'a [(EcsEntity, &'a PickupItem)],
        client: &'a Client,
        info: &'a HudInfo<'a>,
        imgs: &'a Imgs,
        item_imgs: &'a ItemImgs,
        rot_imgs: &'a ImgsRot,
        fonts: &'a Fonts,
        localized_strings: &'a Localization,
        item_i18n: &'a ItemI18n,
        msm: &'a MaterialStatManifest,
        rbm: &'a RecipeBookManifest,
        inventory: Option<&'a Inventory>,
        item_tooltip_manager: &'a mut ItemTooltipManager,
        pulse: f32,
    ) -> Self {
        Self {
            items,
            client,
            info,
            imgs,
            item_imgs,
            rot_imgs,
            fonts,
            localized_strings,
            item_i18n,
            msm,
            rbm,
            inventory,
            item_tooltip_manager,
            pulse,
            common: widget::CommonBuilder::default(),
        }
    }
}

impl Widget for LootWindow<'_> {
    type Event = Option<LootWindowEvent>;
    type State = State;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            page: 0,
        }
    }

    fn style(&self) -> Self::Style {}

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        common_base::prof_span!("LootWindow::update");
        let widget::UpdateArgs { state, ui, .. } = args;

        let total_items = self.items.len();
        if total_items == 0 {
            return Some(LootWindowEvent::Close);
        }

        let total_pages = (total_items + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
        let current_page = state.page.min(total_pages.saturating_sub(1));
        if current_page != state.page {
            state.update(|s| s.page = current_page);
        }

        let start_idx = current_page * ITEMS_PER_PAGE;
        let end_idx = (start_idx + ITEMS_PER_PAGE).min(total_items);
        let visible_items = &self.items[start_idx..end_idx];
        let visible_count = visible_items.len();

        // Resize per-item widget vectors
        if state.ids.item_rows.len() < visible_count {
            state.update(|s| {
                s.ids.item_rows.resize(visible_count, &mut ui.widget_id_generator());
                s.ids.item_row_bgs.resize(visible_count, &mut ui.widget_id_generator());
                s.ids.item_slot_bgs.resize(visible_count, &mut ui.widget_id_generator());
                s.ids.item_slot_frames.resize(visible_count, &mut ui.widget_id_generator());
                s.ids.item_icons.resize(visible_count, &mut ui.widget_id_generator());
                s.ids.item_amounts.resize(visible_count, &mut ui.widget_id_generator());
                s.ids.item_amount_bgs.resize(visible_count, &mut ui.widget_id_generator());
                s.ids.item_names.resize(visible_count, &mut ui.widget_id_generator());
                s.ids.item_categories.resize(visible_count, &mut ui.widget_id_generator());
                s.ids.item_loot_btns.resize(visible_count, &mut ui.widget_id_generator());
            });
        }

        // Window geometry
        let list_height = (visible_count as f64) * (ROW_HEIGHT + ROW_SPACING);
        let window_height = HEADER_HEIGHT + list_height + FOOTER_HEIGHT + 12.0;

        // Position window on left-center of screen (classic WoW Loot Frame position)
        let window_x = -260.0;
        let window_y = 30.0;

        // Base background
        RoundedRectangle::fill_with([WINDOW_WIDTH, window_height], 6.0, Color::Rgba(0.07, 0.08, 0.11, 0.95))
            .middle_of(ui.window)
            .x_y(window_x, window_y)
            .set(state.ids.window_bg, ui);

        // Gold border
        RoundedRectangle::outline_styled(
            [WINDOW_WIDTH, window_height],
            6.0,
            widget::line::Style::solid()
                .color(Color::Rgba(0.85, 0.72, 0.32, 1.0))
                .thickness(2.0),
        )
        .middle_of(state.ids.window_bg)
        .set(state.ids.window_border, ui);

        // Header bar
        RoundedRectangle::fill_with([WINDOW_WIDTH - 8.0, HEADER_HEIGHT - 6.0], 4.0, Color::Rgba(0.13, 0.15, 0.20, 0.98))
            .mid_top_with_margin_on(state.ids.window_bg, 4.0)
            .set(state.ids.header_bg, ui);

        // Header title
        Text::new("⚔️ BOTÍN")
            .top_left_with_margins_on(state.ids.header_bg, 7.0, 12.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(16))
            .color(Color::Rgba(1.0, 0.85, 0.30, 1.0))
            .set(state.ids.header_title, ui);

        // Item count badge
        let count_str = format!("({} objetos)", total_items);
        Text::new(&count_str)
            .right_from(state.ids.header_title, 8.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(12))
            .color(Color::Rgba(0.70, 0.70, 0.75, 1.0))
            .set(state.ids.header_count, ui);

        // Close button 'X' in header
        if Button::image(self.imgs.close_btn)
            .hover_image(self.imgs.close_btn_hover)
            .press_image(self.imgs.close_btn_press)
            .w_h(20.0, 20.0)
            .top_right_with_margins_on(state.ids.header_bg, 6.0, 6.0)
            .set(state.ids.close_btn, ui)
            .was_clicked()
        {
            return Some(LootWindowEvent::Close);
        }

        // Pagination buttons if total_pages > 1
        if total_pages > 1 {
            let page_text = format!("{}/{}", current_page + 1, total_pages);
            Text::new(&page_text)
                .mid_right_with_margin_on(state.ids.header_bg, 64.0)
                .font_id(self.fonts.cyri.conrod_id)
                .font_size(self.fonts.cyri.scale(11))
                .color(Color::Rgba(0.80, 0.80, 0.85, 1.0))
                .set(state.ids.page_indicator, ui);

            if Button::new()
                .w_h(18.0, 18.0)
                .left_from(state.ids.page_indicator, 4.0)
                .label("<")
                .label_font_id(self.fonts.cyri.conrod_id)
                .label_font_size(self.fonts.cyri.scale(11))
                .label_color(Color::Rgba(0.9, 0.8, 0.3, 1.0))
                .color(Color::Rgba(0.2, 0.22, 0.28, 0.9))
                .hover_color(Color::Rgba(0.3, 0.33, 0.42, 1.0))
                .set(state.ids.page_prev_btn, ui)
                .was_clicked()
            {
                state.update(|s| s.page = s.page.saturating_sub(1));
            }

            if Button::new()
                .w_h(18.0, 18.0)
                .right_from(state.ids.page_indicator, 4.0)
                .label(">")
                .label_font_id(self.fonts.cyri.conrod_id)
                .label_font_size(self.fonts.cyri.scale(11))
                .label_color(Color::Rgba(0.9, 0.8, 0.3, 1.0))
                .color(Color::Rgba(0.2, 0.22, 0.28, 0.9))
                .hover_color(Color::Rgba(0.3, 0.33, 0.42, 1.0))
                .set(state.ids.page_next_btn, ui)
                .was_clicked()
            {
                state.update(|s| {
                    if s.page + 1 < total_pages {
                        s.page += 1;
                    }
                });
            }
        }

        // Shared item tooltip helper
        let item_tooltip = ItemTooltip::new(
            {
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
            self.inventory,
            self.localized_strings,
            self.item_i18n,
        )
        .title_font_size(self.fonts.cyri.scale(16))
        .parent(ui.window)
        .desc_font_size(self.fonts.cyri.scale(12))
        .font_id(self.fonts.cyri.conrod_id)
        .desc_text_color(TEXT_COLOR);

        // Render each visible loot item
        let mut clicked_pickup: Option<EcsEntity> = None;

        for (i, &(entity, pickup_item)) in visible_items.iter().enumerate() {
            let item = pickup_item.item();
            let quality = item.quality();
            let quality_col = get_quality_col(quality);

            let quality_slot_img = match quality {
                Quality::Low => self.imgs.inv_slot_grey,
                Quality::Common => self.imgs.inv_slot_common,
                Quality::Moderate => self.imgs.inv_slot_green,
                Quality::High => self.imgs.inv_slot_blue,
                Quality::Epic => self.imgs.inv_slot_purple,
                Quality::Legendary => self.imgs.inv_slot_gold,
                Quality::Artifact => self.imgs.inv_slot_orange,
                _ => self.imgs.inv_slot_red,
            };

            let quality_name = match quality {
                Quality::Low => "Pobre",
                Quality::Common => "Común",
                Quality::Moderate => "Poco común",
                Quality::High => "Raro",
                Quality::Epic => "Épico",
                Quality::Legendary => "Legendario",
                Quality::Artifact => "Artefacto",
                _ => "Especial",
            };

            let row_y_offset = HEADER_HEIGHT + 4.0 + (i as f64) * (ROW_HEIGHT + ROW_SPACING);

            // Row background container
            RoundedRectangle::fill_with([WINDOW_WIDTH - 12.0, ROW_HEIGHT], 4.0, Color::Rgba(0.11, 0.12, 0.16, 0.85))
                .mid_top_with_margin_on(state.ids.window_bg, row_y_offset)
                .set(state.ids.item_row_bgs[i], ui);

            // Slot icon background & frame on left (38x38 px)
            RoundedRectangle::fill_with([38.0, 38.0], 3.0, Color::Rgba(0.05, 0.06, 0.08, 0.95))
                .mid_left_with_margin_on(state.ids.item_row_bgs[i], 5.0)
                .set(state.ids.item_slot_bgs[i], ui);

            Image::new(quality_slot_img)
                .w_h(38.0, 38.0)
                .middle_of(state.ids.item_slot_bgs[i])
                .set(state.ids.item_slot_frames[i], ui);

            // Item icon with tooltip
            Image::new(animate_by_pulse(
                &self.item_imgs.img_ids_or_not_found_img(item.into()),
                self.pulse,
            ))
            .w_h(32.0, 32.0)
            .middle_of(state.ids.item_slot_bgs[i])
            .with_item_tooltip(
                self.item_tooltip_manager,
                core::iter::once(item as &dyn ItemDesc),
                &None,
                &item_tooltip,
            )
            .set(state.ids.item_icons[i], ui);

            // Amount badge if > 1
            let amount = item.amount();
            if amount > 1 {
                let amount_str = format!("{}", amount);
                Text::new(&amount_str)
                    .bottom_right_with_margins_on(state.ids.item_slot_bgs[i], 0.0, 2.0)
                    .font_id(self.fonts.cyri.conrod_id)
                    .font_size(self.fonts.cyri.scale(10))
                    .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                    .set(state.ids.item_amount_bgs[i], ui);

                Text::new(&amount_str)
                    .bottom_right_with_margins_on(state.ids.item_slot_bgs[i], 1.0, 3.0)
                    .font_id(self.fonts.cyri.conrod_id)
                    .font_size(self.fonts.cyri.scale(10))
                    .color(Color::Rgba(1.0, 1.0, 1.0, 1.0))
                    .set(state.ids.item_amounts[i], ui);
            }

            // Item Name
            let (raw_name, _) = util::item_text(&item, self.localized_strings, self.item_i18n);
            let display_name = if raw_name.chars().count() > 22 {
                let mut truncated: String = raw_name.chars().take(20).collect();
                truncated.push_str("...");
                truncated
            } else {
                raw_name.to_string()
            };

            Text::new(&display_name)
                .top_left_with_margins_on(state.ids.item_row_bgs[i], 8.0, 48.0)
                .font_id(self.fonts.cyri.conrod_id)
                .font_size(self.fonts.cyri.scale(13))
                .color(quality_col)
                .with_item_tooltip(
                    self.item_tooltip_manager,
                    core::iter::once(item as &dyn ItemDesc),
                    &None,
                    &item_tooltip,
                )
                .set(state.ids.item_names[i], ui);

            // Quality / Type Subtitle
            Text::new(quality_name)
                .bottom_left_with_margins_on(state.ids.item_row_bgs[i], 8.0, 48.0)
                .font_id(self.fonts.cyri.conrod_id)
                .font_size(self.fonts.cyri.scale(11))
                .color(Color::Rgba(0.60, 0.62, 0.68, 1.0))
                .set(state.ids.item_categories[i], ui);

            // "Recoger" button on the right
            if Button::new()
                .w_h(64.0, 26.0)
                .mid_right_with_margin_on(state.ids.item_row_bgs[i], 8.0)
                .label("Recoger")
                .label_font_id(self.fonts.cyri.conrod_id)
                .label_font_size(self.fonts.cyri.scale(11))
                .label_color(Color::Rgba(1.0, 0.88, 0.40, 1.0))
                .color(Color::Rgba(0.18, 0.20, 0.26, 0.95))
                .hover_color(Color::Rgba(0.30, 0.34, 0.45, 1.0))
                .press_color(Color::Rgba(0.12, 0.14, 0.18, 1.0))
                .border(1.0)
                .border_color(Color::Rgba(0.70, 0.60, 0.25, 0.8))
                .set(state.ids.item_loot_btns[i], ui)
                .was_clicked()
            {
                clicked_pickup = Some(entity);
            }
        }

        if let Some(entity) = clicked_pickup {
            return Some(LootWindowEvent::PickUp(entity));
        }

        // Footer background
        RoundedRectangle::fill_with([WINDOW_WIDTH - 8.0, FOOTER_HEIGHT - 6.0], 4.0, Color::Rgba(0.10, 0.11, 0.15, 0.98))
            .mid_bottom_with_margin_on(state.ids.window_bg, 4.0)
            .set(state.ids.footer_bg, ui);

        // "Despojar Todo" button
        if Button::new()
            .w_h(150.0, 28.0)
            .mid_left_with_margin_on(state.ids.footer_bg, 10.0)
            .label("💰 Despojar Todo")
            .label_font_id(self.fonts.cyri.conrod_id)
            .label_font_size(self.fonts.cyri.scale(12))
            .label_color(Color::Rgba(1.0, 0.90, 0.45, 1.0))
            .color(Color::Rgba(0.24, 0.20, 0.10, 0.95))
            .hover_color(Color::Rgba(0.36, 0.30, 0.15, 1.0))
            .press_color(Color::Rgba(0.18, 0.14, 0.08, 1.0))
            .border(1.0)
            .border_color(Color::Rgba(0.85, 0.72, 0.32, 0.9))
            .set(state.ids.loot_all_btn, ui)
            .was_clicked()
        {
            return Some(LootWindowEvent::PickUpAll);
        }

        // "Cerrar" button
        if Button::new()
            .w_h(80.0, 28.0)
            .mid_right_with_margin_on(state.ids.footer_bg, 10.0)
            .label("Cerrar")
            .label_font_id(self.fonts.cyri.conrod_id)
            .label_font_size(self.fonts.cyri.scale(12))
            .label_color(Color::Rgba(0.85, 0.85, 0.85, 1.0))
            .color(Color::Rgba(0.18, 0.20, 0.24, 0.95))
            .hover_color(Color::Rgba(0.26, 0.28, 0.34, 1.0))
            .press_color(Color::Rgba(0.12, 0.14, 0.18, 1.0))
            .border(1.0)
            .border_color(Color::Rgba(0.40, 0.42, 0.48, 0.6))
            .set(state.ids.close_btn_bottom, ui)
            .was_clicked()
        {
            return Some(LootWindowEvent::Close);
        }

        None
    }
}
