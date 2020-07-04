use std::collections::HashMap;

use amethyst::{
    assets::{
        AssetStorage,
        Handle,
        Loader,
    },
    core::{
        math::{
            Vector2,
            Vector3,
            Vector4,
        },
        Parent,
        Transform,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    renderer::{
        ActiveCamera,
        Camera,
        formats::texture::ImageFormat,
        Sprite,
        Texture,
    },
    shrev::{
        EventChannel,
        ReaderId,
    },
    ui::{
        Anchor,
        FontAsset,
        FontHandle,
        Interactable,
        TtfFormat,
        UiButton,
        UiEvent,
        UiEventType,
        UiImage,
        UiTransform,
    },
    window::ScreenDimensions,
};

use crate::core::get_root;
use crate::game::character::Character;
use crate::game::combat::process::Principal;
use crate::game::combat::Team;
use crate::game::ui::ability::{ShowAbilitiesEvent, UiAbilitySelection, UiAbilitySelectionTag};
use crate::game::ui::hud::{UiBase, UiCharacterBase};

pub const CROSSHAIR_BASE_FACTOR: f32 = 3.0;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CrosshairType {
    Passive(Team),
    Target(Team),
}

impl CrosshairType {
    pub fn team(&self) -> Team {
        match self {
            CrosshairType::Passive(team) => *team,
            CrosshairType::Target(team) => *team,
        }
    }

    pub fn set_team(&mut self, new_team: Team) {
        match self {
            CrosshairType::Passive(team) => *team = new_team,
            CrosshairType::Target(team) => *team = new_team,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct UiCrosshair {
    pub ty: CrosshairType,
    pub uniform_scale: f32,
    pub hover_ty: Option<CrosshairType>,
    pub hover: bool,
    pub visible: bool,
}

impl UiCrosshair {
    pub(crate) fn new(ty: CrosshairType) -> Self {
        Self {
            ty,
            uniform_scale: 1.0,
            hover_ty: None,
            hover: false,
            visible: true,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CrosshairClickedEvent(pub Entity);

impl CrosshairClickedEvent {
    #[inline]
    pub fn entity(&self) -> Entity {
        self.0
    }
}


#[derive(Debug, SystemDesc, new)]
#[system_desc(name(CrosshairUiSystemDesc))]
pub struct CrosshairUiSystem {
    crosshairs: HashMap<CrosshairType, Handle<Texture>>,

    #[system_desc(event_channel_reader)]
    ui_reader: ReaderId<UiEvent>,
}

impl<'s> System<'s> for CrosshairUiSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Principal>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiCrosshair>,
        WriteStorage<'s, UiCharacterBase>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, Interactable>,
        Write<'s, UiAbilitySelection>,
        ReadStorage<'s, UiBase>,
        Write<'s, EventChannel<CrosshairClickedEvent>>,
        Read<'s, EventChannel<UiEvent>>,
        ReadExpect<'s, ScreenDimensions>,
    );

    fn setup(&mut self, world: &mut World) {
        if !world.has_value::<AssetStorage::<Texture>>() {
            world.insert(AssetStorage::<Texture>::new());
        }
        let loader = world.read_resource::<Loader>();
        self.crosshairs.insert(
            CrosshairType::Target(Team::Friendly),
            loader.load(
                "ui/crosshair_friendly_target.png",
                ImageFormat::default(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            ),
        );
        self.crosshairs.insert(
            CrosshairType::Passive(Team::Friendly),
            loader.load(
                "ui/crosshair_friendly_passive.png",
                ImageFormat::default(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            ),
        );
        self.crosshairs.insert(
            CrosshairType::Target(Team::Enemy),
            loader.load(
                "ui/crosshair_enemy_target.png",
                ImageFormat::default(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            ),
        );
        self.crosshairs.insert(
            CrosshairType::Passive(Team::Enemy),
            loader.load(
                "ui/crosshair_enemy_passive.png",
                ImageFormat::default(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            ),
        );
    }


    fn run(&mut self, (entities, principals, parents, mut images, mut ui_crosshairs, ui_character_bases, mut ui_transforms, mut interactables, mut ui_ability_selection, ui_bases, mut crosshair_clicked_events, ui_events, dims): Self::SystemData) {

        // Update the crosshair.
        for (entity, mut crosshair, mut ui_transform, mut image) in (&entities, &mut ui_crosshairs, &mut ui_transforms, &mut images).join() {
            *image = {
                if crosshair.visible {
                    ui_transform.opaque = true;
                    match crosshair.hover {
                        true => {
                            match &crosshair.hover_ty {
                                Some(ty) => UiImage::Texture(
                                    self.crosshairs.get(ty).cloned().expect("No crosshair!"),
                                ),
                                None => UiImage::Texture(
                                    self.crosshairs.get(&crosshair.ty).cloned().expect("No crosshair!"),
                                ),
                            }
                        }
                        false => UiImage::Texture(
                            self.crosshairs.get(&crosshair.ty).cloned().expect("No crosshair!"),
                        ),
                    }
                } else {
                    ui_transform.opaque = false;
                    ;
                    UiImage::SolidColor([0.0; 4])
                }
            };
            if let Some((character_base, _)) = get_root::<UiCharacterBase, _, _>(&parents, &ui_character_bases, entity) {
                ui_transform.width = CROSSHAIR_BASE_FACTOR * dims.height() * character_base.z_factor * crosshair.uniform_scale;
                ui_transform.height = CROSSHAIR_BASE_FACTOR * dims.height() * character_base.z_factor * crosshair.uniform_scale;
            }
        }

        // Insertion of crosshair ui elements.
        let transform_mask = ui_transforms.mask().clone();
        let image_mask = images.mask().clone();
        for (entity, crosshair, _) in (&entities, &ui_crosshairs, !(image_mask & transform_mask)).join() {
            let crosshair_texture = self.crosshairs.get(&crosshair.ty).cloned().expect("No crosshair!");
            images.insert(entity, UiImage::Texture(
                crosshair_texture
            ));
            //TODO: fix duplicate ids.
            let id: String = String::from("crosshair:") + &entity.id().to_string();
            let mut ui_transform = UiTransform::new(
                id,
                Anchor::BottomLeft,
                Anchor::Middle,
                0.0, 0.0, 0.0,
                CROSSHAIR_BASE_FACTOR * dims.height() * crosshair.uniform_scale, CROSSHAIR_BASE_FACTOR * dims.height() * crosshair.uniform_scale,
            );
            if let Some((character_base, _)) = get_root::<UiCharacterBase, _, _>(&parents, &ui_character_bases, entity) {
                ui_transform.width = CROSSHAIR_BASE_FACTOR * dims.height() * character_base.z_factor * crosshair.uniform_scale;
                ui_transform.height = CROSSHAIR_BASE_FACTOR * dims.height() * character_base.z_factor * crosshair.uniform_scale;
            }
            ui_transforms.insert(entity, ui_transform);

            interactables.insert(entity, Interactable::default());
        }

        for ui_event in ui_events.read(&mut self.ui_reader) {
            if let Some(ui_crosshair) = ui_crosshairs.get_mut(ui_event.target) {
                match ui_event.event_type {
                    UiEventType::Click => {
                        if ui_crosshair.visible {
                            // Clicked on an ability.
                            let event: CrosshairClickedEvent = CrosshairClickedEvent(ui_event.target);
                            crosshair_clicked_events.single_write(event);
                        }
                    },
                    UiEventType::HoverStart => {
                        ui_crosshair.hover = true;
                    },
                    UiEventType::HoverStop => {
                        ui_crosshair.hover = false;
                    }
                    _ => {},
                }
            }
        }
    }
}

