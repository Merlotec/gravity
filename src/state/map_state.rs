use amethyst::{
    assets::{
        Handle,
        Prefab,
        PrefabLoader,
        PrefabLoaderSystemDesc,
        RonFormat,
        ProgressCounter,
    },
    core::{
        math::Vector3,
        Parent,
        Transform,
    },
    ecs::prelude::*,
    input::{InputBundle, is_close_requested, is_key_down, StringBindings},
    prelude::*,
    renderer::camera::{
        Camera,
        Projection,
    },
    shrev::EventChannel,
    utils::{
        auto_fov::AutoFov,
        tag::Tag,
    },
    window::ScreenDimensions,
    winit::VirtualKeyCode,
};
use combat_render::flash::Flash;
use failure::_core::any::TypeId;
use nalgebra::{Quaternion, UnitQuaternion};

use crate::{
    core::action::{Action, Invoke},
    game::{
        character::master::MasterDrone,
        combat::spawn::{SlotManager, SpawnProcess, SpawnSource},
    },
    state::AggregateData,
};
use crate::game::combat::CombatData;
use crate::game::map::{MapPrefabData, CurrentState, MapStage, MapPawn, MapPoint, MapRoot, EngageCombat};
use crate::game::character::{CharacterPrefabData, CharacterStore};
use std::thread::sleep;
use crate::state::combat_state::CombatState;
use crate::game::combat::process::Principal;
use crate::core::rebuild_pass::RebuildRendering;
use crate::game::ui::map_notification::UiMapNotification;

pub const MAP_STD_FOV: f32 = 1.4;
pub const MAP_MAX_FOV: f32 = std::f32::consts::PI;
pub const MAP_MIN_FOV: f32 = 0.8;

#[derive(Default)]
pub struct MapState {
    initial_idx: Option<usize>,
}

impl MapState {
    pub fn new(initial_idx: Option<usize>) -> Self {
        Self {
            initial_idx,
        }
    }
}

impl<'a, 'b> State<AggregateData<'a, 'b>, StateEvent> for MapState {
    fn on_start(&mut self, data: StateData<'_, AggregateData<'a, 'b>>) {
        data.world.delete_all();
        // Top down camera.
        data.world.exec(|(entities, mut transforms, mut cameras, mut auto_fovs, dims): (Entities, WriteStorage<Transform>, WriteStorage<Camera>, WriteStorage<AutoFov>, ReadExpect<ScreenDimensions>)| {
            // The camera itself.
            let camera_ent: Entity = entities.create();
            let camera: Camera = Camera::from(Projection::perspective(
                dims.aspect_ratio(),
                crate::STD_FOV, // y FOV in radians
                0.1, // near
                -1000000000.0, // far
            ));
//            let camera_size: f32 = 1000.0;
//            let camera: Camera = Camera::from(Projection::orthographic(
//                -camera_size * crate::STD_FOV,
//                camera_size * crate::STD_FOV,
//                -camera_size,
//                camera_size,
//                0.0,
//                -1000000000.0,
//            ));
            cameras.insert(camera_ent, camera);
            let mut camera_transform = Transform::from(Vector3::new(0.0, 10000.0, 0.0));
            camera_transform.append_rotation_x_axis(-std::f32::consts::PI / 2.0);
            transforms.insert(camera_ent, camera_transform);
            // Auto fov.
            let mut auto_fov: AutoFov = AutoFov::new();
            auto_fov.set_base_aspect_ratio(dims.width() as usize, dims.height() as usize);
            auto_fov.set_base_aspect_ratio(dims.width() as usize, dims.height() as usize);
            auto_fov.set_min(crate::MIN_FOV * dims.aspect_ratio());
            auto_fov.set_max(crate::MAX_FOV * dims.aspect_ratio());
            auto_fov.set_base_fovx(crate::STD_FOV * dims.aspect_ratio());

            auto_fovs.insert(camera_ent, auto_fov);
        });

        // Load map
        let map_handle = data.world.exec(
            |loader: PrefabLoader<'_, MapPrefabData>| {
                loader.load(
                    "maps/map.ron",
                    RonFormat,
                    (),
                )
            },
        );

        let max_point: usize = data.world.read_resource::<CurrentState>().max_point;

        // Add the loaded entities to the scene.
        let mut map_ent: Entity = data.world.create_entity().with(map_handle).build();

        let master_handle: Handle<Prefab<CharacterPrefabData>> = data.world.read_resource::<CharacterStore>().prefab(&MasterDrone::character_id()).expect("No master drone prefab!").clone();

        let pawn_ent: Entity = data.world.create_entity().with(MapPawn::default()).with(master_handle).build();

        let root_ent: Entity = data.world.create_entity().with(MapRoot { pawn_ent, point_idx: self.initial_idx.unwrap_or(max_point), }).with(Principal::new()).build();

        data.world.write_component::<Parent>().insert(pawn_ent, Parent { entity: root_ent });

        data.world.write_component::<Parent>().insert(map_ent, Parent { entity: root_ent });

        data.world.insert::<Option<EngageCombat>>(None);

        data.world.create_entity().with(UiMapNotification { root_ent }).build();

        // Rebuild view.
        *data.world.write_resource::<RebuildRendering>() = RebuildRendering(true);
    }

    fn update(&mut self, data: StateData<AggregateData<'a, 'b>>) -> Trans<AggregateData<'a, 'b>, StateEvent> {
        data.data.dispatch_all(data.world);
        if let Some(engage) = data.world.fetch_mut::<Option<EngageCombat>>().take() {
            Trans::Switch(Box::new(
                CombatState::with_combat(engage.combat_data, Some(engage.point_idx))
            ))
        } else {
            Trans::None
        }
    }

    fn handle_event(
        &mut self,
        _: StateData<AggregateData<'a, 'b>>,
        event: StateEvent,
    ) -> Trans<AggregateData<'a, 'b>, StateEvent> {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(ui_event) => {
                Trans::None
            }
            StateEvent::Input(input) => {
                Trans::None
            }
        }
    }
}