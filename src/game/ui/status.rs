use amethyst::{
    assets::{
        AssetStorage,
        Loader,
        Handle,
    },
    core::{
        math::{
            Vector2,
            Vector3,
            Vector4,
        },
        Parent,
        Transform,
        ParentHierarchy,
    },
    renderer::{
        ImageFormat,
        Texture,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    renderer::{
        ActiveCamera,
        Camera,
    },
    ui::{
        Anchor,
        FontAsset,
        FontHandle,
        TtfFormat,
        UiImage,
        UiText,
        UiTransform,
    },
    window::ScreenDimensions,
    shrev::{
        EventChannel,
        ReaderId,
    }
};

use crate::core::get_root;
use crate::game::character::{
    Character,
};
use crate::game::combat::status::StatusType;
use crate::game::combat::{Team, Rank};
use crate::game::ui::font::GameFonts;
use crate::game::ui::hud::UiCharacterBase;
use std::collections::HashMap;
use crate::game::combat::ability::Performing;

pub const STATUS_WIDTH: f32 = 200.0;
pub const STATUS_HEIGHT: f32 = 30.0;
pub const STATUS_PADDING: f32 = 40.0;
pub const STATUS_BAR_HEIGHT: f32 = 5.0;
pub const HEALTH_BAR_COLOR: [f32; 4] = [1.0, 0.1, 0.1, 0.5];
pub const CHARGE_BAR_COLOR: [f32; 4] = [0.1, 0.1, 1.0, 0.5];
pub const RANK_ICON_SIZE: f32 = 40.0;
pub const STATUS_ICON_SIZE: f32 = 35.0;

#[derive(Debug, Clone, PartialEq)]
pub struct UiStatus {
    pub offset: Vector2<f32>,
    pub name: String,
    pub team: Team,
    pub character_ent: Entity,
    pub show_health: bool,
    pub show_charge: bool,
    pub show_status_effects: bool,
}

impl UiStatus {
    pub(crate) fn new(character_ent: Entity, name: String, team: Team) -> Self {
        Self {
            offset: Vector2::new(0.0, 100.0),
            name,
            team,
            character_ent,
            show_health: true,
            show_charge: true,
            show_status_effects: true,
        }
    }
}

impl Component for UiStatus {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Copy, Clone, new, Component)]
pub struct UiHealthBar {
    character_ent: Entity,
}

#[derive(Debug, Copy, Clone, new, Component)]
pub struct UiChargeBar {
    character_ent: Entity,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct UiStatusIcon {
    pub ty: StatusType,
    pub count: usize,
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(StatusUiSystemDesc))]
pub struct StatusUiSystem {
    pub icon_textures: HashMap<Rank, Handle<Texture>>,
    pub status_textures: HashMap<StatusType, Handle<Texture>>,
}

