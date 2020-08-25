use amethyst::{
    animation::{AnimationBundle, VertexSkinningBundle},
    assets::{Completion, Format as AssetFormat, Handle, Loader, Prefab, ProgressCounter},
    assets::{
        PrefabLoader,
        PrefabLoaderSystemDesc,
        RonFormat,
        AssetStorage,
    },
    controls::{ControlTagPrefab, FlyControlBundle},
    core::{
        math::{
            Isometry3,
            Point3,
            Translation,
            UnitQuaternion,
            Vector3,
        },
        Parent,
        transform::{
            Transform,
            TransformBundle,
        },
        ParentHierarchy,
    },
    audio::{
        SourceHandle,
        OggFormat,
        output::Output,
        Source,
    },
    ecs::{World, WorldExt},
    ecs::prelude::*,
    error::Error,
    gltf::GltfSceneLoaderSystemDesc,
    input::{InputBundle, is_close_requested, is_key_down, StringBindings},
    prelude::*,
    renderer::{
        camera::{Camera, Projection},
        light::{Light, PointLight},
        mtl::{Material, MaterialDefaults},
        palette::{Srgb, Srgba},
        plugins::{RenderPbr3D, RenderSkybox, RenderToWindow},
        RenderingBundle,
        rendy::{
            mesh::{MeshBuilder, Normal, Position, TexCoord},
            texture::palette::load_from_srgba,
        },
        types::{DefaultBackend, Mesh, MeshData},
    },
    ui::{UiCreator, UiEvent, UiEventType, UiFinder, UiTransform, UiText},
    utils::{
        auto_fov::AutoFovSystem,
        tag::{Tag, TagFinder},
    },
    utils::application_root_dir,
    utils::auto_fov::AutoFov,
    window::ScreenDimensions,
    winit::VirtualKeyCode,
};
use amethyst_particle::ParticleRender;
use space_render::AtmosphereRender;
use space_render::cosmos::{Cosmos, CosmosRender};
use space_render::StarRender;
use std::process::exit;

use crate::game;
use crate::game::combat::{CombatData, Wave, Rank, CharacterSpawn};
use crate::game::control::camera::menu::MenuCamera;
use crate::physics::PhysicsBody;
use crate::state::AggregateData;
use crate::state::combat_state::CombatState;
use crate::state::map_state::MapState;

use game::character::spacebot::SpacebotDrone;
use crate::game::map::{CurrentState, save_current};

pub const CAMERA_ROTATION_SPEED: f32 = 0.02;

/// The game scene root.
#[derive(Default)]
pub struct MainMenuState {
    pub difficulty_ent: Option<Entity>,
    pub music_handle: Option<SourceHandle>,
    pub playing_music: bool,
}

impl<'a, 'b> State<AggregateData<'a, 'b>, StateEvent> for MainMenuState {
    fn on_start(&mut self, data: StateData<'_, AggregateData<'a, 'b>>) {
        let StateData { world, .. } = data;

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/menu.ron", ());
        });

        // Set up a camera for star rendering...
        // Set up camera.
        world.exec(|(entities, mut parents, mut menu_cameras, mut transforms, mut cameras, mut auto_fovs, dims): (Entities, WriteStorage<Parent>, WriteStorage<MenuCamera>, WriteStorage<Transform>, WriteStorage<Camera>, WriteStorage<AutoFov>, ReadExpect<ScreenDimensions>)| {
            // The camera itself.
            let camera_ent: Entity = entities.create();
            let camera: Camera = Camera::from(Projection::perspective(
                dims.aspect_ratio(),
                crate::STD_FOV, // y FOV in radians
                0.1, // near
                -10000000.0, // far
            ));
            cameras.insert(camera_ent, camera);
            let mut camera_transform = Transform::from(Vector3::new(0.0, 0.0, 10.0));
            transforms.insert(camera_ent, camera_transform);
            menu_cameras.insert(camera_ent, MenuCamera::new(CAMERA_ROTATION_SPEED));
            // Auto fov.
            let mut auto_fov: AutoFov = AutoFov::new();
            auto_fov.set_base_aspect_ratio(dims.width() as usize, dims.height() as usize);
            auto_fov.set_min(crate::MIN_FOV * dims.aspect_ratio());
            auto_fov.set_max(crate::MAX_FOV * dims.aspect_ratio());
            auto_fov.set_base_fovx(crate::STD_FOV * dims.aspect_ratio());
            auto_fovs.insert(camera_ent, auto_fov);
        });

        // Load music.
        self.music_handle = Some(world.read_resource::<Loader>().load("music/theme.ogg", OggFormat, (), &world.read_resource()));
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, AggregateData<'a, 'b>>,
        event: StateEvent,
    ) -> Trans<AggregateData<'a, 'b>, StateEvent> {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(ui_event) => {
                if ui_event.event_type == UiEventType::Click {
                    if let Some(transform) = data.world.read_storage::<UiTransform>().get(ui_event.target).cloned() {
                        match transform.id.as_str() {
                            "play" => {
                                Trans::Switch(
                                    Box::new(
                                        MapState::new(None),
                                    )
                                )
                            },
                            "difficulty" => {
                                let mut current_state = data.world.write_resource::<CurrentState>();
                                let difficulty = &mut current_state.difficulty;
                                *difficulty = difficulty.next();
                                save_current(&current_state);
                                Trans::None
                            },
                            "quit" => Trans::Quit,
                            _ => Trans::None,
                        }
                    } else {
                        Trans::None
                    }
                } else {
                    Trans::None
                }
            }
            StateEvent::Input(input) => {
                Trans::None
            }
        }
    }

    fn update(&mut self, data: StateData<AggregateData<'a, 'b>>) -> Trans<AggregateData<'a, 'b>, StateEvent> {
        // We dont need to dispatch the game logic.
        if self.difficulty_ent.is_none() {
            data.world.exec(|finder: UiFinder<'_>| {
                if let Some(entity) = finder.find("difficulty") {
                    self.difficulty_ent = Some(entity);
                }
            });
        }
        if let Some(difficulty_ent) = self.difficulty_ent {
            let mut ui_texts = data.world.write_storage::<UiText>();
            for (mut text, _) in (&mut ui_texts, data.world.read_resource::<ParentHierarchy>().all_children(difficulty_ent)).join() {
                let difficulty = "Difficulty: ".to_string() + &data.world.read_resource::<CurrentState>().difficulty.to_string();
                text.text = difficulty;
            }
        }

        // if !self.playing_music {
        //     if let Some(ref handle) = self.music_handle {
        //         if let Some(output) = data.world.try_fetch::<Output>() {
        //             let storage = data.world.read_resource::<AssetStorage<Source>>();
        //             if let Some(sound) = storage.get(&handle) {
        //                 output.play_n_times(sound, 1.0, 500);
        //                 self.playing_music = true;
        //             }
        //         }
        //     }
        // }

        data.data.dispatch_all(data.world);
        Trans::None
    }
}
