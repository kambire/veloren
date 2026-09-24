#[cfg(not(feature = "worldgen"))]
use crate::test_world::World;
#[cfg(feature = "worldgen")]
use common_net::msg::WorldMapMsg;
#[cfg(feature = "worldgen")]
use world::{IndexOwned, World};

use crate::{
    EditableSettings,
    automod::AutoMod,
    character_creator,
    client::Client,
    persistence::{character_loader::CharacterLoader, character_updater::CharacterUpdater},
};
#[cfg(feature = "worldgen")]
use common::terrain::TerrainChunkSize;
use common::{
    comp::{Admin, AdminRole, ChatType, Content, Player, Presence, Waypoint},
    event::{
        ChatEvent, ClientDisconnectEvent, DeleteCharacterEvent, EmitExt, InitializeCharacterEvent,
        InitializeSpectatorEvent,
    },
    event_emitters,
    resources::Time,
    uid::Uid,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::{ClientGeneral, ServerGeneral};
use specs::{
    Entities, Join, ReadExpect, ReadStorage, SystemData, WriteExpect, WriteStorage, shred,
};
use std::sync::{Arc, atomic::Ordering};
use tracing::debug;

event_emitters! {
    struct Events[Emitters] {
        init_spectator: InitializeSpectatorEvent,
        init_character_data: InitializeCharacterEvent,
        delete_character: DeleteCharacterEvent,
        client_disconnect: ClientDisconnectEvent,
        chat: ChatEvent,
    }
}

impl Sys {
    fn handle_client_character_screen_msg(
        emitters: &mut Emitters,
        entity: specs::Entity,
        client: &Client,
        character_updater: &mut WriteExpect<'_, CharacterUpdater>,
        msg: ClientGeneral,
        data: &ReadData<'_>,
    ) -> Result<(), crate::error::Error> {
        let mut send_join_messages = || -> Result<(), crate::error::Error> {
            // Give the player a welcome message
            let localized_description = data
                .editable_settings
                .server_description
                .get(client.locale.as_deref());
            if !localized_description.is_none_or(|d| d.motd.is_empty()) {
                client.send(ServerGeneral::server_msg(
                    ChatType::CommandInfo,
                    localized_description.map_or(Content::Plain("".to_string()), |d| {
                        Content::Plain(d.motd.to_owned())
                    }),
                ))?;
            }

            // Warn them about automod
            if data.automod.enabled() {
                client.send(ServerGeneral::server_msg(
                    ChatType::CommandInfo,
                    Content::Plain(
                        "Automatic moderation is enabled: play nice and have fun!".to_string(),
                    ),
                ))?;
            }

            if client.client_type.emit_login_events()
                && !client.login_msg_sent.load(Ordering::Relaxed)
                && let Some(player_uid) = data.uids.get(entity)
            {
                emitters.emit(ChatEvent {
                    msg: ChatType::Online(*player_uid).into_plain_msg(""),
                    from_client: false,
                });

                client.login_msg_sent.store(true, Ordering::Relaxed);
            }
            Ok(())
        };
        match msg {
            // Request spectator state
            ClientGeneral::Spectate(requested_view_distances) => {
                if let Some(admin) = data.admins.get(entity)
                    && admin.0 >= AdminRole::Moderator
                {
                    send_join_messages()?;

                    emitters.emit(InitializeSpectatorEvent(entity, requested_view_distances));
                } else {
                    debug!("dropped Spectate msg from unprivileged client")
                }
            },
            ClientGeneral::Character(character_id, requested_view_distances) => {
                if let Some(player) = data.players.get(entity) {
                    // NOTE: Because clients retain their Uid when exiting to the character
                    // selection screen, we rely on this check to prevent them from immediately
                    // re-entering in-game in the same tick so that we can synchronize their
                    // removal without this being mixed up (and e.g. telling other clients to
                    // delete the re-joined version) or requiring more complex handling to avoid
                    // this.
                    if data.presences.contains(entity) {
                        debug!("player already ingame, aborting");
                    } else if character_updater.has_pending_database_action(character_id) {
                        debug!("player recently logged out pending persistence, aborting");
                        client.send(ServerGeneral::CharacterDataLoadResult(Err(
                            "You have recently logged out, please wait a few seconds and try again"
                                .to_string(),
                        )))?;
                    } else if character_updater.disconnect_all_clients_requested() {
                        // If we're in the middle of disconnecting all clients due to a persistence
                        // transaction failure, prevent new logins
                        // temporarily.
                        debug!(
                            "Rejecting player login while pending disconnection of all players is \
                             in progress"
                        );
                        client.send(ServerGeneral::CharacterDataLoadResult(Err(
                            "The server is currently recovering from an error, please wait a few \
                             seconds and try again"
                                .to_string(),
                        )))?;
                    } else {
                        // Send a request to load the character's component data from the
                        // DB. Once loaded, persisted components such as stats and inventory
                        // will be inserted for the entity
                        data.character_loader.load_character_data(
                            entity,
                            player.uuid().to_string(),
                            character_id,
                        );

                        send_join_messages()?;

                        // Start inserting non-persisted/default components for the entity
                        // while we load the DB data
                        emitters.emit(InitializeCharacterEvent {
                            entity,
                            character_id,
                            requested_view_distances,
                        });
                    }
                } else {
                    debug!("Client is not yet registered");
                    client.send(ServerGeneral::CharacterDataLoadResult(Err(String::from(
                        "Failed to fetch player entity",
                    ))))?
                }
            },
            ClientGeneral::RequestCharacterList => {
                if let Some(player) = data.players.get(entity) {
                    data.character_loader
                        .load_character_list(entity, player.uuid().to_string())
                }
            },
            ClientGeneral::CreateCharacter {
                alias,
                mainhand,
                offhand,
                body,
                hardcore,
                start_site: _,
            } => {
                if !common::character::verify_character_name(&alias) {
                    client.send(ServerGeneral::CharacterActionError(
                        "That name is too long".to_owned(),
                    ))?;
                } else if data.censor.check(&alias) {
                    debug!(?alias, "denied alias as it contained a banned word");
                    client.send(ServerGeneral::CharacterActionError(format!(
                        "Alias '{}' contains a banned word",
                        alias
                    )))?;
                } else if let Some(player) = data.players.get(entity) {
                    #[cfg(feature = "worldgen")]
                    let waypoint = {
                        let target_wpos2d = match body {
                            common::comp::Body::Humanoid(humanoid_body) => {
                                let total = data.world_map_msg.possible_starting_sites.len();
                                let site_idx_opt = if total > 0 {
                                    let s_idx = common::zone::species_starting_site_index(
                                        humanoid_body.species,
                                        total,
                                    );
                                    data.world_map_msg.possible_starting_sites.get(s_idx).copied()
                                } else {
                                    None
                                };

                                site_idx_opt
                                    .and_then(|site_idx| {
                                        data.world
                                            .civs()
                                            .sites
                                            .iter()
                                            .find(|(_, site)| site.site_tmp.map(|i| i.id()) == Some(site_idx))
                                            .map(|(_, site)| TerrainChunkSize::center_wpos(site.center))
                                    })
                                    .unwrap_or_else(|| common::zone::get_species_spawn_wpos(humanoid_body.species))
                            },
                            _ => common::zone::get_faction_spawn_wpos(common::zone::FactionId::Alliance),
                        };

                        // Garantizar que el spawn esté en un suelo plano y transitable
                        let safe_flat_pos = data.world.find_flat_accessible_pos(
                            data.index.as_index_ref(),
                            target_wpos2d,
                            10,
                        );

                        Some(Waypoint::new(safe_flat_pos, *data.time))
                    };
                    #[cfg(not(feature = "worldgen"))]
                    let waypoint = Some(Waypoint::new(
                        data.world.get_center().with_z(10).as_(),
                        *data.time,
                    ));
                    if let Err(error) = character_creator::create_character(
                        entity,
                        player.uuid().to_string(),
                        alias,
                        mainhand.clone(),
                        offhand.clone(),
                        body,
                        hardcore,
                        character_updater,
                        waypoint,
                    ) {
                        debug!(
                            ?error,
                            ?mainhand,
                            ?offhand,
                            ?body,
                            "Denied creating character because of invalid input."
                        );
                        client.send(ServerGeneral::CharacterActionError(error.to_string()))?;
                    }
                }
            },
            ClientGeneral::EditCharacter { id, alias, body } => {
                if !common::character::verify_character_name(&alias) {
                    client.send(ServerGeneral::CharacterActionError(
                        "That name is too long".to_owned(),
                    ))?;
                } else if data.censor.check(&alias) {
                    debug!(?alias, "denied alias as it contained a banned word");
                    client.send(ServerGeneral::CharacterActionError(format!(
                        "Alias '{}' contains a banned word",
                        alias
                    )))?;
                } else if let Some(player) = data.players.get(entity)
                    && let Err(error) = character_creator::edit_character(
                        entity,
                        player.uuid().to_string(),
                        id,
                        alias,
                        body,
                        character_updater,
                    )
                {
                    debug!(
                        ?error,
                        ?body,
                        "Denied editing character because of invalid input."
                    );
                    client.send(ServerGeneral::CharacterActionError(error.to_string()))?;
                }
            },
            ClientGeneral::DeleteCharacter(character_id) => {
                if let Some(player) = data.players.get(entity) {
                    emitters.emit(DeleteCharacterEvent {
                        entity,
                        requesting_player_uuid: player.uuid().to_string(),
                        character_id,
                    });
                }
            },
            _ => {
                debug!("Kicking possibly misbehaving client due to invalid character request");
                emitters.emit(ClientDisconnectEvent(
                    entity,
                    common::comp::DisconnectReason::NetworkError,
                ));
            },
        }
        Ok(())
    }
}

#[derive(SystemData)]
pub struct WriteData<'a> {
    clients: WriteStorage<'a, Client>,
    character_updater: WriteExpect<'a, CharacterUpdater>,
}

