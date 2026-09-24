use specs::{Join, LendJoin, WorldExt};
use vek::*;

use client::{self, Client};
use common::{
    comp::{self, Health, tool::ToolKind},
    consts::{MAX_INTERACT_RANGE, MAX_PICKUP_RANGE},
    link::Is,
    mounting::{Mount, Rider},
    uid::Uid,
    util::find_dist::{Cylinder, FindDist},
    vol::ReadVol,
};
use common_base::span;

#[derive(Clone, Copy, Debug)]
pub struct Target<T> {
    pub kind: T,
    pub position: Vec3<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Build(pub Vec3<f32>);

#[derive(Clone, Copy, Debug)]
pub struct Collectable;

#[derive(Clone, Copy, Debug)]
pub struct Entity(pub specs::Entity);

#[derive(Clone, Copy, Debug)]
pub struct Mine;

#[derive(Clone, Copy, Debug)]
// line of sight (if not bocked by entity). Not build/mine mode dependent.
pub struct Terrain;

impl<T> Target<T> {
    pub fn position_int(self) -> Vec3<i32> { self.position.map(|p| p.floor() as i32) }
}

/// Max distance an entity can be "targeted"
pub const MAX_TARGET_RANGE: f32 = 500.0;

/// Calculate what the cursor is pointing at within the 3d scene
pub(super) fn targets_under_cursor(
    client: &Client,
    cam_pos: Vec3<f32>,
    cam_dir: Vec3<f32>,
    can_build: bool,
    active_mine_tool: Option<ToolKind>,
    viewpoint_entity: specs::Entity,
) -> (
    Option<Target<Build>>,
    Option<Target<Collectable>>,
    Option<Target<Entity>>,
    Option<Target<Mine>>,
    Option<Target<Terrain>>,
) {
    span!(_guard, "targets_under_cursor");
    // Choose a spot above the player's head for item distance checks
    let player_entity = client.entity();
    let ecs = client.state().ecs();
    let positions = ecs.read_storage::<comp::Pos>();
    let player_pos = match positions.get(player_entity) {
        Some(pos) => pos.0,
        None => cam_pos, // Should never happen, but a safe fallback
    };
    let scales = ecs.read_storage();
    let colliders = ecs.read_storage();
    let char_states = ecs.read_storage::<comp::CharacterState>();
    let player_char_state = char_states.get(player_entity);
    let player_scale = scales.get(player_entity).copied();
    // Get the player's cylinder
    let player_cylinder = Cylinder::from_components(
        player_pos,
        player_scale,
        colliders.get(player_entity),
        player_char_state,
    );
    let eye_height = ecs
        .read_storage::<comp::Body>()
        .get(player_entity)
        .map(|b| b.eye_height(player_scale.map_or(1.0, |s| s.0)))
        .unwrap_or(0.0);
    let eye_pos = player_pos + Vec3::unit_z() * eye_height;
    let terrain = client.state().terrain();

    // The maximum distance we might need to go, plus a safe threshold
    const MAX_RAY_DIST: f32 = MAX_TARGET_RANGE
        .max(MAX_PICKUP_RANGE)
        .max(MAX_INTERACT_RANGE)
        + 1.0;
    let ray = terrain
        .ray(cam_pos, cam_pos + cam_dir * MAX_RAY_DIST)
        .max_iter(500);

    let break_tgt_pos = |dist: f32| (cam_pos + cam_dir * (dist + 0.01)).map(|e| e.floor() + 0.5);
    let place_tgt_pos = |dist: f32| (cam_pos + cam_dir * (dist - 0.01)).map(|e| e.floor() + 0.5);

    let collect_cast = Some(ray.until(|b| b.is_solid() || b.is_directly_collectible()).cast())
        .filter(|(_, b)| matches!(b, Ok(Some(b)) if b.is_directly_collectible()))
        // Collection is limited by the player's actual position
        .filter(|(d, _)| player_pos.distance(break_tgt_pos(*d)) < MAX_INTERACT_RANGE);
    let mine_cast = Some(ray.until(|b| b.is_solid() || b.mine_tool().is_some()).cast())
        // Mining is limited by the target block being mineable with the active mining tool...
        .filter(|(_, b)| matches!(b, Ok(Some(b)) if b.mine_tool().zip(active_mine_tool).map_or(false, |(a, b)| a == b)))
        // ...and by the distance to the player's eye position
        .filter(|(d, _)| eye_pos.distance(break_tgt_pos(*d)) < MAX_PICKUP_RANGE);
    let build_cast = Some(ray.until(|b| b.is_solid()).cast())
        // Building is limited by the maximum target distance
        .filter(|(d, _)| *d < MAX_TARGET_RANGE);
    // Visual obstacles are limited by filled blocks
    let obstacle_cast =
        Some(ray.until(|b| b.is_filled()).cast()).filter(|(d, _)| *d < MAX_TARGET_RANGE);

    // The maximum distance at which entities can be targetted is based on the
    // distance to the nearest terrain obstacle being looked at
    let max_target_dist = obstacle_cast.map_or(MAX_TARGET_RANGE, |(d, _)| d);
    let cam_segment = LineSegment3 {
        start: cam_pos,
        end: cam_pos + cam_dir * max_target_dist,
    };

    let uids = ecs.read_storage::<Uid>();

    // Need to raycast by distance to cam
    // But also filter out by distance to the player (but this only needs to be done
    // on final result)
    let player_wielding = player_char_state.is_some_and(|cs| cs.is_wield());
    let mut nearby = (
        &ecs.entities(),
        &positions,
        scales.maybe(),
        &ecs.read_storage::<comp::Body>(),
        ecs.read_storage::<comp::PickupItem>().maybe(),
        !&ecs.read_storage::<Is<Mount>>(),
        ecs.read_storage::<Is<Rider>>().maybe(),
        ecs.read_storage::<Health>().maybe(),
    )
        .join()
        .filter(|(e, _, _, _, _, _, _, _)| *e != viewpoint_entity)
        .filter_map(|(e, p, s, b, i, _, is_rider, health)| {
            const RADIUS_SCALE: f32 = 1.15;
            // TODO: use collider radius instead of body radius?
            let radius = s.map_or(1.0, |s| s.0) * (b.dimensions() * Vec3::new(1.0, 1.0, 0.5)).reduce_partial_max() * RADIUS_SCALE;
            let height = s.map_or(1.0, |s| s.0) * b.height();
            // Move position up from the feet
            let pos = Vec3::new(p.0.x, p.0.y, p.0.z + (height / 2.0));
            // Distance squared from camera to the entity
            let dist_sqr = pos.distance_squared(cam_pos);
            // We only care about interacting with entities that contain items,
            // or are not inanimate (to trade with), or have health (and thus can be targeted by abilities); and are not riding the player.
            let not_riding_player = is_rider.is_none_or(|is_rider| Some(&is_rider.mount) != uids.get(viewpoint_entity));
            if (i.is_some() || !matches!(b, comp::Body::Object(_)) || health.is_some()) && not_riding_player {
                Some((e, pos, radius, dist_sqr))
            } else {
                None
            }
        })
        // Roughly filter out entities farther than ray distance
        .filter(|(_, _, r, d_sqr)| *d_sqr <= max_target_dist.powi(2) + 2.0 * max_target_dist * r + r.powi(2))
        // Ignore entities intersecting the camera, unless the player is wielding (heuristic used to decide if player is trying to target an entity with an ability)
        .filter(|(_, _, r, d_sqr)| player_wielding || *d_sqr > r.powi(2))
        // Substract sphere radius from distance to the camera
        .map(|(e, p, r, d_sqr)| {
            (e, p, r, d_sqr.sqrt() - r, cam_segment.distance_to_point(p))
        })
        .collect::<Vec<_>>();

    // Priorizar siempre la entidad que esté más cercana al rayo del cursor
    nearby.sort_unstable_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));

    let seg_ray = LineSegment3 {
        start: cam_pos,
        end: cam_pos + cam_dir * max_target_dist,
    };
    // TODO: fuzzy borders
    let entity_target = nearby
        .iter()
        .map(|(e, p, r, _, _)| (e, *p, r))
        // Find first one that intersects the ray segment
        .find(|(_, p, r)| {
            seg_ray.projected_point(*p).distance_squared(*p) < r.powi(2)
        })
        .and_then(|(e, p, _)| {
            // Get the entity's cylinder
            let target_cylinder = Cylinder::from_components(
                p,
                scales.get(*e).copied(),
                colliders.get(*e),
                char_states.get(*e),
            );

            if player_cylinder.min_distance(target_cylinder) < MAX_TARGET_RANGE {
                Some(Target {
                    kind: Entity(*e),
                    position: p,
                })
            } else { None }
        });

    let terrain_target = obstacle_cast.map(|(d, _)| Target {
        kind: Terrain,
        position: break_tgt_pos(d),
    });

    let build_target = if let (true, Some((d, _))) = (can_build, build_cast) {
        Some(Target {
            kind: Build(place_tgt_pos(d)),
            position: break_tgt_pos(d),
        })
    } else {
        None
    };

    let collect_target = collect_cast.map(|(d, _)| Target {
        kind: Collectable,
        position: break_tgt_pos(d),
    });

    let mine_target = mine_cast.map(|(d, _)| Target {
        kind: Mine,
        position: break_tgt_pos(d),
    });

    // Return multiple possible targets
    // GameInput events determine which target to use.
    (
        build_target,
        collect_target,
        entity_target,
        mine_target,
        terrain_target,
    )
}
