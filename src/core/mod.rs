use amethyst::{
    animation::{AnimationBundle, VertexSkinningBundle},
    assets::{
        AssetStorage,
        Handle,
        Loader,
        PrefabLoader,
        PrefabLoaderSystemDesc,
        RonFormat,
    },
    controls::{ControlTagPrefab, FlyControlBundle},
    core::{
        HideHierarchySystemDesc,
        Parent,
        transform::{
            Transform,
            TransformBundle,
        },
    },
    ecs::{
        prelude::*,
        storage::{
            GenericReadStorage,
            GenericWriteStorage,
        },
    },
    audio::AudioBundle,
    Error,
    gltf::GltfSceneLoaderSystemDesc,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderPbr3D, RenderSkybox, RenderToWindow},
        RenderingBundle,
        Texture,
        types::{DefaultBackend, Mesh, MeshData},
    },
    ui::{RenderUi, UiBundle},
    utils::{
        application_root_dir,
        auto_fov::AutoFovSystem,
        tag::{Tag, TagFinder},
    },
};
// Our own rendering plugins.
use amethyst_particle::ParticleRender;
use combat_render::flash::pass::FlashRender;
use space_render::AtmosphereRender;
use space_render::cosmos::{Cosmos, CosmosRender};
use space_render::StarRender;

pub use action::Action;
pub use add_to_limit::*;

use crate::game;
use crate::game::combat::process::Principal;
use crate::game::ui::font::GameFonts;
use crate::state::*;
use rand::Rng;
use crate::game::character::{CharacterStore, CharacterRole};
use crate::game::combat::ability::{AbilityList, AbilityUsability};
use crate::game::combat::ability::charge::ChargeAbility;
use crate::game::combat::ability::spawn::SpawnAbility;
use crate::game::combat::ability::hack::HackAbility;
use crate::game::combat::ability::barrage::BarrageAbility;
use crate::game::combat::ability::twin_shot::TwinShotAbility;
use crate::game::combat::ability::focused_charge::FocusedChargeAbility;
use crate::game::combat::ability::snipe::SnipeAbility;
use crate::game::combat::ability::annihilate::AnnihilateAbility;
use crate::game::map::CombatStore;
use crate::game::combat::{CombatData, Wave};
use crate::game::CoreGameBundle;
use crate::core::rebuild_pass::RebuildPlugin;

pub mod action;
pub mod activity;
pub mod rebuild_pass;
mod add_to_limit;

/// Builds the application object for the game.
/// This contains the built ECS data, which has all the required logic registered.
pub fn build_application<'a, 'b, 'c, S: State<AggregateData<'a, 'b>, StateEvent> + 'c>(initial_state: S) -> Result<Application<'c, AggregateData<'a, 'b>>, Error> {
    // Get the application root directory for asset loading.
    let app_root = application_root_dir()?;

    // Add our meshes directory to the asset loader.
    let assets_dir = app_root.join("assets");

    // Load display config
    let display_config_path = app_root.join("config\\display.ron");

    let mut world: World = World::new();

    let game_data = AggregateDataBuilder::new()
        // GAME SYSTEMS - these are only run when actually 'in' the game world, such as game logic and input systems.
        .with_combat(
            GameDataBuilder::new()
                // Our custom core game bundle data.
                .with_bundle(game::combat::CombatBundle)?
                .with_bundle(game::character::CharacterBundle)?
        )
        .with_map(
            GameDataBuilder::new()
                // Our custom core game bundle data.
                .with_bundle(game::map::MapBundle)?
        )
        // UI SYSTEMS - these should be ui specific and should not be required if not running with a UI.
        // TODO: implement UI
        .with_ui(
            GameDataBuilder::new()
                .with_system_desc(
                    crate::game::control::camera::menu::MenuCameraSystemDesc::default(),
                    "menu_camera",
                    &[],
                )
        )

        // CORE SYSTEMS - these should be present at all times during the lifecycle of the game.
        // This allows us to load assets at any time, even when not in the game state.
        // FOV will update wen window is resized.
        //.with_core(AutoFovSystem::default(), "auto_fov", &[])
        .with_core(
            GameDataBuilder::new()
                // Input bundle.
                .with_bundle(InputBundle::<StringBindings>::new())?
                // Automatic FOV and aspect ration modifications
                .with(
                    AutoFovSystem::new(),
                    "auto_fov",
                    &[],
                )
                // Register drone loader.
//                .with_system_desc(
//                    PrefabLoaderSystemDesc::<game::drone::DronePrefabData>::default(),
//                    "drone_loader",
//                    &[],
//                )
                // Register map loader.
                .with_system_desc(
                    PrefabLoaderSystemDesc::<game::map::MapPrefabData>::default(),
                    "map_loader",
                    &[],
                )
                // Register scene loader.
                .with_system_desc(
                    PrefabLoaderSystemDesc::<game::map::WorldPrefabData>::default(),
                    "scene_loader",
                    &[],
                )
                // Register character loader.
                .with_system_desc(
                    PrefabLoaderSystemDesc::<game::character::CharacterPrefabData>::default(),
                    "character_loader",
                    &[],
                )
                // 3D asset loading using the gltf format.
                .with_system_desc(
                    GltfSceneLoaderSystemDesc::default(),
                    "gltf_loader",
                    &["map_loader", "scene_loader", "character_loader"], // This is important so that entity instantiation is performed in a single frame.
                )
                // Animation system.
//                .with_bundle(
//                    AnimationBundle::<usize, Transform>::new("animation_control", "sampler_interpolation")
//                        .with_dep(&["gltf_loader"]),
//                )?
                // Basic transforms.
                .with_bundle(TransformBundle::new().with_dep(&[
                    //"animation_control",
                    //"sampler_interpolation",
                ]))?
                // Vertex skinning (applying bones to vertices) and manipulating.
//                .with_bundle(VertexSkinningBundle::new().with_dep(&[
//                    "transform_system",
//                    //"animation_control",
//                    "sampler_interpolation",
//                ]))?
                // Amethyst core UI bundle - this requires other systems within the dispatcher to function properly.
                .with_bundle(UiBundle::<StringBindings>::new())?
                // Audio bundle
                .with_bundle(AudioBundle::default())?
                // RENDER SYSTEMS - these should be required only for rendering abstract data to the screen.
                // Add the render bundle.
                .with_bundle(CoreGameBundle)?
                .with_system_desc(
                    HideHierarchySystemDesc::default(),
                    "hide_hierarchy",
                    &[],
                )
                .with_bundle(
                    RenderingBundle::<DefaultBackend>::new()
                        // Clear color is black - for custom background colors (e.g. for ui) extra geometry will need to be rendered.
                        .with_plugin(RenderToWindow::from_config_path(display_config_path)?.with_clear([0.0, 0.0, 0.0, 0.0]))
                        .with_plugin(RenderPbr3D::default())

                        // Our own custom plugins
                        .with_plugin(CosmosRender::new(Some(Cosmos::default())))
                        .with_plugin(FlashRender::new("textures/flash_billboard.png"))
                        .with_plugin(StarRender::new("textures/star_glow.png"))
                        .with_plugin(AtmosphereRender::default())
                        .with_plugin(ParticleRender::default())
                        .with_plugin(RebuildPlugin::default())

                        // Ui rendering.
                        .with_plugin(RenderUi::default())
                )?
        );

    Ok(Application::new(assets_dir, initial_state, game_data)?)
}

