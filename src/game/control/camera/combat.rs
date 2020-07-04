use amethyst::{
    core::{
        math::Vector3,
        Time,
        Transform,
    },
    ecs::prelude::*,
    input::{Button, InputHandler, StringBindings}, prelude::SystemDesc, renderer::Camera, shrev::{
        EventChannel,
        ReaderId,
    }, Trans, winit::{
        DeviceEvent,
        Event,
        MouseButton,
        MouseScrollDelta,
        WindowEvent,
    }};

#[derive(Debug, Copy, Clone)]
pub struct CombatCameraTag;

impl Component for CombatCameraTag {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(CombatCameraSystemDesc))]
pub struct CombatCameraSystem {
    sensitivity_x: f32,
    sensitivity_y: f32,

    min_rot: f32,
    max_rot: f32,

    scroll_sensitivity: f32,

    min_dist: f32,
    max_dist: f32,

    current_x: f32,
    current_y: f32,

    is_focused: bool,

    #[system_desc(event_channel_reader)]
    event_reader: ReaderId<Event>,
}

impl<'s> System<'s> for CombatCameraSystem {
    type SystemData = (
        Read<'s, EventChannel<Event>>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, CombatCameraTag>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (events, mut transforms, cameras, tags, input): Self::SystemData) {
        for event in events.read(&mut self.event_reader) {
            if self.is_focused {
                if let Event::DeviceEvent { ref event, .. } = *event {
                    if input.button_is_down(Button::Mouse(MouseButton::Right)) {
                        if let DeviceEvent::MouseMotion { delta: (x, y) } = *event {
                            self.current_x += (-(y as f32) * self.sensitivity_y).to_radians();
                            self.current_y += (-(x as f32) * self.sensitivity_x).to_radians();
                            if self.current_x > self.max_rot {
                                self.current_x = self.max_rot;
                            } else if self.current_x < self.min_rot {
                                self.current_x = self.min_rot;
                            }
                            for (transform, _) in (&mut transforms, &tags).join() {
                                transform.set_rotation_euler(self.current_x, self.current_y, 0.0);
                            }
                        }
                    }
                    if let DeviceEvent::MouseWheel { delta } = *event {
                        let scroll_value = match delta {
                            MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                            MouseScrollDelta::LineDelta(_, y) => -y,
                        };
                        for (transform, _) in (&mut transforms, &cameras).join() {
                            let delta: f32 = scroll_value as f32 * transform.translation().z * self.scroll_sensitivity;
                            let mut val: f32 = transform.translation().z + delta;
                            if val > self.max_dist {
                                val = self.max_dist;
                            } else if val < self.min_dist {
                                val = self.min_dist;
                            }
                            transform.set_translation_z(val);
                        }
                    }
                }
            }

            if let Event::WindowEvent { ref event, .. } = *event {
                if let WindowEvent::Focused(focused) = *event {
                    self.is_focused = focused;
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct CameraTargetPoint(pub Option<Vector3<f32>>, pub Option<Entity>);

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(CameraDriftSystemDesc))]
pub struct CameraDriftSystem {
    speed: f32,
}

impl<'s> System<'s> for CameraDriftSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, CombatCameraTag>,
        Write<'s, CameraTargetPoint>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, tags, mut target_point, time): Self::SystemData) {
        if let Some(target) = target_point.0 {
            for (mut transform, _) in (&mut transforms, &tags).join() {
                let current: Vector3<f32> = *transform.translation();
                let delta: Vector3<f32> = target - current;
                let abs: f32 = delta.norm();
                let v: f32 = delta.norm() * self.speed + 4.0;
                let adj: f32 = v * time.delta_seconds();
                if adj >= abs {
                    transform.set_translation(target);
                    // We've arrived at the target point.
                    target_point.0 = None;
                } else {
                    transform.prepend_translation(delta.normalize() * adj);
                }
            }
        }
    }
}
