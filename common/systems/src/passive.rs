use common::{
    DamageSource,
    class::CharacterClass,
    combat::DamageContributor,
    comp::{
        Alignment, Body, BuffKind, Buffs, CharacterState, Combo, Energy, Health, HealthChange,
        Inventory, Mass, Poise, Stats,
        body::humanoid::Species,
        buff::{Buff, BuffChange, BuffData, BuffSource, DestInfo},
    },
    event::{BuffEvent, EmitExt, HealthChangeEvent},
    event_emitters,
    resources::{Secs, Time},
    uid::Uid,
};
use std::collections::HashMap;
use common_ecs::{Job, Origin, Phase, System};
use specs::{Entities, Join, Read, ReadStorage, SystemData, shred};

event_emitters! {
    struct Events[Emitters] {
        buff: BuffEvent,
        health_change: HealthChangeEvent,
    }
}

/// Identificador único para cada aura pasiva condicional (Racial o de Clase)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PassiveKind {
    // Auras Pasivas Raciales
    OrcBloodfury,
    DwarfStonebastion,
    ElfSylvanAgility,
    HumanTenacity,
    DanariElementalFocus,
    DraugrDeathTouch,
    // Auras Pasivas de Clase
    TamerFeralBond,
    WarriorColossusRage,
    PaladinHolyBulwark,
    RogueShadowDance,
    HunterEagleEye,
    MageArcaneIgnition,
    PriestDivineGrace,
}

#[derive(SystemData)]
pub struct ReadData<'a> {
    entities: Entities<'a>,
    time: Read<'a, Time>,
    events: Events<'a>,
    uids: ReadStorage<'a, Uid>,
    bodies: ReadStorage<'a, Body>,
    healths: ReadStorage<'a, Health>,
    buffs: ReadStorage<'a, Buffs>,
    stats: ReadStorage<'a, Stats>,
    masses: ReadStorage<'a, Mass>,
    energies: ReadStorage<'a, Energy>,
    combos: ReadStorage<'a, Combo>,
    poises: ReadStorage<'a, Poise>,
    char_states: ReadStorage<'a, CharacterState>,
    char_classes: ReadStorage<'a, CharacterClass>,
    inventories: ReadStorage<'a, Inventory>,
    alignments: ReadStorage<'a, Alignment>,
}

#[derive(Default)]
pub struct Sys {
    cooldowns: HashMap<(Uid, PassiveKind), Time>,
    last_cleanup: f64,
}

impl<'a> System<'a> for Sys {
    type SystemData = ReadData<'a>;

    const NAME: &'static str = "passive";
    const ORIGIN: Origin = Origin::Common;
    const PHASE: Phase = Phase::Create;

