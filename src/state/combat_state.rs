use amethyst::{
    assets::{
        Handle,
        Prefab,
        PrefabLoader,
        PrefabLoaderSystemDesc,
        RonFormat,
    },
    core::{
        math::Vector3,
        Parent,
        Transform,
        Time,
    },
    ecs::prelude::*,
    input::{InputBundle, is_close_requested, is_key_down, StringBindings, InputEvent},
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
use crate::core::activity::ActivityState;
use crate::game::character::{Character, CharacterId, CharacterPrefabData, CharacterRole, CharacterSpawnError, CharacterStore, UnassignedCharacter, CharacterData};
use crate::game::character::spacebot::SpacebotDrone;
use crate::game::character::spangles::SpanglesDrone;
use crate::game::character::defender::DefenderDrone;
use crate::game::combat::{CombatData, CombatRoot, Team, Wave, Rank};
use crate::game::combat::ability::{AbilityData, AbilityList, AbilityTargetArea, AbilityTargetInfo, AbilityTargetType, AbilityUsability};
use crate::game::combat::ability::annihilate::AnnihilateAbility;
use crate::game::combat::ability::barrage::BarrageAbility;
use crate::game::combat::ability::charge::ChargeAbility;
use crate::game::combat::ability::focused_charge::FocusedChargeAbility;
use crate::game::combat::ability::snipe::SnipeAbility;
use crate::game::combat::ability::spawn::SpawnAbility;
use crate::game::combat::ability::twin_shot::TwinShotAbility;
use crate::game::combat::process::Principal;
use crate::game::combat::spawn::SpawnAction;
use crate::game::control::camera::combat::CombatCameraTag;
use crate::game::map::{WorldPrefabData, CombatStore, CurrentState, MapStage, save_current, DialogueStore};
use crate::game::ui::{
    UiDisengageEvent,
    turn_notification::UiTurnNotification,
};
use crate::game::combat::ability::hack::HackAbility;
use crate::game::combat::systems::enemy_wave::SpawnWaveEvent;
use crate::core::rebuild_pass::RebuildRendering;
use crate::game::combat::systems::standard_combat::{ExitCombat, EXIT_TIMER};
use crate::state::map_state::MapState;
use std::fs::File;
use crate::game::ui::dialogue::ShowDialogueDisplayEvent;

#[derive(Debug, Clone)]
pub struct CombatState {
    combat: CombatData,
    point_idx: Option<usize>,
    combat_root: Option<Entity>,
    friendly_root: Option<Entity>,
    enemy_root: Option<Entity>,
}

impl CombatState {
    pub fn with_combat(combat: CombatData, point_idx: Option<usize>) -> Self {
        Self {
            combat,
            point_idx,
            combat_root: None,
            friendly_root: None,
            enemy_root: None,
        }
    }
}

impl<'a, 'b> State<AggregateData<'a, 'b>, StateEvent> for CombatState {
    fn on_start(&mut self, data: StateData<'_, AggregateData<'a, 'b>>) {
        // Start fresh
        data.world.delete_all();

        let combat_data: CombatData = self.combat.clone();
        let StateData { world, .. } = data;

        // Load map
        let map_handle = world.exec(
            |loader: PrefabLoader<'_, WorldPrefabData>| {
                loader.load(
                    combat_data.prefab_path(),
                    RonFormat,
                    (),
                )
            },
        );

        // Add the loaded entities to the scene.
        world.create_entity().with(map_handle).build();

        // Set up our combat instance.
        world.exec(|(entities, mut parents, mut transforms, mut roots, mut principals, mut slot_managers, mut teams): (Entities, WriteStorage<Parent>, WriteStorage<Transform>, WriteStorage<CombatRoot>, WriteStorage<Principal>, WriteStorage<SlotManager>, WriteStorage<Team>)| {
            let root_ent = entities.create();
            let root: CombatRoot = CombatRoot::new(combat_data.clone());
            let slot_manager: SlotManager = SlotManager::new();
            let principal: Principal = Principal::new();

            roots.insert(root_ent, root);
            slot_managers.insert(root_ent, slot_manager);
            principals.insert(root_ent, principal);
            transforms.insert(root_ent, Transform::default());

            self.combat_root = Some(root_ent);

            // Set up teams.
            let friendly_ent: Entity = entities.create();
            teams.insert(friendly_ent, Team::Friendly);
            parents.insert(friendly_ent, Parent { entity: root_ent });
            // Set transform.
            let mut friendly_transform: Transform = Transform::from(Vector3::new(0.0, 0.0, 30.0));
            transforms.insert(friendly_ent, friendly_transform);

            let enemy_ent: Entity = entities.create();
            teams.insert(enemy_ent, Team::Enemy);
            parents.insert(enemy_ent, Parent { entity: root_ent });
            // Set transform.
            let mut enemy_transform: Transform = Transform::from(Vector3::new(0.0, 0.0, -30.0));
            enemy_transform.append_rotation_y_axis(std::f32::consts::PI);
            transforms.insert(enemy_ent, enemy_transform);

            self.friendly_root = Some(friendly_ent);
            self.enemy_root = Some(enemy_ent);
        });

        // Add ui.
        let turn_notification_ent = world.entities().create();
        world.write_storage::<UiTurnNotification>().insert(turn_notification_ent, UiTurnNotification {
            team: Team::Friendly,
            root_ent: self.combat_root.unwrap(),
            precursor_text: "Turn: ".to_string(),
        });

        // Particle TEST
        //world.write_storage::<>

        // Set up camera.
        world.exec(|(entities, mut parents, mut transforms, mut cameras, mut combat_camera_tags, mut auto_fovs, dims): (Entities, WriteStorage<Parent>, WriteStorage<Transform>, WriteStorage<Camera>, WriteStorage<CombatCameraTag>, WriteStorage<AutoFov>, ReadExpect<ScreenDimensions>)| {

            // Fly pivot entity (the pivot point of the camera).
            let pivot_ent: Entity = entities.create();
            parents.insert(pivot_ent, Parent { entity: self.friendly_root.unwrap() });
            let mut pivot_transform = Transform::default();
            pivot_transform.set_rotation_x_axis(-std::f32::consts::FRAC_PI_8);
            transforms.insert(pivot_ent, pivot_transform);
            combat_camera_tags.insert(pivot_ent, CombatCameraTag);

            // The camera itself.
            let camera_ent: Entity = entities.create();
            let camera: Camera = Camera::from(Projection::perspective(
                dims.aspect_ratio(),
                crate::STD_FOV, // y FOV in radians
                0.1, // near
                -10000000.0, // far
            ));
            cameras.insert(camera_ent, camera);
            let mut camera_transform = Transform::from(Vector3::new(0.0, 0.0, 30.0));
            transforms.insert(camera_ent, camera_transform);
            parents.insert(camera_ent, Parent { entity: pivot_ent });
            // Auto fov.
            let mut auto_fov: AutoFov = AutoFov::new();
            auto_fov.set_base_aspect_ratio(dims.width() as usize, dims.height() as usize);
            auto_fov.set_min(crate::MIN_FOV * dims.aspect_ratio());
            auto_fov.set_max(crate::MAX_FOV * dims.aspect_ratio());
            auto_fov.set_base_fovx(crate::STD_FOV * dims.aspect_ratio());

            auto_fovs.insert(camera_ent, auto_fov);
        });

        // Spawn our master drone.
//        Character::spawn_to_world(
//            world,
//            self.friendly_root.unwrap(),
//            MasterDrone::character_id(),
//            Team::Friendly,
//            0,
//        );

        let current_state: CurrentState = *world.read_resource::<CurrentState>();

        let mut master_data: CharacterData = MasterDrone::data();
        master_data.max_health *= current_state.master_health_mul;
        master_data.max_charge *= current_state.master_charge_mul;
        master_data.artificial_charge *= current_state.master_charge_mul;
        // Spawn friendlies.
        Character::spawn_to_world(
            world,
            self.friendly_root.unwrap(),
            MasterDrone::character_id(),
            Some(master_data),
            Rank::Basic,
            Team::Friendly,
            0,
            true,
        );

        world.write_resource::<EventChannel<SpawnWaveEvent>>().single_write(
            SpawnWaveEvent {
                team_ent: self.enemy_root.unwrap(),
                idx: 0,
                wave: combat_data.waves().get(0).expect("The combat has no waves!").clone(),
            }
        );

        // Rebuild view.
        *world.write_resource::<RebuildRendering>() = RebuildRendering(true);
    }

    fn update(&mut self, data: StateData<AggregateData<'a, 'b>>) -> Trans<AggregateData<'a, 'b>, StateEvent> {

        let delta: f32 = data.world.read_resource::<Time>().delta_seconds();
        if let Some(mut exit_opt) = data.world.try_fetch_mut::<Option<ExitCombat>>() {
            let mut exit_idx: Option<(usize, Option<Team>)> = None;
            if let Some(exit) = exit_opt.as_mut() {
                exit.timer -= delta;
                if exit.timer <= 0.0 {
                    if let Some(point_idx) = self.point_idx {
                        exit_idx = Some((point_idx, exit.winner));
                    }
                }
            }
            if let Some((exit_idx, winner)) = exit_idx {
                *exit_opt = None;
                // Progress game.
                if let Some(mut current_state) = data.world.try_fetch_mut::<CurrentState>() {
                    if current_state.max_point == exit_idx && winner == Some(Team::Friendly) {
                        if let Some(slot_manager) = data.world.read_storage::<SlotManager>().get(self.combat_root.unwrap()) {
                            if let Some(master) = slot_manager.friendly.master() {
                                if let Some(character) = data.world.read_storage::<Character>().get(master) {
                                    current_state.master_health = character.relative_health();
                                }
                            }
                        }
                        if current_state.max_stage != MapStage::Complete {
                            current_state.max_stage = MapStage::PostDialogue;
                        }
                    }

                    save_current(&current_state);
                    return Trans::Switch(
                        Box::new(
                            MapState::new(Some(exit_idx)),
                        )
                    );
                }
            }
        }
        // During the play state all the systems must be available since we need UI rendering for HUD etc.
        data.data.dispatch_all(data.world);

        Trans::None
    }

    fn handle_event(
        &mut self,
        data: StateData<AggregateData<'a, 'b>>,
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
                if let InputEvent::KeyPressed { key_code: VirtualKeyCode::Escape, .. } = input {
                    data.world.write_resource::<EventChannel<UiDisengageEvent>>().single_write(
                        UiDisengageEvent::Cancel,
                    );
                }
                if let InputEvent::KeyPressed { key_code: VirtualKeyCode::W, .. } = input {
                    let mut has_cheats: bool = false;
                    if let Some(state) = data.world.try_fetch::<CurrentState>().clone() {
                        has_cheats = state.has_cheats;
                    }
                    if has_cheats {
                        data.world.insert(Some(ExitCombat { timer: 0.0, winner: Some(Team::Friendly) }));
                    }
                }
                if let InputEvent::KeyPressed { key_code: VirtualKeyCode::L, .. } = input {
                    let mut has_cheats: bool = false;
                    if let Some(state) = data.world.try_fetch::<CurrentState>().clone() {
                        has_cheats = state.has_cheats;
                    }
                    if has_cheats {
                        data.world.insert(Some(ExitCombat { timer: 0.0, winner: Some(Team::Enemy) }));
                    }
                }
                if let InputEvent::KeyPressed { key_code: VirtualKeyCode::H, .. } = input {
                    if let Some(dialogue) = data.world.fetch::<DialogueStore>().dialogue_list.get("small_tutorial").cloned() {
                        data.world.fetch_mut::<EventChannel<ShowDialogueDisplayEvent>>().single_write(
                            ShowDialogueDisplayEvent {
                                dialogue,
                                owner: Some(self.combat_root.unwrap()),
                                start_idx: 0,
                                principal: true,
                            }
                        );
                    }
                }
                Trans::None
            }
        }
    }
}
