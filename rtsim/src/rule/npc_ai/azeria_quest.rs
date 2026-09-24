//! Diálogo de las misiones estilo World of Warcraft de World of Azeria: los NPC
//! de pueblo ofrecen, siguen y reciben las misiones de `common::quest` según su
//! profesión y el continente en el que viven.

use super::*;
use common::{
    comp::Item,
    quest::{self, ActiveQuests, QuestDefinition, QuestGiver, QuestProgress},
    zone::FactionId,
};

/// Objeto con el que se pagan las monedas de cobre de las recompensas
const COINS_ITEM: &str = "common.items.utility.coins";

/// Copia de las misiones y la facción del jugador con el que habla el NPC.
/// `None` si no es un jugador con misiones.
fn player_state(ctx: &NpcCtx, player: ActorId) -> Option<(ActiveQuests, Option<FactionId>)> {
    let entity = ctx.system_data.id_maps.rtsim_entity(player)?;
    let quests = ctx
        .system_data
        .active_quests
        .lock()
        .ok()?
        .get(entity)
        .cloned()?;
    let faction = ctx.system_data.players.get(entity).and_then(|p| p.faction);
    Some((quests, faction))
}

/// Modifica las misiones del jugador
fn with_player_quests<T>(
    ctx: &NpcCtx,
    player: ActorId,
    f: impl FnOnce(&mut ActiveQuests) -> T,
) -> Option<T> {
    let entity = ctx.system_data.id_maps.rtsim_entity(player)?;
    let mut storage = ctx.system_data.active_quests.lock().ok()?;
    let mut quests = storage.get_mut(entity)?;
    Some(f(&mut *quests))
}

/// Objetos que componen la recompensa (monedas incluidas)
fn reward_items(quest: &QuestDefinition) -> Vec<Item> {
    let mut items = Vec::new();
    let mut add = |item_id: &str, amount: u32| {
        if amount == 0 {
            return;
        }
        let mut item = Item::new_from_asset_expect(item_id);
        if item.set_amount(amount).is_ok() {
            items.push(item);
        } else {
            // No se apila: un objeto por unidad
            items.push(item);
            for _ in 1..amount {
                items.push(Item::new_from_asset_expect(item_id));
            }
        }
    };
    add(COINS_ITEM, quest.rewards.coins);
    for (item_id, amount) in &quest.rewards.items {
        add(item_id, *amount);
    }
    items
}

/// Huecos libres que tiene el jugador en el inventario
fn free_slots(ctx: &NpcCtx, player: ActorId) -> usize {
    ctx.system_data
        .id_maps
        .rtsim_entity(player)
        .and_then(|entity| {
            ctx.system_data
                .inventories
                .lock()
                .ok()?
                .get(entity)
                .map(|inv| inv.free_slots())
        })
        .unwrap_or(0)
}

/// Entrega al jugador la experiencia, las monedas y los objetos de la misión
fn give_rewards(ctx: &NpcCtx, player: ActorId, quest: &QuestDefinition) {
    let Some(entity) = ctx.system_data.id_maps.rtsim_entity(player) else {
        return;
    };
    if let Ok(mut skill_sets) = ctx.system_data.skill_sets.lock()
        && let Some(mut skill_set) = skill_sets.get_mut(entity)
    {
        quest::grant_xp(&mut *skill_set, quest.rewards.xp);
    }
    if let Ok(mut inventories) = ctx.system_data.inventories.lock()
        && let Some(mut inventory) = inventories.get_mut(entity)
    {
        for item in reward_items(quest) {
            // Ya se comprobó antes que había sitio suficiente
            let _ = inventory.push(item);
        }
    }
}

/// "5 plata, 20 cobre" a partir de monedas de cobre
fn coins_text(coins: u32) -> String {
    let (gold, silver, copper) = (coins / 10_000, (coins / 100) % 100, coins % 100);
    let mut parts = Vec::new();
    if gold > 0 {
        parts.push(format!("{gold} oro"));
    }
    if silver > 0 {
        parts.push(format!("{silver} plata"));
    }
    if copper > 0 || parts.is_empty() {
        parts.push(format!("{copper} cobre"));
    }
    parts.join(", ")
}

fn rewards_text(quest: &QuestDefinition) -> String {
    let mut text = format!(
        "{} de experiencia y {}",
        quest.rewards.xp,
        coins_text(quest.rewards.coins)
    );
    if !quest.rewards.items.is_empty() {
        text.push_str(", además de algunos objetos");
    }
    text
}