pub fn get_root<'s, 'a, T, P, C>(
    parents: &P, components: &'a C,
    entity: Entity,
) -> Option<(&'a T, Entity)>
    where T: Component, P: GenericReadStorage<Component=Parent>, C: GenericReadStorage<Component=T> {
    if let Some(component) = components.get(entity) {
        Some((component, entity))
    } else {
        let parent_ent: Entity = parents.get(entity)?.entity;
        get_root(
            parents, components,
            parent_ent,
        )
    }
}

pub fn get_root_cloned<'s, 'a, T, P, C>(
    parents: &P, components: &'a C,
    entity: Entity,
) -> Option<(T, Entity)>
    where T: Component + Clone, P: GenericReadStorage<Component=Parent>, C: GenericReadStorage<Component=T> {
    if let Some(component) = components.get(entity) {
        Some((component.clone(), entity))
    } else {
        let parent_ent: Entity = parents.get(entity)?.entity;
        get_root_cloned(
            parents, components,
            parent_ent,
        )
    }
}

pub fn get_root_mut<'s, 'a, T, P, C>(
    parents: &P, components: &'a mut C,
    entity: Entity,
) -> Option<(&'a mut T, Entity)>
    where T: Component, P: GenericReadStorage<Component=Parent>, C: GenericWriteStorage<Component=T> {
    if components.get_mut(entity).is_some() {
        Some((components.get_mut(entity).unwrap(), entity))
    } else {
        let parent_ent: Entity = parents.get(entity)?.entity;
        get_root_mut(
            parents, components,
            parent_ent,
        )
    }
}

pub fn load_texture(world: &mut World, path: impl Into<String>) -> Handle<Texture> {
    if !world.has_value::<AssetStorage::<Texture>>() {
        world.insert(AssetStorage::<Texture>::new());
    }
    let loader = world.read_resource::<Loader>();
    loader.load(
        path,
        amethyst::renderer::formats::texture::ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    )
}

/// Rolls a 'dice' based on the specified chance.
pub fn roll(chance: f32) -> bool {
    let mut rng = rand::thread_rng();
    let value: f32 = rng.gen_range(0.0, 1.0);
    if value < chance {
        true
    } else {
        false
    }
}

pub fn select_rng(values: &[f32]) -> Option<usize> {
    if values.is_empty() {
        return None;
    }
    let mut total: f32 = 0.0;
    for value in values {
        total += *value;
    }
    if total <= 0.0 {
        return None;
    }
    let mut rng = rand::thread_rng();
    let random: f32 = rng.gen_range(0.0, total);
    let mut current: f32 = 0.0;
    for (i, value) in values.iter().enumerate() {
        if *value != 0.0 {
            let next: f32 = current + *value;
            if random >= current && random < next {
                return Some(i);
            }
            current = next;
        }
    }
    panic!("No value selected. The slice must contain non zero values...");
}