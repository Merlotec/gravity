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
use crate::game::combat::ability::{DmgPackage, DmgTimer, Element, FireLaserEvent, MissEvent, StatusInflictDesc};
use crate::game::map::WorldPrefabData;
use crate::game::combat::{CombatRoot, CombatState};

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct Laser {
    pub timer: f32,
    pub target: Entity,
    pub source: Option<Entity>,
    pub power: f32,
    pub element: Element,
    pub effect: Option<StatusInflictDesc>,
    pub hit: bool,
}

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(FireLaserSystemDesc))]
pub struct FireLaserSystem {
    laser: Option<Handle<Prefab<WorldPrefabData>>>,
    #[system_desc(event_channel_reader)]
    reader: ReaderId<FireLaserEvent>,
}

/// Set as none for single frame timer.
#[derive(Debug, Copy, Clone, Default, PartialEq, Component)]
pub struct FlashTimer(Option<f32>);

impl<'s> System<'s> for FireLaserSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, CombatRoot>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Laser>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Handle<Prefab<WorldPrefabData>>>,
        ReadStorage<'s, Named>,
        ReadStorage<'s, WeaponSlot>,
        Read<'s, EventChannel<FireLaserEvent>>,
        Write<'s, EventChannel<MissEvent>>,
        Write<'s, EventChannel<DmgPackage>>,
        ReadExpect<'s, ParentHierarchy>,
        PrefabLoader<'s, WorldPrefabData>,
        Read<'s, Time>,
    );

    /*
    fn setup(&mut self, world: &mut World) {
        self.laser = Some(world.exec(
            |loader: PrefabLoader<'_, WorldPrefabData>| {
                loader.load(
                    "object/projectile/laser/laser.ron",
                    RonFormat,
                    (),
                )
            },
        ));
    }
*/

    fn run(&mut self, (entities, roots, mut characters, mut lasers, mut transforms, mut prefabs, names, weapon_slots, fire_laser_events, mut miss_events, mut dmg_events, hierarchy, prefab_loader, time): Self::SystemData) {
        if self.laser.is_none() {
            for root in roots.join() {
                if root.current_state == CombatState::Init {
                    let handle = prefab_loader.load(
                        "object/projectile/laser/laser.ron",
                        RonFormat,
                        (),
                    );
                    self.laser = Some(handle);
                }
            }
        }

        for (entity, mut laser) in (&entities, &mut lasers).join() {
            laser.timer -= time.delta_seconds();
            if laser.timer < 0.0 {
                entities.delete(entity);
                if laser.hit {
                    dmg_events.single_write(
                        DmgPackage {
                            target: laser.target,
                            power: laser.power,
                            element: laser.element,
                            source: laser.source,
                            status: laser.effect,
                        }
                    );
                } else {
                    miss_events.single_write(
                        MissEvent {
                            target: laser.target,
                            source: laser.source,
                        }
                    );
                }

            }
        }

        for event in fire_laser_events.read(&mut self.reader) {
            if let Ok(hit) = Character::check_hit(&characters, event.target, event.source, event.accuracy) {
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
                    if let Some(target_trans) = transforms.get(event.target).cloned() {
                        let pos: Vector3<f32> = Vector4::from(target_trans.global_matrix().column(3)).xyz();
                        let delta: Vector3<f32> = pos - source;
                        let dir = delta.normalize();
                        let abs: f32 = delta.norm();
                        let forward: Vector3<f32> = Vector3::new(0.0, 0.0, -1.0).normalize();
                        let cross: Vector3<f32> = forward.cross(&dir);
                        let quat: UnitQuaternion<f32> = UnitQuaternion::new_normalize(Quaternion::new(1.0 + dir.dot(&forward), cross.x, cross.y, cross.z));

                        let mut transform: Transform = Transform::from(source);
                        transform.set_rotation(quat);
                        transform.set_scale(Vector3::new(1.0, 1.0, -abs));

                        let laser_ent: Entity = entities.create();
                        transforms.insert(laser_ent, transform);
                        prefabs.insert(laser_ent, self.laser.clone().unwrap());
                        lasers.insert(laser_ent, Laser {
                            source: Some(event.source),
                            target: event.target,
                            power: event.power,
                            element: event.element,
                            effect: event.effect,
                            timer: event.time,
                            hit,
                        });
                    }
                }
            }
        }
    }
}