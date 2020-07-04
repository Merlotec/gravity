use amethyst::{
    assets::{
        Handle,
        Prefab,
        AssetPrefab,
        PrefabLoader,
        RonFormat,
    },
    gltf::{
        GltfSceneFormat,
        GltfSceneAsset,
        GltfPrefab,
    },
    core::{
        math::{
            Vector3,
            Vector4,
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
use crate::game::combat::ability::{DmgPackage, DmgTimer, Element, FireTorpedoEvent, MissEvent, StatusInflictDesc};
use crate::core::roll;
use crate::game::map::WorldPrefabData;
use crate::game::combat::{CombatRoot, CombatState};

pub const TORPEDO_SPEED: f32 = 40.0;

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(FireTorpedoSystemDesc))]
pub struct FireTorpedoSystem {
    torpedo_model: Option<Handle<Prefab<WorldPrefabData>>>,
    #[system_desc(event_channel_reader)]
    reader: ReaderId<FireTorpedoEvent>,
}

/// Set as none for single frame timer.
#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct Torpedo {
    hit: bool,
    source: Option<Entity>,
    target: Entity,
    speed: f32,
    power: f32,
    element: Element,
    effect: Option<StatusInflictDesc>
}

impl<'s> System<'s> for FireTorpedoSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, CombatRoot>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Flash>,
        WriteStorage<'s, Torpedo>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Handle<Prefab<WorldPrefabData>>>,
        ReadStorage<'s, Named>,
        ReadStorage<'s, WeaponSlot>,
        Read<'s, EventChannel<FireTorpedoEvent>>,
        Write<'s, EventChannel<MissEvent>>,
        Write<'s, EventChannel<DmgPackage>>,
        ReadExpect<'s, ParentHierarchy>,
        PrefabLoader<'s, WorldPrefabData>,
        Read<'s, Time>,
    );

    /*
    fn setup(&mut self, world: &mut World) {

        let handle = world.exec(
            |loader: PrefabLoader<'_, WorldPrefabData>| {
                loader.load(
                    "object/projectile/torpedo/torpedo.ron",
                    RonFormat,
                    (),
                )
            },
        );
        self.torpedo_model = Some(handle);
        println!("LOAD");
    }
*/

    fn run(&mut self, (entities, roots, mut characters, mut flashes, mut torpedoes, mut transforms, mut model_prefabs, names, weapon_slots, fire_bullet_events, mut miss_events, mut dmg_packages, hierarchy, prefab_loader, time): Self::SystemData) {
        if self.torpedo_model.is_none() {
            for root in roots.join() {
                if root.current_state == CombatState::Init {
                    let handle = prefab_loader.load(
                        "object/projectile/torpedo/torpedo.ron",
                        RonFormat,
                        (),
                    );
                    self.torpedo_model = Some(handle);
                }
            }
        }

        for (torpedo_ent, torpedo) in (&entities, &torpedoes).join() {
            if let Some(dest) = transforms.get(torpedo.target).cloned() {
                if let Some(trans) = transforms.get_mut(torpedo_ent) {
                    let mut delta: Vector3<f32> = Vector4::from(dest.global_matrix().column(3)).xyz() - *trans.translation();
                    let mut dir: Vector3<f32> = delta.normalize();
                    let abs_v: f32 = torpedo.speed * time.delta_seconds();
                    let mv: Vector3<f32> = dir * abs_v;
                    trans.prepend_translation(mv);
                    if abs_v >= delta.norm() {
                        // Torpedo has arrived.
                        if torpedo.hit {
                            dmg_packages.single_write(
                                DmgPackage {
                                    element: torpedo.element,
                                    power: torpedo.power,
                                    source: torpedo.source,
                                    target: torpedo.target,
                                    status: torpedo.effect,
                                }
                            );
                        } else {
                            miss_events.single_write(
                                MissEvent {
                                    target: torpedo.target,
                                    source: torpedo.source,
                                }
                            );
                        }
                        entities.delete(torpedo_ent);
                    }
                }
            }
        }

        for event in fire_bullet_events.read(&mut self.reader) {
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
                if let Some(source_pos) = source_pos {
                    let torpedo_ent: Entity = entities.create();
                    transforms.insert(torpedo_ent, Transform::from(source_pos));
                    torpedoes.insert(torpedo_ent, Torpedo {
                        source: Some(event.source),
                        target: event.target,
                        element: event.element,
                        power: event.power,
                        hit,
                        speed: TORPEDO_SPEED,
                        effect: event.effect,
                    });
                    model_prefabs.insert(torpedo_ent, self.torpedo_model.clone().unwrap());
                }
            }

        }
    }
}