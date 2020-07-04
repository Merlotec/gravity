use amethyst::{
    core::{
        SystemDesc,
        Time,
        Transform,
    },
    ecs::prelude::*,
    renderer::Camera,
};

#[derive(Debug, Copy, Clone, new, Component)]
pub struct MenuCamera {
    pub rotation_speed: f32,
}

#[derive(Debug, Default, new, SystemDesc)]
#[system_desc(name(MenuCameraSystemDesc))]
pub struct MenuCameraSystem;

impl<'s> System<'s> for MenuCameraSystem {
    type SystemData = (
        ReadStorage<'s, Camera>,
        ReadStorage<'s, MenuCamera>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (cameras, menu_cameras, mut transforms, time): Self::SystemData) {
        for (_, menu_camera, mut transform) in (&cameras, &menu_cameras, &mut transforms).join() {
            transform.append_rotation_x_axis(menu_camera.rotation_speed * time.delta_seconds());
            transform.append_rotation_y_axis(menu_camera.rotation_speed * time.delta_seconds());
            transform.append_rotation_z_axis(menu_camera.rotation_speed * time.delta_seconds());
        }
    }
}