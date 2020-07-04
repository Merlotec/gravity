use amethyst::{
    core::{
        math::Vector2,
        Parent,
        SystemDesc,
        Time,
    },
    ecs::prelude::*,
    shrev::{
        EventChannel,
        ReaderId,
    },
    ui::{
        Anchor,
        UiText,
        UiTransform,
    },
};

use crate::game::ui::font::GameFonts;
use crate::game::ui::hud::UiBase;

#[derive(Debug, Clone, Component)]
pub struct UiMarkerText {
    pub text: String,
    pub anim_time: f32,
    pub current_time: f32,
    pub anim_vel: Option<Vector2<f32>>,
    pub fade: bool,
    pub owner: Option<Entity>,
    pub character: Option<Entity>,
}

#[derive(Debug, Clone)]
pub struct ShowUiMarkerEvent {
    pub position: Vector2<f32>,
    pub text: String,
    pub anim_time: f32,
    pub anim_vel: Option<Vector2<f32>>,
    pub fade: bool,
    pub text_color: [f32; 4],
    pub owner: Option<Entity>,
    pub character: Option<Entity>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct MarkerUiCompletedEvent {
    pub owner: Option<Entity>,
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(MarkerUiSystemDesc))]
pub struct MarkerUiSystem {
    #[system_desc(event_channel_reader)]
    marker_event_reader: ReaderId<ShowUiMarkerEvent>,
}

impl<'s> System<'s> for MarkerUiSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Parent>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, UiText>,
        ReadStorage<'s, UiBase>,
        WriteStorage<'s, UiMarkerText>,
        Read<'s, EventChannel<ShowUiMarkerEvent>>,
        Write<'s, EventChannel<MarkerUiCompletedEvent>>,
        Read<'s, Time>,
        ReadExpect<'s, GameFonts>,
    );

    fn run(&mut self, (entities, mut parents, mut ui_transforms, mut ui_texts, ui_bases, mut ui_marker_texts, marker_events, mut completed_events, time, fonts): Self::SystemData) {
        for event in marker_events.read(&mut self.marker_event_reader) {
            let entity = entities.create();

            let mut current_count: usize = 0;
            for (transforms, marker_texts) in (&ui_transforms, &ui_marker_texts).join() {
                if marker_texts.character.is_some() && marker_texts.character == event.character {
                    current_count += 1;
                }
            }

            let mut slot = current_count % 3;

            let pos: Vector2<f32> = match slot {
                0 => event.position,
                1 => event.position + Vector2::new(50.0, 0.0),
                2 => event.position + Vector2::new(50.0, 0.0),
                _ => unreachable!(),
            };

            let marker: UiMarkerText = UiMarkerText {
                anim_time: event.anim_time,
                anim_vel: event.anim_vel,
                text: event.text.clone(),
                current_time: 0.0,
                fade: event.fade,
                owner: event.owner,
                character: event.character,
            };
            ui_marker_texts.insert(entity, marker);

            let text: UiText = UiText::new(fonts.ability().clone(), event.text.to_string(), event.text_color, 25.0);
            ui_texts.insert(entity, text);

            let id: String = String::from("marker:") + &entity.id().to_string();
            let mut trans = UiTransform::new(
                id,
                Anchor::BottomLeft,
                Anchor::Middle,
                pos.x, pos.y, 0.0,
                500.0, 30.0,
            );
            trans.opaque = false;
            ui_transforms.insert(entity, trans);
        }

        for (entity, mut ui_transform, mut ui_text, mut ui_marker_text) in (&entities, &mut ui_transforms, &mut ui_texts, &mut ui_marker_texts).join() {
            if ui_marker_text.current_time >= ui_marker_text.anim_time {
                completed_events.single_write(
                    MarkerUiCompletedEvent {
                        owner: ui_marker_text.owner,
                    }
                );
                entities.delete(entity);
            } else {
                ui_marker_text.current_time += time.delta_seconds();

                if let Some(anim_vel) = ui_marker_text.anim_vel {
                    let current: Vector2<f32> = Vector2::new(ui_transform.local_x, ui_transform.local_y);
                    let new_trans: Vector2<f32> = current + (anim_vel * time.delta_seconds());
                    ui_transform.local_x = new_trans.x;
                    ui_transform.local_y = new_trans.y;
                }
                if ui_marker_text.fade {
                    let alpha: f32 = 1.0 - ui_marker_text.current_time / ui_marker_text.anim_time;
                    ui_text.color[3] = alpha;
                }
            }
        }
    }
}