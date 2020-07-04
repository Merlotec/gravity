use amethyst::{
    assets::{
        AssetStorage,
        Loader,
    },
    audio::{
        Mp3Format,
        output::Output,
        Source,
        SourceHandle,
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
use crate::game::combat::ability::{DmgPackage, DmgTimer, Element, FireBulletEvent, MissEvent};

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(FireBulletSystemDesc))]
pub struct FireBulletSystem {
    sound_handle: Option<SourceHandle>,
    #[system_desc(event_channel_reader)]
    reader: ReaderId<FireBulletEvent>,
}

/// Set as none for single frame timer.
#[derive(Debug, Copy, Clone, Default, PartialEq, Component)]
pub struct FlashTimer(Option<f32>);

impl<'s> System<'s> for FireBulletSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Flash>,
        WriteStorage<'s, FlashTimer>,
        WriteStorage<'s, DmgTimer>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Named>,
        ReadStorage<'s, WeaponSlot>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Loader>,
        Option<Read<'s, Output>>,
        Read<'s, EventChannel<FireBulletEvent>>,
        Write<'s, EventChannel<MissEvent>>,
        ReadExpect<'s, ParentHierarchy>,
        Read<'s, Time>,
    );

    fn setup(&mut self, world: &mut World) {
        self.sound_handle = Some(world.read_resource::<Loader>().load("music/gunshot.mp3", Mp3Format, (), &world.read_resource()));
    }

    fn run(&mut self, (entities, mut characters, mut flashes, mut flash_timers, mut dmg_timers, mut transforms, names, weapon_slots, audio_assets, loader, output, fire_bullet_events, mut miss_events, hierarchy, time): Self::SystemData) {
        for (entity, flash, mut flash_timer) in (&entities, &flashes, &mut flash_timers).join() {
            if let Some(mut t) = flash_timer.0 {
                t -= time.delta_seconds();
                if t < 0.0 {
                    flash_timer.0 = None;
                } else {
                    flash_timer.0 = Some(t);
                }
            }
            if let None = flash_timer.0 {
                entities.delete(entity);
            }
        }

        for event in fire_bullet_events.read(&mut self.reader) {
            if let Ok(hit) = Character::check_hit(&characters, event.target, event.source, event.accuracy) {
                if hit {
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
                        let flash_ent: Entity = entities.create();
                        flashes.insert(flash_ent, Flash::default());
                        let mut trans: Transform = Transform::from(source);
                        trans.set_scale(Vector3::new(80.0, 80.0, 80.0));
                        transforms.insert(flash_ent, trans);
                        flash_timers.insert(flash_ent, FlashTimer(None));

                        // Play sound
                        if let Some(ref handle) = self.sound_handle {
                            if let Some(ref output) = output {
                                if let Some(sound) = audio_assets.get(&handle) {
                                    output.play_once(sound, 0.7);
                                }
                            }
                        }

                        let dmg_ent: Entity = entities.create();
                        dmg_timers.insert(dmg_ent, DmgTimer {
                            package: DmgPackage {
                                source: Some(event.source),
                                target: event.target,
                                power: event.power,
                                element: Element::Kinetic,
                                status: event.effect,
                            },
                            timer: 0.0,
                        });
                    }
                } else {
                    // Play sound
                    if let Some(ref handle) = self.sound_handle {
                        if let Some(ref output) = output {
                            if let Some(sound) = audio_assets.get(&handle) {
                                output.play_once(sound, 0.7);
                            }
                        }
                    }

                    miss_events.single_write(
                        MissEvent {
                            target: event.target,
                            source: Some(event.source),
                        }
                    );
                }
            }
        }
    }
}