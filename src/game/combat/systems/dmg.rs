use amethyst::{
    core::{
        math::Vector2,
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
    ui::UiTransform,
};
use combat_render::flash::Flash;

use crate::game::character::{Character, LastDamaged, WeaponSlot};
use crate::game::combat::ability::{DmgPackage, DmgTimer, MissEvent};
use crate::game::ui::hud::UiBase;
use crate::game::ui::marker::ShowUiMarkerEvent;

pub const DMG_MARKER_ANIM_TIME: f32 = 0.7;

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(DmgSystemDesc))]
pub struct DmgSystem {
    #[system_desc(event_channel_reader)]
    dmg_package_event_reader: ReaderId<DmgPackage>,

    #[system_desc(event_channel_reader)]
    miss_event_reader: ReaderId<MissEvent>,
}

impl<'s> System<'s> for DmgSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, DmgTimer>,
        WriteStorage<'s, UiBase>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, LastDamaged>,
        Write<'s, EventChannel<DmgPackage>>,
        Read<'s, EventChannel<MissEvent>>,
        Write<'s, EventChannel<ShowUiMarkerEvent>>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, mut characters, mut dmg_timers, ui_bases, ui_transforms, mut last_damaged, mut dmg_events, miss_events, mut show_marker_events, time): Self::SystemData) {
        for (entity, mut dmg_timer) in (&entities, &mut dmg_timers).join() {
            if dmg_timer.timer <= 0.0 {
                dmg_events.single_write(
                    dmg_timer.package
                );
                entities.delete(entity);
            }
            dmg_timer.timer -= time.delta_seconds();
        }

        for event in dmg_events.read(&mut self.dmg_package_event_reader) {
            // Inflict damage on character.
            if let Some(character) = characters.get_mut(event.target) {
                if let Ok(received) = Character::inflict_dmg_silent(&mut characters, event.source, event.target, event.power, event.element) {
                    if let Some(ui_base) = ui_bases.get(event.target) {
                        if let Some(ui_transform) = ui_transforms.get(ui_base.entity()) {
                            show_marker_events.single_write(
                                ShowUiMarkerEvent {
                                    owner: None,
                                    position: Vector2::new(ui_transform.local_x, ui_transform.local_y),
                                    text: format!("{:.1}", received),
                                    text_color: [1.0, 0.0, 0.0, 1.0],
                                    anim_vel: Some(Vector2::new(0.0, 100.0)),
                                    anim_time: DMG_MARKER_ANIM_TIME,
                                    fade: true,
                                    character: Some(event.target),
                                }
                            );
                            last_damaged.insert(event.target, LastDamaged {
                                entity: event.source,
                                dmg: received,
                            });
                        }
                    }
                    if let Some(status) = event.status {
                        Character::inflict_status_silent(&mut characters, event.target, status);
                    }
                }
            }
        }

        for event in miss_events.read(&mut self.miss_event_reader) {
            if let Some(ui_base) = ui_bases.get(event.target) {
                if let Some(ui_transform) = ui_transforms.get(ui_base.entity()) {
                    show_marker_events.single_write(
                        ShowUiMarkerEvent {
                            character: Some(event.target),
                            owner: None,
                            position: Vector2::new(ui_transform.local_x, ui_transform.local_y),
                            text: "Miss".to_string(),
                            text_color: [1.0, 1.0, 0.5, 1.0],
                            anim_vel: Some(Vector2::new(0.0, 100.0)),
                            anim_time: DMG_MARKER_ANIM_TIME,
                            fade: true,
                        }
                    );
                }
            }
        }
    }
}