    fn run(job: &mut Job<Self>, read_data: Self::SystemData) {
        let current_time = *read_data.time;
        let mut emitters = read_data.events.get_emitters();

        // Limpieza periódica de cooldowns expirados cada 10 segundos
        if current_time.0 - job.own.last_cleanup > 10.0 {
            job.own
                .cooldowns
                .retain(|_, &mut end_time| current_time.0 < end_time.0);
            job.own.last_cleanup = current_time.0;
        }

        // Iterar sobre entidades vivas con UID, Salud y Cuerpo
        for (entity, uid, body, health) in (
            &read_data.entities,
            &read_data.uids,
            &read_data.bodies,
            &read_data.healths,
        )
            .join()
        {
            if health.is_dead {
                continue;
            }

            let uid = *uid;
            let dest_info = DestInfo {
                stats: read_data.stats.get(entity),
                mass: read_data.masses.get(entity),
            };
            let mass = read_data.masses.get(entity);
            let entity_buffs = read_data.buffs.get(entity);
            let char_state = read_data.char_states.get(entity);
            let energy = read_data.energies.get(entity);
            let poise = read_data.poises.get(entity);
            let combo = read_data.combos.get(entity);

            // ==============================================================
            // 1. AURAS PASIVAS RACIALES (Para humanoides: 6 razas jugables)
            // ==============================================================
            if let Body::Humanoid(humanoid) = body {
                match humanoid.species {
                    Species::Orc => {
                        // Orco: Furia de Sangre / Sed de Batalla
                        // Condición: Salud cae por debajo del 35%
                        const ICD: f64 = 45.0;
                        if health.fraction() < 0.35 && can_proc(&job.own.cooldowns, uid, PassiveKind::OrcBloodfury, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::OrcBloodfury, current_time, ICD);

                            // Buffs: Berserk (0.8, 8s) + Hastened (0.4, 8s)
                            apply_buff(entity, uid, BuffKind::Berserk, 0.8, 8.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::Hastened, 0.4, 8.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);

                            // Curación instantánea de emergencia: 10% salud máxima
                            let heal_amount = health.maximum() * 0.10;
                            apply_heal(entity, uid, heal_amount, current_time, &mut emitters);
                        }
                    },
                    Species::Dwarf => {
                        // Enano: Bastión de Piedra / Indomable
                        // Condición: Salud < 40% o Poise < 30%
                        const ICD: f64 = 45.0;
                        let poise_low = poise.map_or(false, |p| p.fraction() < 0.30);
                        if (health.fraction() < 0.40 || poise_low) && can_proc(&job.own.cooldowns, uid, PassiveKind::DwarfStonebastion, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::DwarfStonebastion, current_time, ICD);

                            // Buffs: ProtectingWard (0.8, 6s) + Fortitude (1.0, 6s)
                            apply_buff(entity, uid, BuffKind::ProtectingWard, 0.8, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::Fortitude, 1.0, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                    Species::Elf => {
                        // Elfo: Agilidad Silvana / Viento Ancestral
                        // Condición: Esquiva (Roll) o energía < 25% atacando
                        const ICD: f64 = 30.0;
                        let is_rolling = matches!(char_state, Some(CharacterState::Roll(_)));
                        let energy_low_attacking = energy.map_or(false, |e| e.fraction() < 0.25)
                            && char_state.map_or(false, |cs| cs.is_attack());

                        if (is_rolling || energy_low_attacking) && can_proc(&job.own.cooldowns, uid, PassiveKind::ElfSylvanAgility, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::ElfSylvanAgility, current_time, ICD);

                            // Buffs: Hastened (0.6, 6s) + EnergyRegen (20.0, 6s)
                            apply_buff(entity, uid, BuffKind::Hastened, 0.6, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::EnergyRegen, 20.0, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                    Species::Human => {
                        // Humano: Espíritu Inquebrantable / Determinación
                        // Condición: Salud < 50%
                        const ICD: f64 = 40.0;
                        if health.fraction() < 0.50 && can_proc(&job.own.cooldowns, uid, PassiveKind::HumanTenacity, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::HumanTenacity, current_time, ICD);

                            // Buffs: Tenacity (0.8, 6s) + Regeneration (35.0, 6s)
                            apply_buff(entity, uid, BuffKind::Tenacity, 0.8, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::Regeneration, 35.0, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                    Species::Danari => {
                        // Danari: Armonía Elemental / Enfoque Danari
                        // Condición: Combo >= 6 o energía alta (> 75%) en ataque
                        const ICD: f64 = 35.0;
                        let high_combo = combo.map_or(false, |c| c.counter() >= 6);
                        let high_energy_attacking = energy.map_or(false, |e| e.fraction() > 0.75)
                            && char_state.map_or(false, |cs| cs.is_attack());

                        if (high_combo || high_energy_attacking) && can_proc(&job.own.cooldowns, uid, PassiveKind::DanariElementalFocus, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::DanariElementalFocus, current_time, ICD);

                            // Buffs: Fury (0.5, 7s) + Sunderer (0.6, 7s)
                            apply_buff(entity, uid, BuffKind::Fury, 0.5, 7.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::Sunderer, 0.6, 7.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                    Species::Draugr => {
                        // Draugr: Toque de la Muerte / Furia Helada
                        // Condición: Salud < 30%
                        const ICD: f64 = 45.0;
                        if health.fraction() < 0.30 && can_proc(&job.own.cooldowns, uid, PassiveKind::DraugrDeathTouch, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::DraugrDeathTouch, current_time, ICD);

                            // Buffs: Lifesteal (0.5, 7s) + Frigid (0.5, 7s)
                            apply_buff(entity, uid, BuffKind::Lifesteal, 0.5, 7.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::Frigid, 0.5, 7.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                }
            }

            // ==============================================================
            // 2. AURAS PASIVAS DE CLASE (7 clases)
            // ==============================================================
            let character_class = read_data
                .char_classes
                .get(entity)
                .copied()
                .or_else(|| read_data.inventories.get(entity).map(CharacterClass::from_inventory));

            if let Some(class) = character_class {
                match class {
                    CharacterClass::Tamer => {
                        // Entrenador: Vínculo Feral / Manada Imparable
                        // Condición: Salud cae por debajo del 50%
                        const ICD: f64 = 40.0;
                        if health.fraction() < 0.50 && can_proc(&job.own.cooldowns, uid, PassiveKind::TamerFeralBond, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::TamerFeralBond, current_time, ICD);

                            // Buff Frenzied (0.8, 8s) + 15% curación instantánea al dueño
                            apply_buff(entity, uid, BuffKind::Frenzied, 0.8, 8.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            let owner_heal = health.maximum() * 0.15;
                            apply_heal(entity, uid, owner_heal, current_time, &mut emitters);

                            // Curar y potenciar también a las mascotas del Entrenador
                            for (pet_entity, pet_health, pet_alignment, pet_uid) in (
                                &read_data.entities,
                                &read_data.healths,
                                &read_data.alignments,
                                &read_data.uids,
                            )
                                .join()
                            {
                                if let Alignment::Owned(owner) = pet_alignment {
                                    if *owner == uid && *pet_uid != uid && !pet_health.is_dead {
                                        let pet_heal = pet_health.maximum() * 0.15;
                                        apply_heal(pet_entity, *pet_uid, pet_heal, current_time, &mut emitters);

                                        let pet_dest_info = DestInfo {
                                            stats: read_data.stats.get(pet_entity),
                                            mass: read_data.masses.get(pet_entity),
                                        };
                                        apply_buff(
                                            pet_entity,
                                            *pet_uid,
                                            BuffKind::Frenzied,
                                            0.8,
                                            8.0,
                                            current_time,
                                            pet_dest_info,
                                            read_data.masses.get(pet_entity),
                                            read_data.buffs.get(pet_entity),
                                            &mut emitters,
                                        );
                                    }
                                }
                            }
                        }
                    },
                    CharacterClass::Warrior => {
                        // Guerrero: Ira del Coloso
                        // Condición: Energía > 40 en combate/ataque
                        const ICD: f64 = 25.0;
                        let high_energy = energy.map_or(false, |e| e.current() > 40.0);
                        let is_attacking = char_state.map_or(false, |cs| cs.is_attack());

                        if high_energy && is_attacking && can_proc(&job.own.cooldowns, uid, PassiveKind::WarriorColossusRage, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::WarriorColossusRage, current_time, ICD);

                            // Buffs: ImminentCritical (1.0, 6s) + Defiance (1.0, 6s)
                            apply_buff(entity, uid, BuffKind::ImminentCritical, 1.0, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::Defiance, 1.0, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                    CharacterClass::Paladin => {
                        // Paladín: Baluarte Sagrado
                        // Condición: Salud < 50%
                        const ICD: f64 = 35.0;
                        if health.fraction() < 0.50 && can_proc(&job.own.cooldowns, uid, PassiveKind::PaladinHolyBulwark, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::PaladinHolyBulwark, current_time, ICD);

                            // Buffs: ProtectingWard (0.6, 6s) + Regeneration (30.0, 6s)
                            apply_buff(entity, uid, BuffKind::ProtectingWard, 0.6, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::Regeneration, 30.0, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                    CharacterClass::Rogue => {
                        // Pícaro: Danza de las Sombras
                        // Condición: Esquiva (Roll) o Combo >= 4
                        const ICD: f64 = 25.0;
                        let is_rolling = matches!(char_state, Some(CharacterState::Roll(_)));
                        let good_combo = combo.map_or(false, |c| c.counter() >= 4);

                        if (is_rolling || good_combo) && can_proc(&job.own.cooldowns, uid, PassiveKind::RogueShadowDance, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::RogueShadowDance, current_time, ICD);

                            // Buffs: Hastened (0.8, 5s) + ImminentCritical (1.0, 5s)
                            apply_buff(entity, uid, BuffKind::Hastened, 0.8, 5.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::ImminentCritical, 1.0, 5.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                    CharacterClass::Hunter => {
                        // Cazador: Ojo de Halcón
                        // Condición: Combo >= 3 o Ataque a Distancia
                        const ICD: f64 = 30.0;
                        let combo_proc = combo.map_or(false, |c| c.counter() >= 3);
                        let ranged_proc = char_state.map_or(false, |cs| cs.is_ranged());

                        if (combo_proc || ranged_proc) && can_proc(&job.own.cooldowns, uid, PassiveKind::HunterEagleEye, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::HunterEagleEye, current_time, ICD);

                            // Buffs: EagleEye (0.8, 6s) + StormChaser (0.6, 6s)
                            apply_buff(entity, uid, BuffKind::EagleEye, 0.8, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::StormChaser, 0.6, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                    CharacterClass::Mage => {
                        // Mago: Ignición Arcana
                        // Condición: Energía < 30% en ataque
                        const ICD: f64 = 30.0;
                        let low_energy = energy.map_or(false, |e| e.fraction() < 0.30);
                        let is_attacking = char_state.map_or(false, |cs| cs.is_attack());

                        if low_energy && is_attacking && can_proc(&job.own.cooldowns, uid, PassiveKind::MageArcaneIgnition, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::MageArcaneIgnition, current_time, ICD);

                            // Buffs: Flame (0.8, 6s) + EnergyRegen (25.0, 6s)
                            apply_buff(entity, uid, BuffKind::Flame, 0.8, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::EnergyRegen, 25.0, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                    CharacterClass::Priest => {
                        // Sacerdote: Gracia Divina
                        // Condición: Salud < 45%
                        const ICD: f64 = 35.0;
                        if health.fraction() < 0.45 && can_proc(&job.own.cooldowns, uid, PassiveKind::PriestDivineGrace, current_time) {
                            set_cooldown(&mut job.own.cooldowns, uid, PassiveKind::PriestDivineGrace, current_time, ICD);

                            // Buffs: ProtectingWard (0.7, 6s) + Regeneration (40.0, 6s)
                            apply_buff(entity, uid, BuffKind::ProtectingWard, 0.7, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::Regeneration, 40.0, 6.0, current_time, dest_info, mass, entity_buffs, &mut emitters);
                            apply_buff(entity, uid, BuffKind::PassiveCooldown, 1.0, ICD, current_time, dest_info, mass, entity_buffs, &mut emitters);
                        }
                    },
                }
            }
        }
    }
}

/// Comprueba si la pasiva no se encuentra en enfriamiento interno (ICD)
fn can_proc(cooldowns: &HashMap<(Uid, PassiveKind), Time>, uid: Uid, kind: PassiveKind, current_time: Time) -> bool {
    cooldowns.get(&(uid, kind)).is_none_or(|&end_time| current_time.0 >= end_time.0)
}

/// Registra el tiempo de expiración del enfriamiento interno
fn set_cooldown(
    cooldowns: &mut HashMap<(Uid, PassiveKind), Time>,
    uid: Uid,
    kind: PassiveKind,
    current_time: Time,
    duration_secs: f64,
) {
    cooldowns.insert((uid, kind), Time(current_time.0 + duration_secs));
}

/// Emite un evento para añadir un Buff a la entidad
fn apply_buff(
    entity: specs::Entity,
    uid: Uid,
    kind: BuffKind,
    strength: f32,
    duration_secs: f64,
    current_time: Time,
    dest_info: DestInfo,
    mass: Option<&Mass>,
    buffs: Option<&Buffs>,
    emitters: &mut impl EmitExt<BuffEvent>,
) {
    // Si la entidad ya tiene un buff igual o más fuerte de este tipo, no reemplazarlo innecesariamente
    if let Some(buffs) = buffs {
        if kind != BuffKind::PassiveCooldown
            && buffs.buffs.iter().any(|(_, b)| b.kind == kind && b.data.strength >= strength)
        {
            return;
        }
    }

    let mut data = BuffData::new(strength, Some(Secs(duration_secs)));
    if matches!(kind, BuffKind::Flame | BuffKind::Frigid) {
        data = data.with_secondary_duration(Secs(5.0));
    }

    emitters.emit(BuffEvent {
        entity,
        buff_change: BuffChange::Add(Buff::new(
            kind,
            data,
            vec![],
            BuffSource::Character {
                by: uid,
                tool_kind: None,
            },
            current_time,
            dest_info,
            mass,
            None,
        )),
    });
}

/// Emite un evento de curación instantánea
fn apply_heal(
    entity: specs::Entity,
    uid: Uid,
    amount: f32,
    current_time: Time,
    emitters: &mut impl EmitExt<HealthChangeEvent>,
) {
    if amount <= 0.0 {
        return;
    }

    emitters.emit(HealthChangeEvent {
        entity,
        change: HealthChange {
            amount,
            by: Some(DamageContributor::Solo(uid)),
            cause: Some(DamageSource::Buff(BuffKind::Regeneration)),
            time: current_time,
            precise: false,
            instance: rand::random(),
        },
    });
}
