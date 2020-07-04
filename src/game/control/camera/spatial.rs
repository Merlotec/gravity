use amethyst::{
    core::{
        math::{
            Point3,
            Vector3,
        },
        transform::Transform,
    },
    ecs::*,
    prelude::*,
    renderer::camera::{
        ActiveCamera,
        Camera,
        Perspective,
    },
};

use crate::game::control::{
    GameState,
    ViewState,
};

/// The system which controls the spatial camera - the camera which is active when looking around in real time at space.
#[derive(Debug, Default)]
pub struct SpatialCameraSystem {
    is_active: bool,
}

impl<'a> System<'a> for SpatialCameraSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, ActiveCamera>,
        Read<'a, GameState>,
        WriteStorage<'a, Transform>
    );
    fn run(&mut self, (entities, active_camera, game_state, mut transform): Self::SystemData) {
        if let Some(camera_entity) = active_camera.entity {
            // We have a valid camera, so lets manipulate it.
            if let Some(camera_transform) = transform.get_mut(camera_entity) {
                // Move the camera.
            }
        }
    }
}

pub fn load_spatial_camera(transform: &mut Transform) {}