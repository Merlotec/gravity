use amethyst::{
    core::{
        math::{
            Unit,
            Vector2,
            Vector3,
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
use rand::Rng;

use crate::core::get_root_mut;
use crate::game::character::{Character, CharacterDefeatedEvent, Defeated, LastDamaged};
use crate::game::combat::spawn::{SlotManager, Slots};
use crate::game::ui::hud::UiBase;

pub const DRIFT_TIME: f32 = 20.0;
pub const SPLASH_PROPORTION: f32 = 0.3;

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(CharacterDefeatSystemDesc))]
pub struct CharacterDefeatSystem {
    #[system_desc(event_channel_reader)]
    reader: ReaderId<CharacterDefeatedEvent>,
}

impl<'s> System<'s> for CharacterDefeatSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Parent>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Defeated>,
        ReadStorage<'s, LastDamaged>,
        WriteStorage<'s, UiBase>,
        WriteStorage<'s, SlotManager>,
        Write<'s, EventChannel<CharacterDefeatedEvent>>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, parents, mut characters, mut transforms, mut defeated, last_damaged, mut ui_bases, mut slot_managers, mut character_defeated_events, time): Self::SystemData) {
        for (entity, character) in (&entities, &characters).join() {
            if character.relative_health() <= 0.0 {
                let killer: Option<Entity> = {
                    if let Some(last_damaged) = last_damaged.get(entity) {
                        last_damaged.entity
                    } else {
                        None
                    }
                };
                character_defeated_events.single_write(
                    CharacterDefeatedEvent {
                        character_ent: entity,
                        splash_dmg: character.max_health() * SPLASH_PROPORTION,
                        killer,
                    }
                );
            }
        }

        for event in character_defeated_events.read(&mut self.reader) {
            characters.remove(event.character_ent);
            let mut rng = rand::thread_rng();
            // TODO: Perhaps chance of crashing if all 3 values of normalized vector are 0.
            defeated.insert(event.character_ent, Defeated {
                drift: Vector3::new(rng.gen_range(-0.5, 0.5), -2.0 + rng.gen_range(-0.7, 0.7), rng.gen_range(-0.5, 0.5)),
                rotation_axis: Unit::new_unchecked(Vector3::new(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)).normalize()),
                rotation_speed: 1.0,
                time: 0.0,
            });
            if let Some(ui_base) = ui_bases.get(event.character_ent) {
                entities.delete(ui_base.entity());
            }
            ui_bases.remove(event.character_ent);
            if let Some((slot_manager, _)) = get_root_mut::<SlotManager, _, _>(&parents, &mut slot_managers, event.character_ent) {
                slot_manager.remove_entity(event.character_ent);
            }
        }

        for (entity, mut transforms, mut defeated) in (&entities, &mut transforms, &mut defeated).join() {
            transforms.prepend_translation(defeated.drift * time.delta_seconds());
            transforms.prepend_rotation(defeated.rotation_axis, defeated.rotation_speed * time.delta_seconds());
            defeated.time += time.delta_seconds();
            if defeated.time >= DRIFT_TIME {
                entities.delete(entity);
            }
        }
    }
}