fn objectives_text(quest: &QuestDefinition, progress: Option<&QuestProgress>) -> String {
    quest
        .objectives
        .iter()
        .enumerate()
        .map(|(i, objective)| {
            format!(
                "- {}: {}/{}",
                objective.label(),
                progress.map_or(0, |p| p.count(i)),
                objective.required()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Respuestas de misión que el NPC añade a su menú de diálogo: entregar las
/// misiones terminadas, preguntar por las que están en curso y ofrecer las
/// disponibles
pub fn quest_responses<S: State>(
    ctx: &NpcCtx,
    player: ActorId,
    session: DialogueSession,
    giver: QuestGiver,
) -> Vec<(Response, Box<dyn Action<S>>)> {
    let zone = common::zone::get_zone_at(ctx.actor.wpos.xy()).id;
    let Some((quests, faction)) = player_state(ctx, player) else {
        return Vec::new();
    };

    let mut responses: Vec<(Response, Box<dyn Action<S>>)> = Vec::new();

    for (progress, quest) in quests.active_for(giver, zone) {
        if progress.is_complete() {
            responses.push((
                Response::from(Content::Plain(format!("Entregar misión: {}", quest.title))),
                turn_in(session, player, quest).boxed(),
            ));
        } else {
            let status = objectives_text(quest, Some(progress));
            responses.push((
                Response::from(Content::Plain(format!("Misión en curso: {}", quest.title))),
                session
                    .say_statement(Content::Plain(format!("{}\n\n{}", quest.progress_text, status)))
                    .boxed(),
            ));
        }
    }

    for quest in quests.available_from(giver, zone, faction) {
        responses.push((
            Response::from(Content::Plain(format!(
                "Misión: {} (Nvl {})",
                quest.title, quest.min_level
            ))),
            offer(session, player, quest).boxed(),
        ));
    }

    responses
}

/// El NPC cuenta la misión y pregunta si la aceptas
fn offer<S: State>(
    session: DialogueSession,
    player: ActorId,
    quest: &'static QuestDefinition,
) -> impl Action<S> {
    session
        .say_statement(Content::Plain(format!(
            "[{}]\n{}",
            quest.chain, quest.offer_text
        )))
        .then(session.say_statement(Content::Plain(format!(
            "Objetivos:\n{}\n\nRecompensa: {}.",
            objectives_text(quest, None),
            rewards_text(quest)
        ))))
        .then(session.ask_yes_no_question(Content::Plain("¿Aceptas la misión?".to_string())))
        .and_then(move |yes: bool| {
            now(move |ctx, _| {
                if !yes {
                    session
                        .say_statement(Content::localized("npc-response-quest-rejected"))
                        .boxed()
                } else if with_player_quests(ctx, player, |quests| quests.accept(quest))
                    .unwrap_or(false)
                {
                    session
                        .say_statement(Content::Plain(format!(
                            "Misión aceptada: {}. ¡Buena suerte!",
                            quest.title
                        )))
                        .boxed()
                } else {
                    session
                        .say_statement(Content::Plain("Ya tienes esta misión.".to_string()))
                        .boxed()
                }
            })
        })
}

/// Entrega de una misión terminada: recompensa y cierre de la misión
fn turn_in<S: State>(
    session: DialogueSession,
    player: ActorId,
    quest: &'static QuestDefinition,
) -> impl Action<S> {
    now(move |ctx, _| {
        let needed_slots = reward_items(quest).len();
        if free_slots(ctx, player) < needed_slots {
            return session
                .say_statement(Content::Plain(format!(
                    "Haz sitio en tu inventario antes de recoger la recompensa: necesitas \
                     {needed_slots} huecos libres."
                )))
                .boxed();
        }
        if with_player_quests(ctx, player, |quests| quests.turn_in(&quest.id))
            .flatten()
            .is_some()
        {
            give_rewards(ctx, player, quest);
            session
                .say_statement(Content::Plain(format!(
                    "{}\n\nRecompensa: {}.",
                    quest.complete_text,
                    rewards_text(quest)
                )))
                .boxed()
        } else {
            session
                .say_statement(Content::Plain(
                    "Todavía no has terminado lo que te pedí.".to_string(),
                ))
                .boxed()
        }
    })
}
