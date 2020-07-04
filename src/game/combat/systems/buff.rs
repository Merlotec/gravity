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
use crate::game::combat::ability::{ChargeEvent, DmgPackage, DmgTimer, HealEvent, MissEvent};
use crate::game::ui::hud::UiBase;
use crate::game::ui::marker::ShowUiMarkerEvent;

pub const BUFF_MARKER_ANIM_TIME: f32 = 0.7;

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(BuffSystemDesc))]
pub struct BuffSystem {
    #[system_desc(event_channel_reader)]
    heal_event_reader: ReaderId<HealEvent>,

    #[system_desc(event_channel_reader)]
    charge_event_reader: ReaderId<ChargeEvent>,
}

impl<'s> System<'s> for BuffSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, UiBase>,
        WriteStorage<'s, UiTransform>,
        Write<'s, EventChannel<HealEvent>>,
        Write<'s, EventChannel<ChargeEvent>>,
        Write<'s, EventChannel<ShowUiMarkerEvent>>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, mut characters, ui_bases, ui_transforms, mut dmg_events, charge_events, mut show_marker_events, time): Self::SystemData) {
        for event in dmg_events.read(&mut self.heal_event_reader) {
            // Inflict damage on character.
            if let Some(character) = characters.get_mut(event.target) {
                let delta_health = character.change_health(event.heal_value);
                if let Some(ui_base) = ui_bases.get(event.target) {
                    if let Some(ui_transform) = ui_transforms.get(ui_base.entity()) {
                        show_marker_events.single_write(
                            ShowUiMarkerEvent {
                                owner: event.owner,
                                position: Vector2::new(ui_transform.local_x, ui_transform.local_y),
                                text: format!("{:.1}", delta_health),
                                text_color: [0.0, 1.0, 0.0, 1.0],
                                anim_vel: Some(Vector2::new(0.0, 100.0)),
                                anim_time: BUFF_MARKER_ANIM_TIME,
                                fade: true,
                                character: Some(event.target),
                            }
                        );
                    }
                }
            }
        }

        for event in charge_events.read(&mut self.charge_event_reader) {
            // Inflict damage on character.
            if let Some(character) = characters.get_mut(event.target) {
                let delta_charge = character.add_charge(event.charge_value);
                if let Some(ui_base) = ui_bases.get(event.target) {
                    if let Some(ui_transform) = ui_transforms.get(ui_base.entity()) {
                        show_marker_events.single_write(
                            ShowUiMarkerEvent {
                                position: Vector2::new(ui_transform.local_x, ui_transform.local_y),
                                text: format!("{:.1}", delta_charge),
                                text_color: [0.0, 1.0, 1.0, 1.0],
                                anim_vel: Some(Vector2::new(0.0, 100.0)),
                                anim_time: BUFF_MARKER_ANIM_TIME,
                                fade: true,
                                owner: event.owner,
                                character: Some(event.target),
                            }
                        );
                    }
                }
            }
        }
    }
}