#[derive(SystemData)]
pub struct ReadData<'a> {
    entities: Entities<'a>,
    events: Events<'a>,
    character_loader: ReadExpect<'a, CharacterLoader>,
    uids: ReadStorage<'a, Uid>,
    players: ReadStorage<'a, Player>,
    admins: ReadStorage<'a, Admin>,
    presences: ReadStorage<'a, Presence>,
    editable_settings: ReadExpect<'a, EditableSettings>,
    censor: ReadExpect<'a, Arc<censor::Censor>>,
    automod: ReadExpect<'a, AutoMod>,
    time: ReadExpect<'a, Time>,
    world: ReadExpect<'a, Arc<World>>,

    #[cfg(feature = "worldgen")]
    index: ReadExpect<'a, IndexOwned>,
    #[cfg(feature = "worldgen")]
    world_map_msg: ReadExpect<'a, WorldMapMsg>,
}

/// This system will handle new messages from clients
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (WriteData<'a>, ReadData<'a>);

    const NAME: &'static str = "msg::character_screen";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(_job: &mut Job<Self>, (mut write_data, read_data): Self::SystemData) {
        let mut emitters = read_data.events.get_emitters();

        for (entity, client) in (&read_data.entities, &mut write_data.clients).join() {
            let _ = super::try_recv_all(client, 1, |client, msg| {
                Self::handle_client_character_screen_msg(
                    &mut emitters,
                    entity,
                    client,
                    &mut write_data.character_updater,
                    msg,
                    &read_data,
                )
            });
        }
    }
}