impl<'s> System<'s> for StatusUiSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Parent>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiStatus>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, UiHealthBar>,
        WriteStorage<'s, UiChargeBar>,
        WriteStorage<'s, UiCharacterBase>,
        WriteStorage<'s, UiStatusIcon>,
        ReadStorage<'s, Character>,
        ReadStorage<'s, Performing>,
        ReadExpect<'s, GameFonts>,
        ReadExpect<'s, ScreenDimensions>,
        ReadExpect<'s, ParentHierarchy>,

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
        self.icon_textures.insert(Rank::Legendary, loader.load(
            "ui/rank_legendary.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));

        self.status_textures.insert(StatusType::Scramble, loader.load(
            "ui/scramble.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
        self.status_textures.insert(StatusType::Overclocked, loader.load(
            "ui/overclocked.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
        self.status_textures.insert(StatusType::Unstable, loader.load(
            "ui/unstable.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
        self.status_textures.insert(StatusType::Defend, loader.load(
            "ui/defend.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
        self.status_textures.insert(StatusType::Empower, loader.load(
            "ui/empower.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
        self.status_textures.insert(StatusType::Focus, loader.load(
            "ui/focus.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));


    }

    fn run(&mut self, (entities, mut parents, mut texts, mut images, mut ui_statuses, mut ui_transforms, mut health_bars, mut charge_bars, ui_character_bases, mut status_icons, characters, performings, fonts, dims, hierarchy): Self::SystemData) {
        for (entity, status, mut ui_transform, mut text) in (&entities, &ui_statuses, &mut ui_transforms, &mut texts).join() {
            if let Some((character_base, _)) = get_root::<UiCharacterBase, _, _>(&parents, &ui_character_bases, entity) {
                ui_transform.local_y = (super::crosshair::CROSSHAIR_BASE_FACTOR * dims.height() * character_base.z_factor) / 2.0 + STATUS_PADDING;
            }

            if performings.contains(status.character_ent) {
                let text_color: [f32; 4] = {
                    match status.team {
                        Team::Friendly => [0.0, 1.0, 0.0, 1.0],
                        Team::Enemy => [1.0, 0.0, 0.0, 1.0],
                    }
                };
                text.color = text_color;
            } else {
                let text_color: [f32; 4] = {
                    match status.team {
                        Team::Friendly => [0.7, 1.0, 0.7, 1.0],
                        Team::Enemy => [1.0, 0.7, 0.7, 1.0],
                    }
                };
                text.color = text_color;
            }
        }

        let texts_mask = texts.mask().clone();
        let transform_mask = ui_transforms.mask().clone();
        for (entity, status, _, _) in (&entities, &ui_statuses, !transform_mask, !texts_mask).join() {
            if let Some(character) = characters.get(status.character_ent) {
                let text_color: [f32; 4] = {
                    match status.team {
                        Team::Friendly => [0.7, 1.0, 0.7, 1.0],
                        Team::Enemy => [1.0, 0.7, 0.7, 1.0],
                    }
                };

                let name: String = {
                    if character.rank() != Rank::Basic && character.rank() != Rank::Legendary {
                        character.rank().to_string() + " " + &status.name.clone()
                    } else {
                        status.name.clone()
                    }
                };

                texts.insert(entity, UiText::new(
                    fonts.status().clone(),
                    name,
                    text_color,
                    15.0,
                ));
                images.insert(entity, UiImage::SolidColor([0.005, 0.005, 0.006, 0.8]));
                let id: String = String::from("status:") + &status.character_ent.id().to_string();
                let mut trans = UiTransform::new(
                    id,
                    Anchor::BottomMiddle,
                    Anchor::Middle,
                    0.0, STATUS_PADDING, 0.0,
                    STATUS_WIDTH, STATUS_HEIGHT,
                );
                trans.opaque = false;
                ui_transforms.insert(entity, trans);

                // Rank
                if character.rank() != Rank::Basic {
                    if let Some(rank_texture) = self.icon_textures.get(&character.rank()).cloned() {
                        let rank_ent: Entity = entities.create();
                        images.insert(rank_ent, UiImage::Texture(rank_texture));
                        let id: String = String::from("rank_icon:") + &status.character_ent.id().to_string();
                        let mut trans = UiTransform::new(
                            id,
                            Anchor::BottomMiddle,
                            Anchor::BottomMiddle,
                            0.0, STATUS_HEIGHT, 0.0,
                            RANK_ICON_SIZE, RANK_ICON_SIZE,
                        );
                        trans.opaque = false;
                        ui_transforms.insert(rank_ent, trans);
                        parents.insert(rank_ent, Parent { entity });
                    }
                }

                if status.show_health {
                    let health_ent: Entity = entities.create();
                    images.insert(health_ent, UiImage::SolidColor(HEALTH_BAR_COLOR));
                    let id: String = String::from("health:") + &status.character_ent.id().to_string();
                    let mut relative_health: f32 = character.relative_health();
                    if relative_health <= 0.0 {
                        relative_health = 0.0;
                    }
                    let mut trans = UiTransform::new(
                        id,
                        Anchor::Middle,
                        Anchor::TopLeft,
                        -STATUS_WIDTH / 2.0, -(STATUS_HEIGHT / 2.0), 0.0,
                        STATUS_WIDTH * relative_health, STATUS_BAR_HEIGHT,
                    );
                    trans.opaque = false;
                    ui_transforms.insert(health_ent, trans);
                    health_bars.insert(health_ent, UiHealthBar::new(status.character_ent));
                    parents.insert(health_ent, Parent { entity });
                }

                if status.show_charge {
                    let charge_ent: Entity = entities.create();
                    images.insert(charge_ent, UiImage::SolidColor(CHARGE_BAR_COLOR));
                    let id: String = String::from("charge:") + &status.character_ent.id().to_string();
                    let mut relative_charge: f32 = character.relative_charge();
                    if relative_charge <= 0.0 {
                        relative_charge = 0.0;
                    }

                    let mut trans = UiTransform::new(
                        id,
                        Anchor::Middle,
                        Anchor::TopLeft,
                        -STATUS_WIDTH / 2.0, -(STATUS_HEIGHT / 2.0) - STATUS_BAR_HEIGHT, 0.0,
                        STATUS_WIDTH * relative_charge, STATUS_BAR_HEIGHT,
                    );

                    trans.opaque = false;
                    ui_transforms.insert(charge_ent, trans);
                    charge_bars.insert(charge_ent, UiChargeBar::new(status.character_ent));
                    parents.insert(charge_ent, Parent { entity });
                }

                if status.show_status_effects {
                    for (i, status) in StatusType::all().into_iter().enumerate() {
                        let side = i % 2;
                        let base_offset: f32 = {
                            if side == 0 {
                                RANK_ICON_SIZE / 2.0 + STATUS_ICON_SIZE / 2.0
                            } else {
                                -(RANK_ICON_SIZE / 2.0 + STATUS_ICON_SIZE / 2.0)
                            }
                        };
                        let multiplied_offset: f32 = {
                            if side == 0 {
                                (i / 2) as f32 * STATUS_ICON_SIZE
                            } else {
                                -((i / 2) as f32 * STATUS_ICON_SIZE)
                            }
                        };
                        let total_offset: f32 = base_offset + multiplied_offset;

                        let status_icon_ent: Entity = entities.create();
                        images.insert(status_icon_ent, UiImage::SolidColor([0.0; 4]));
                        let mut text: UiText = UiText::new(
                            fonts.status().clone(),
                            String::new(),
                            [1.0; 4],
                            20.0,
                        );
                        text.align = Anchor::TopRight;
                        texts.insert(status_icon_ent, text);

                        let id: String = String::from("status_icon:") + &status_icon_ent.id().to_string();
                        let mut trans = UiTransform::new(
                            id,
                            Anchor::BottomMiddle,
                            Anchor::BottomMiddle,
                            total_offset, STATUS_HEIGHT, 0.0,
                            STATUS_ICON_SIZE, STATUS_ICON_SIZE,
                        );
                        trans.opaque = false;
                        ui_transforms.insert(status_icon_ent, trans);
                        status_icons.insert(status_icon_ent, UiStatusIcon {
                            ty: status,
                            count: 0,
                        });
                        parents.insert(status_icon_ent, Parent { entity });
                    }
                }
            }
        }

        for (health_bar, mut ui_transform) in (&health_bars, &mut ui_transforms).join() {
            if let Some(character) = characters.get(health_bar.character_ent) {
                let mut relative_health: f32 = character.relative_health();
                if relative_health <= 0.0 {
                    relative_health = 0.0;
                }
                ui_transform.width = STATUS_WIDTH * relative_health;
            }
        }

        for (charge_bar, mut ui_transform) in (&charge_bars, &mut ui_transforms).join() {
            if let Some(character) = characters.get(charge_bar.character_ent) {
                let mut relative_charge: f32 = character.relative_charge();
                if relative_charge <= 0.0 {
                    relative_charge = 0.0;
                }
                ui_transform.width = STATUS_WIDTH * relative_charge;
            }
        }

        for (entity, status) in (&entities, &ui_statuses).join() {
            if let Some(character) = characters.get(status.character_ent) {
                for (status_ent, mut status_icon, mut image, mut text, _) in (&entities, &mut status_icons, &mut images, &mut texts, hierarchy.all_children(entity)).join() {
                    let value: usize = character.status(status_icon.ty);
                    status_icon.count = value;
                    if value > 0 {
                        if let UiImage::Texture(_) = image {}
                        else {
                            *image = UiImage::Texture(self.status_textures.get(&status_icon.ty).expect("No status icon for this status!").clone());
                        }
                        text.text = "x".to_string() + &value.to_string();
                    } else {
                        *image = UiImage::SolidColor([0.0; 4]);
                        text.text = String::new();
                    }
                }
            }
        }
    }
}