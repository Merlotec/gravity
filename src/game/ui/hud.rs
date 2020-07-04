use amethyst::{
    core::{
        math::{
            Vector2,
            Vector3,
            Vector4,
        },
        Transform,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    renderer::{
        ActiveCamera,
        Camera,
    },
    ui::{
        Anchor,
        UiTransform,
    },
    window::ScreenDimensions,
};

use crate::game::character::Character;

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct UiCharacterBase {
    pub character_ent: Entity,
    pub z_factor: f32,
}

impl UiCharacterBase {
    pub fn new(character_ent: Entity) -> Self {
        Self {
            character_ent,
            z_factor: 1.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component)]
pub struct UiBase(pub Entity);

impl UiBase {
    #[inline]
    pub fn entity(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Clone, Default, SystemDesc)]
#[system_desc(name(UiCharacterBaseSystemDesc))]
pub struct UiCharacterBaseSystem;

impl<'s> System<'s> for UiCharacterBaseSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Character>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        Read<'s, ActiveCamera>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, UiCharacterBase>,
        WriteStorage<'s, UiBase>,
        ReadExpect<'s, ScreenDimensions>,
    );

    fn run(&mut self, (entities, characters, transforms, cameras, active_camera, mut ui_transforms, mut ui_character_bases, mut ui_bases, dims): Self::SystemData) {
        // let ui_character_base_mask: BitSet = character_bases.mask().clone();
        for (ui_ent, ui_character_base, _) in (&entities, &ui_character_bases, !ui_transforms.mask().clone()).join() {
            let id: String = String::from("base:") + &ui_character_base.character_ent.id().to_string();
            ui_transforms.insert(ui_ent, UiTransform::new(
                id,
                Anchor::BottomLeft,
                Anchor::BottomLeft,
                -1000.0, -1000.0, 0.0,
                0.0, 0.0,
            ));
        }

        for (entity, mut ui_character_base, mut ui_transform) in (&entities, &mut ui_character_bases, &mut ui_transforms).join() {

            // COPIED FROM `CameraGatherer`
            let camera_fetch = match active_camera.entity {
                Some(entity) => {
                    if transforms.contains(entity) && cameras.contains(entity) {
                        Some(entity)
                    } else {
                        (&entities, &cameras, &transforms)
                            .join()
                            .next()
                            .map(|(entity, _, _)| entity)
                    }
                }
                None => (&entities, &cameras, &transforms)
                    .join()
                    .next()
                    .map(|(entity, _, _)| entity),
            };

            if let Some(character_transform) = transforms.get(ui_character_base.character_ent) {
                if let Some(camera_ent) = camera_fetch {
                    if let Some(camera) = cameras.get(camera_ent) {
                        if let Some(camera_transform) = transforms.get(camera_ent) {
                            // Depth factor
                            let delta: Vector4<f32> = Vector4::from(character_transform.global_matrix().column(3).clone_owned()) - Vector4::from(camera_transform.global_matrix().column(3).clone_owned());
                            ui_character_base.z_factor = 1.0 / delta.xyz().norm();

                            let mut worldspace: Vector4<f32> = character_transform.global_matrix().column(3).clone_owned().into();
                            worldspace.w = 1.0;
                            let mut cameraspace: Vector4<f32> = camera_transform.global_view_matrix() * worldspace;
                            cameraspace /= cameraspace.w;
                            let mut screenspace: Vector4<f32> = camera.as_matrix() * cameraspace;
                            screenspace /= screenspace.w;
                            let pos2d: Vector2<f32> = Vector2::new((screenspace.x + 1.0) / 2.0, (-screenspace.y + 1.0) / 2.0);
                            let ui_space: Vector2<f32> = Vector2::new(pos2d.x * dims.width(), pos2d.y * dims.height());
                            let id: String = String::from("base:") + &entity.id().to_string();
                            if screenspace.z > -1.0 && screenspace.z < 1.0 {
                                ui_transform.local_x = ui_space.x;
                                ui_transform.local_y = ui_space.y;
                                ui_transform.local_z = -screenspace.z;
//                                ui_transforms.insert(entity, UiTransform::new(
//                                    id,
//                                    Anchor::BottomLeft,
//                                    Anchor::BottomLeft,
//                                    ui_space.x, ui_space.y, 0.0,
//                                    100.0, 100.0,
//                                ));
                            } else {
                                ui_transform.local_x = -1000.0;
                                ui_transform.local_y = -1000.0;
                            }
                        }
                    }
                }
            }
        }
    }
}
