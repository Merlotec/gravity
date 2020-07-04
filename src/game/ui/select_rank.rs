use amethyst::{
    assets::{
        AssetStorage,
        Handle,
        Loader,
    },
    core::{
        Parent,
        ParentHierarchy,
        Time,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    renderer::ImageFormat,
    renderer::Texture,
    shrev::{
        EventChannel,
        ReaderId,
    },
    ui::{
        Anchor,
        Interactable,
        UiEvent,
        UiEventType,
        UiImage,
        UiText,
        UiTransform,
    },
    window::ScreenDimensions,
};
use crate::core::{get_root, roll, get_root_mut};
use crate::game::ui::font::GameFonts;
use crate::game::character::{Character, CharacterId};
use crate::game::combat::{Rank};
use crate::game::ui::UiDisengageEvent;
use std::collections::HashMap;

pub const RANK_OPTION_SIZE: f32 = 150.0;
pub const RANK_CONTAINER_HEIGHT: f32 = RANK_OPTION_SIZE + 50.0;
pub const RANK_OPTION_SPACING: f32 = 50.0;
pub const RANK_ALERT_TIME: f32 = 1.0;

#[derive(Debug, Clone, PartialEq)]
pub struct RankSelectedEvent {
    pub owner: Entity,
    pub character_id: CharacterId,
    pub selection: Option<UiSelectRankOption>,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct UiSelectRankOption {
    pub charge: f32,
    pub rank: Rank,
}

#[derive(Debug, Clone, PartialEq, Component)]
pub struct UiSelectRankData {
    pub owner: Entity,
    pub character_id: CharacterId,
    pub options: Vec<UiSelectRankOption>,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct UiSelectRankRemoveTimer {
    remaining: f32,
}

pub type ShowSelectRankUiEvent = UiSelectRankData;

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(SelectRankUiSystemDesc))]
pub struct SelectRankUiSystem {
    pub icon_textures: HashMap<Rank, Handle<Texture>>,
    pub charge_texture: Option<Handle<Texture>>,
    #[system_desc(event_channel_reader)]
    rank_event_reader: ReaderId<ShowSelectRankUiEvent>,
    #[system_desc(event_channel_reader)]
    ui_event_reader: ReaderId<UiEvent>,
    #[system_desc(event_channel_reader)]
    disengage_event_reader: ReaderId<UiDisengageEvent>,
}

impl<'s> System<'s> for SelectRankUiSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Parent>,
        WriteStorage<'s, UiTransform>,
        ReadStorage<'s, Character>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, Interactable>,
        WriteStorage<'s, UiSelectRankOption>,
        WriteStorage<'s, UiSelectRankData>,
        WriteStorage<'s, UiSelectRankRemoveTimer>,
        ReadExpect<'s, ParentHierarchy>,
        ReadExpect<'s, GameFonts>,
        Read<'s, Time>,
        Read<'s, EventChannel<ShowSelectRankUiEvent>>,
        Read<'s, EventChannel<UiEvent>>,
        Write<'s, EventChannel<RankSelectedEvent>>,
        Write<'s, EventChannel<UiDisengageEvent>>,
    );

    fn setup(&mut self, world: &mut World) {
        if !world.has_value::<AssetStorage::<Texture>>() {
            world.insert(AssetStorage::<Texture>::new());
        }
        let loader = world.read_resource::<Loader>();
        self.icon_textures.insert(Rank::Basic, loader.load(
            "ui/rank_basic.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
        self.icon_textures.insert(Rank::Advanced, loader.load(
            "ui/rank_advanced.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
        self.icon_textures.insert(Rank::Elite, loader.load(
            "ui/rank_elite.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
        self.charge_texture = Some(loader.load(
            "ui/charge_icon.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
    }

    fn run(&mut self, (entities, mut parents, mut transforms, characters, mut texts, mut images, mut interactables, mut rank_options, mut rank_bases, mut rank_removes, hierarchy, fonts, time, rank_events, ui_events, mut rank_selected_events, disengage_events): Self::SystemData) {
        for event in rank_events.read(&mut self.rank_event_reader) {
            let base_ent: Entity = entities.create();

            let total_width: f32 = (RANK_OPTION_SIZE * event.options.len() as f32) + (RANK_OPTION_SPACING * (event.options.len() as f32 - 1.0));

            let id: String = String::from("rank_base:") + &base_ent.id().to_string();
            let mut transform: UiTransform = UiTransform::new(
                id,
                Anchor::Middle,
                Anchor::Middle,
                0.0, 0.0, 1.0,
                total_width, RANK_CONTAINER_HEIGHT,
            );
            transform.opaque = false;
            transforms.insert(base_ent, transform);
            images.insert(base_ent, UiImage::SolidColor([0.005, 0.005, 0.006, 0.9]));
            rank_bases.insert(base_ent, event.clone());

            let half_width: f32 = total_width / 2.0;
            for (i, option) in event.options.iter().enumerate() {
                let invokable: bool = {
                    if let Some((_, character_ent)) = get_root::<Character, _,_>(&parents, &characters, event.owner) {
                        if Character::can_take_turn(&characters, character_ent, option.charge) {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                let text_color: [f32; 4] = {
                    if invokable {
                        [1.0; 4]
                    } else {
                        [0.4, 0.1, 0.1, 1.0]
                    }
                };

                let opt_ent: Entity = entities.create();

                let current_offset: f32 = (RANK_OPTION_SIZE * i as f32) + (RANK_OPTION_SPACING * (i as f32));
                let x = current_offset - half_width;
                let id: String = String::from("rank_option:") + &opt_ent.id().to_string();
                let mut transform: UiTransform = UiTransform::new(
                    id,
                    Anchor::Middle,
                    Anchor::MiddleLeft,
                    x, 0.0, 1.0,
                    RANK_OPTION_SIZE, RANK_OPTION_SIZE,
                );
                transforms.insert(opt_ent, transform);
                images.insert(opt_ent, UiImage::SolidColor([0.0, 0.0, 0.0, 0.0]));
                rank_options.insert(opt_ent, *option);

                if invokable {
                    interactables.insert(opt_ent, Interactable::default());
                }

                let title_ent: Entity = entities.create();
                let id: String = String::from("rank_option_title:") + &title_ent.id().to_string();
                let mut transform: UiTransform = UiTransform::new(
                    id,
                    Anchor::TopMiddle,
                    Anchor::TopMiddle,
                    0.0, 0.0, 1.0,
                    RANK_OPTION_SIZE * 0.8, 35.0,
                );
                transform.opaque = false;
                texts.insert(title_ent, UiText::new(fonts.ability().clone(), option.rank.to_string(), text_color, 20.0));
                parents.insert(title_ent, Parent { entity: opt_ent });
                transforms.insert(title_ent, transform);

                let img_ent: Entity = entities.create();
                let id: String = String::from("rank_image:") + &opt_ent.id().to_string();
                let mut transform: UiTransform = UiTransform::new(
                    id,
                    Anchor::Middle,
                    Anchor::Middle,
                    0.0, 0.0, 1.0,
                    RANK_OPTION_SIZE * 0.65, RANK_OPTION_SIZE * 0.65,
                );
                transform.opaque = false;
                transforms.insert(img_ent, transform);
                if let Some(rank_texture) = self.icon_textures.get(&option.rank).cloned() {
                    images.insert(img_ent, UiImage::Texture(rank_texture));
                }

                // Charge Icon
                let charge_ent: Entity = entities.create();
                let charge_icon_img: UiImage = UiImage::Texture(self.charge_texture.clone().unwrap());
                images.insert(charge_ent, charge_icon_img);
                let id: String = String::from("rank_charge_icon:") + &charge_ent.id().to_string();
                let mut transform: UiTransform = UiTransform::new(
                    id,
                    Anchor::Middle,
                    Anchor::Middle,
                    -15.0, -(RANK_OPTION_SIZE * 0.4), 1.5,
                    25.0, 25.0,
                );
                transform.opaque = false;
                transforms.insert(charge_ent, transform);
                parents.insert(charge_ent, Parent { entity: opt_ent });

                // Charge Text
                let charge_text_ent: Entity = entities.create();
                texts.insert(charge_text_ent, UiText::new(fonts.ability().clone(), format!("{:.0}", option.charge), text_color, 15.0));
                let id: String = String::from("rank_charge_text:") + &charge_text_ent.id().to_string();
                let mut transform: UiTransform = UiTransform::new(
                    id,
                    Anchor::Middle,
                    Anchor::Middle,
                    25.0, 0.0, 1.5,
                    RANK_OPTION_SIZE, RANK_OPTION_SIZE * 0.4,
                );
                transform.opaque = false;
                transforms.insert(charge_text_ent, transform);
                parents.insert(charge_text_ent, Parent { entity: charge_ent });

                parents.insert(img_ent, Parent { entity: opt_ent });
                parents.insert(opt_ent, Parent { entity: base_ent });
            }

        }
        for ui_event in ui_events.read(&mut self.ui_event_reader) {
            if ui_event.event_type == UiEventType::Click {
                if let Some(rank_option) = rank_options.get(ui_event.target) {
                    // Initiate ability target selection.
                    if let Some((rank_data, base_ent)) = get_root_mut::<UiSelectRankData, _, _>(&parents, &mut rank_bases, ui_event.target) {
                        rank_selected_events.single_write(
                            RankSelectedEvent {
                                selection: Some(*rank_option),
                                character_id: rank_data.character_id,
                                owner: rank_data.owner,
                            }
                        );
                        entities.delete(base_ent);
                    }

                }
            } else if ui_event.event_type == UiEventType::HoverStart {
                if let Some(rank_option) = rank_options.get_mut(ui_event.target) {
                    if let Some(ui_image) = images.get_mut(ui_event.target) {
                        *ui_image = UiImage::SolidColor([0.2, 0.05, 0.05, 1.0]);
                    }
                }
            } else if ui_event.event_type == UiEventType::HoverStop {
                if let Some(rank_option) = rank_options.get_mut(ui_event.target) {
                    if let Some(ui_image) = images.get_mut(ui_event.target) {
                        *ui_image = UiImage::SolidColor([0.0, 0.0, 0.0, 0.0]);
                    }
                }
            }
        }
        for (entity, mut remove, base) in (&entities, &mut rank_removes, &rank_bases).join() {
            remove.remaining -= time.delta_seconds();
            if remove.remaining < 0.0 {
                entities.delete(entity);
            }
        }
        for event in disengage_events.read(&mut self.disengage_event_reader) {
            for (entity, rank_base) in (&entities, &rank_bases).join() {
                rank_selected_events.single_write(
                    RankSelectedEvent {
                        owner: rank_base.owner,
                        selection: None,
                        character_id: rank_base.character_id,
                    }
                );
                entities.delete(entity);
            }

        }
    }

}