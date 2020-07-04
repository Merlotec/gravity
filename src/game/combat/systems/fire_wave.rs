use amethyst::{
    assets::{
        Handle,
        Prefab,
        Loader,
        RonFormat,
        PrefabLoader,
    },
    core::{
        math::{
            Vector3,
            Vector4,
            UnitQuaternion,
            Quaternion,
        },
        Named,
        Parent,
        ParentHierarchy,
        SystemDesc,
        Time,
        Transform,
    },
    ecs::prelude::*,
    shrev::{
        EventChannel,
        ReaderId,
    },
};
use combat_render::flash::Flash;
use rand::Rng;

use crate::game::character::{Character, WeaponSlot};
use crate::game::combat::ability::{DmgPackage, DmgTimer, Element, FireWaveEvent, MissEvent, StatusInflictDesc, WaveDmg};
use crate::game::map::WorldPrefabData;
use crate::game::combat::{CombatRoot, CombatState};

#[derive(Debug, Clone, PartialEq, Component)]
pub struct Wave {
    pub timer: f32,
    pub rate: f32,
    pub targets: Vec<Entity>,
    pub source: Option<Entity>,
    pub dmg: Option<WaveDmg>,
}

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(FireWaveSystemDesc))]
pub struct FireWaveSystem {
    wave: Option<Handle<Prefab<WorldPrefabData>>>,
    #[system_desc(event_channel_reader)]
    reader: ReaderId<FireWaveEvent>,
}

impl<'s> System<'s> for FireWaveSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, CombatRoot>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Wave>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Handle<Prefab<WorldPrefabData>>>,
        ReadStorage<'s, Named>,
        ReadStorage<'s, WeaponSlot>,
        Read<'s, EventChannel<FireWaveEvent>>,
        Write<'s, EventChannel<MissEvent>>,
        Write<'s, EventChannel<DmgPackage>>,
        ReadExpect<'s, ParentHierarchy>,
        PrefabLoader<'s, WorldPrefabData>,
        Read<'s, Time>,
    );

    /*
    fn setup(&mut self, world: &mut World) {
        self.wave = Some(world.exec(
            |loader: PrefabLoader<'_, WorldPrefabData>| {
                loader.load(
                    "object/projectile/wave/wave.ron",
                    RonFormat,
                    (),
                )
            },
        ));
    }
    */

    fn run(&mut self, (entities, roots, mut characters, mut waves, mut transforms, mut prefabs, names, weapon_slots, fire_wave_events, mut miss_events, mut dmg_events, hierarchy, prefab_loader, time): Self::SystemData) {

        if self.wave.is_none() {
            for root in roots.join() {
                if root.current_state == CombatState::Init {
                    let handle = prefab_loader.load(
                        "object/projectile/wave/wave.ron",
                        RonFormat,
                        (),
                    );
                    self.wave = Some(handle);
                }
            }
        }

        for (entity, mut wave, mut transform) in (&entities, &mut waves, &mut transforms).join() {
            wave.timer -= time.delta_seconds();
            transform.set_scale(transform.scale() + (Vector3::new(wave.rate, wave.rate, wave.rate) * time.delta_seconds()));
            if wave.timer <= 0.0 {
                entities.delete(entity);
                if let Some(dmg) = wave.dmg {
                    if let Some(source_ent) = wave.source {
                        for target_ent in wave.targets.iter() {
                            if let Ok(hit) = Character::check_hit(&characters, *target_ent, source_ent, dmg.accuracy) {
                                if hit {
                                    dmg_events.single_write(
                                        DmgPackage {
                                            target: *target_ent,
                                            power: dmg.power,
                                            element: dmg.element,
                                            source: Some(source_ent),
                                            status: dmg.effect,
                                        }
                                    );
                                } else {
                                    miss_events.single_write(
                                        MissEvent {
                                            target: *target_ent,
                                            source: Some(source_ent),
                                        }
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        for event in fire_wave_events.read(&mut self.reader) {
            let mut source_pos: Option<Vector3<f32>> = None;
            for (weapon_slot, transform, _) in (&weapon_slots, &transforms, hierarchy.all_children(event.source)).join() {
                // Get bullet source point.
                if weapon_slot.index() == 0 && source_pos.is_none() {
                    let source: Vector3<f32> = Vector4::from(transform.global_matrix().column(3)).xyz();
                    source_pos = Some(source);
                }
                if weapon_slot.index() == event.weapon_idx {
                    let source: Vector3<f32> = Vector4::from(transform.global_matrix().column(3)).xyz();
                    source_pos = Some(source);
                }
            }

            if let Some(source) = source_pos {
                let mut transform: Transform = Transform::from(source);
                let wave_ent: Entity = entities.create();
                transforms.insert(wave_ent, transform);
                prefabs.insert(wave_ent, self.wave.clone().unwrap());
                waves.insert(wave_ent, Wave {
                    source: Some(event.source),
                    targets: event.targets.clone(),
                    timer: event.time,
                    rate: 200.0,
                    dmg: event.dmg,
                });
            }
        }

    }
}