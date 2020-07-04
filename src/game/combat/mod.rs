use std::any::TypeId;
use std::collections::HashMap;
use std::default::Default;

use amethyst::{
    core::{
        math::Isometry3,
        Parent,
        ParentHierarchy,
    },
    core::{
        SystemBundle,
        SystemDesc,
    },
    ecs::{
        prelude::*,
        storage::{
            GenericReadStorage,
            GenericWriteStorage,
        },
    },
    Error,
};

use crate::core::get_root;
use crate::game::character::CharacterId;
use crate::game::combat::ability::{AbilityTarget, AbilityTargetType};
use crate::game::combat::spawn::SlotManager;
use crate::game::ui::font::GameFonts;

#[macro_use]
pub mod ability;
pub mod status;
pub mod spawn;
pub mod tactical;
pub mod systems;
pub mod enemy;
pub mod process;
pub mod player;
pub mod ai;

pub struct CombatBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for CombatBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {

        // Load fonts.
        let fonts: GameFonts = GameFonts::load(world);
        world.insert(fonts);

        // Controller systems.
        builder.add(
            systems::standard_combat::StandardCombatSystemDesc::default()
                .build(world),
            "standard_combat",
            &[],
        );
        builder.add(
            crate::game::control::camera::combat::CombatCameraSystemDesc::new(
                0.1, 0.1, -std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, 0.1, 10.0, 80.0, -std::f32::consts::FRAC_PI_8, 0.0, false,
            )
                .build(world),
            "combat_camera",
            &[],
        );
        builder.add(
            crate::game::control::camera::combat::CameraDriftSystemDesc::new(10.0) // input camera movement speed
                .build(world),
            "camera_drift",
            &["combat_camera"],
        );
        builder.add(
            enemy::EnemyControllerSystemDesc::default()
                .build(world),
            "enemy_control",
            &["standard_combat"],
        );

        // HUD systems
        builder.add(
            crate::game::combat::player::CombatUiSystemDesc::default()
                .build(world),
            "combat_ui",
            &[],
        );
        builder.add(
            crate::game::combat::player::CrosshairUiControllerSystemDesc::default()
                .build(world),
            "crosshair_control_ui",
            &[],
        );
        builder.add(
            crate::game::ui::hud::UiCharacterBaseSystemDesc::default()
                .build(world),
            "character_ui",
            &["combat_ui"],
        );
        builder.add(
            crate::game::ui::ability::AbilitySelectSystemDesc::new(None)
                .build(world),
            "ability_ui",
            &["character_ui"],
        );
        builder.add(
            crate::game::ui::status::StatusUiSystemDesc::new(HashMap::new(), HashMap::new())
                .build(world),
            "status_ui",
            &["character_ui"],
        );
        builder.add(
            crate::game::ui::crosshair::CrosshairUiSystemDesc::new(HashMap::new())
                .build(world),
            "crosshair_ui",
            &["character_ui"],
        );
        builder.add(
            crate::game::ui::select_character::CharacterSelectSystemDesc::default()
                .build(world),
            "select_character",
            &["combat_ui"],
        );
        builder.add(
            crate::game::ui::description::DescriptionPanelSystemDesc::default()
                .build(world),
            "description_ui",
            &["combat_ui"],
        );
        builder.add(
            systems::target_select::SelectAbilityTargetSystemDesc::default()
                .build(world),
            "select_ability_target",
            &["standard_combat"],
        );
        builder.add(
            crate::game::ui::select_all_button::SelectAllUiSystemDesc::default()
                .build(world),
            "select_all_button",
            &["standard_combat"],
        );
        builder.add(
            crate::game::ui::marker::MarkerUiSystemDesc::default()
                .build(world),
            "marker",
            &["standard_combat"],
        );
        builder.add(
            crate::game::ui::turn_notification::TurnNotificationUiSystemDesc::default()
                .build(world),
            "turn_notificaion",
            &["standard_combat"],
        );
        builder.add(
            crate::game::ui::hack::HackUiSystemDesc::new(None, None)
                .build(world),
            "hack_ui",
            &["standard_combat"],
        );
        builder.add(
            crate::game::ui::banner::BannerUiSystemDesc::default()
                .build(world),
            "banner_ui",
            &["standard_combat"],
        );
        builder.add(
            crate::game::ui::select_rank::SelectRankUiSystemDesc::new(HashMap::new(), None)
                .build(world),
            "select_rank_ui",
            &["standard_combat"],
        );
        builder.add(
            ai::spacebot::SpacebotAiSystemDesc::default()
                .build(world),
            "ai_spacebot",
            &["standard_combat"],
        );
        builder.add(
            ai::guardian::GuardianAiSystemDesc::default()
                .build(world),
            "ai_guardian",
            &["standard_combat"],
        );

        // Ability Systems
        builder.add(
            ability::charge::ChargeAbilitySystemDesc::new(None)
                .build(world),
            "ability_charge",
            &["enemy_control"],
        );
        builder.add(
            ability::spawn::SpawnAbilitySystemDesc::default()
                .build(world),
            "ability_spawn",
            &["enemy_control"],
        );
        builder.add(
            ability::hack::HackAbilitySystemDesc::default()
                .build(world),
            "ability_hack",
            &["enemy_control"],
        );
        builder.add(
            ability::focused_charge::FocusedChargeAbilitySystemDesc::default()
                .build(world),
            "ability_focused_charge",
            &["enemy_control"],
        );
        builder.add(
            ability::nanobots::NanobotsAbilitySystemDesc::default()
                .build(world),
            "ability_nanobots",
            &["enemy_control"],
        );
        builder.add(
            ability::twin_shot::TwinShotAbilitySystemDesc::default()
                .build(world),
            "ability_twin_shot",
            &["enemy_control"],
        );
        builder.add(
            ability::snipe::SnipeAbilitySystemDesc::default()
                .build(world),
            "ability_snipe",
            &["enemy_control"],
        );
        builder.add(
            ability::barrage::BarrageAbilitySystemDesc::default()
                .build(world),
            "ability_barrage",
            &["enemy_control"],
        );
        builder.add(
            ability::barrage::HyperBarrageAbilitySystemDesc::default()
                .build(world),
            "ability_hyper_barrage",
            &["enemy_control"],
        );
        builder.add(
            ability::annihilate::AnnihilateAbilitySystemDesc::default()
                .build(world),
            "ability_annihilate",
            &["enemy_control"],
        );
        builder.add(
            ability::big_bullet::BigBulletAbilitySystemDesc::default()
                .build(world),
            "ability_big_bullet",
            &["enemy_control"],
        );
        builder.add(
            ability::scrambling_laser::ScramblingLaserAbilitySystemDesc::default()
                .build(world),
            "ability_scrambling_laser",
            &["enemy_control"],
        );
        builder.add(
            ability::overclock::OverclockAbilitySystemDesc::default()
                .build(world),
            "ability_overclock",
            &["enemy_control"],
        );
        builder.add(
            ability::overclock::SystemOverclockAbilitySystemDesc::default()
                .build(world),
            "ability_system_overclock",
            &["enemy_control"],
        );
        builder.add(
            ability::shock::ShockAbilitySystemDesc::default()
                .build(world),
            "ability_shock",
            &["enemy_control"],
        );
        builder.add(
            ability::solid_laser::SolidLaserAbilitySystemDesc::default()
                .build(world),
            "ability_solid_laser",
            &["enemy_control"],
        );
        builder.add(
            ability::jammer::JammerAbilitySystemDesc::default()
                .build(world),
            "ability_jammer",
            &["enemy_control"],
        );
        builder.add(
            ability::nuke::NukeAbilitySystemDesc::default()
                .build(world),
            "ability_nuke",
            &["enemy_control"],
        );
        builder.add(
            ability::overload::EnergyOverloadAbilitySystemDesc::default()
                .build(world),
            "ability_overload",
            &["enemy_control"],
        );
        builder.add(
            ability::corrupt::CorruptAbilitySystemDesc::default()
                .build(world),
            "ability_corrupt",
            &["enemy_control"],
        );
        builder.add(
            ability::sheild::ReinforceAbilitySystemDesc::default()
                .build(world),
            "ability_reinforce",
            &["enemy_control"],
        );
        builder.add(
            ability::sheild::ShieldAbilitySystemDesc::default()
                .build(world),
            "ability_shield",
            &["enemy_control"],
        );
        builder.add(
            ability::empower::EmpowerAbilitySystemDesc::default()
                .build(world),
            "ability_empower",
            &["enemy_control"],
        );
        builder.add(
            ability::focus::FocusAbilitySystemDesc::default()
                .build(world),
            "ability_focus",
            &["enemy_control"],
        );
        builder.add(
            ability::annihilate::AnnihilatePlusAbilitySystemDesc::default()
                .build(world),
            "ability_annihilate_plus",
            &["enemy_control"],
        );
        builder.add(
            ability::retribution::RetributionAbilitySystemDesc::default()
                .build(world),
            "ability_retribution",
            &["enemy_control"],
        );
        builder.add(
            ability::self_destruct::SelfDestructAbilitySystemDesc::default()
                .build(world),
            "ability_self_destruct",
            &["enemy_control"],
        );
//        builder.add(
//            ability::upgrade::UpgradeAbilitySystemDesc::default()
//                .build(world),
//            "ability_upgrade",
//            &["enemy_control"],
//        );



        builder.add_barrier();
        // Enemy Ability invocation
        builder.add(
            enemy::EnemyAbilityInvocationSystemDesc::default()
                .build(world),
            "enemy_ability_invocation",
            &["enemy_control"],
        );

        // Process systems.
        builder.add(
            systems::dmg::DmgSystemDesc::default()
                .build(world),
            "dmg",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            systems::buff::BuffSystemDesc::default()
                .build(world),
            "buff",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            spawn::SpawnInvokeSystemDesc::default()
                .build(world),
            "spawn_invoke",
            &[],
        );
        builder.add(
            spawn::SpawnSystemDesc::default()
                .build(world),
            "spawn",
            &["enemy_control", "spawn_invoke"],
        );
        builder.add(
            systems::fire_bullet::FireBulletSystemDesc::new(None)
                .build(world),
            "fire_bullet",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            systems::fire_torpedo::FireTorpedoSystemDesc::new(None)
                .build(world),
            "fire_torpedo",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            systems::fire_laser::FireLaserSystemDesc::new(None)
                .build(world),
            "fire_laser",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            systems::fire_wave::FireWaveSystemDesc::new(None)
                .build(world),
            "fire_wave",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            systems::kill::CharacterDefeatSystemDesc::default()
                .build(world),
            "character_defeat",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            systems::delay::DelaySystemDesc::default()
                .build(world),
            "delay",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            systems::enemy_wave::EnemyWaveSystemDesc::default()
                .build(world),
            "enemy_wave",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            status::StatusSystemDesc::default()
                .build(world),
            "status",
            &["standard_combat", "enemy_control"],
        );
        builder.add(
            systems::earth_combat::EarthCombatSystemDesc::default()
                .build(world),
            "earth_combat",
            &["spawn"],
        );
        builder.add(
            ability::PerformingAbilitySystemDesc::default()
                .build(world),
            "perform_ability",
            &[],
        );


        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Extreme,
}

impl Difficulty {
    pub fn level(&self) -> u32 {
        match self {
            Difficulty::Easy => 0,
            Difficulty::Normal => 1,
            Difficulty::Hard => 2,
            Difficulty::Extreme => 3,
        }
    }

    pub fn next(&self) -> Difficulty {
        match self {
            Difficulty::Easy => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Extreme,
            Difficulty::Extreme => Difficulty::Easy,
        }
    }

    pub fn previous(&self) -> Difficulty {
        match self {
            Difficulty::Easy => Difficulty::Extreme,
            Difficulty::Normal => Difficulty::Easy,
            Difficulty::Hard => Difficulty::Normal,
            Difficulty::Extreme => Difficulty::Hard,
        }
    }
}

impl ToString for Difficulty {
    fn to_string(&self) -> String {
        match self {
            Difficulty::Easy => "Easy".to_string(),
            Difficulty::Normal => "Normal".to_string(),
            Difficulty::Hard => "Hard".to_string(),
            Difficulty::Extreme => "Extreme".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatData {
    /// The name of the combat (internal).
    name: &'static str,

    /// The system to be used.
    system: TypeId,

    /// The enemy data used by the default combat system.
    /// If a custom system is used, this could be set to `None` if the system spawns the enemies.
    waves: Vec<Wave>,

    /// The prefab path of the combat data.
    prefab_path: &'static str,
}

impl CombatData {
    pub fn basic(name: &'static str, prefab_path: &'static str, waves: Vec<Wave>) -> Self {
        Self {
            name,
            system: TypeId::of::<systems::standard_combat::StandardCombatSystem>(),
            waves,
            prefab_path,
        }
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    #[inline]
    pub fn prefab_path(&self) -> &'static str {
        self.prefab_path
    }

    #[inline]
    pub fn system(&self) -> TypeId {
        self.system
    }

    #[inline]
    pub fn waves(&self) -> &[Wave] {
        self.waves.as_slice()
    }

    #[inline]
    pub fn poster(&self) -> CharacterId {
        self.waves[0].characters[0].character_id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Wave {
    /// This is where a boss would go.
    pub master: Option<CharacterSpawn>,
    /// The 'ordinary' enemies.
    pub characters: Vec<CharacterSpawn>,
}

impl Wave {
    pub fn boss(character: CharacterSpawn) -> Self {
        Self {
            master: Some(character),
            characters: Vec::new(),
        }
    }
    pub fn new_simple(characters: Vec<CharacterSpawn>) -> Self {
        Self {
            master: None,
            characters,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Rank {
    Basic,
    Advanced,
    Elite,
    Legendary,
}

impl Rank {
    pub fn try_upgrade(&mut self) -> bool {
        match self {
            Rank::Basic => *self = Rank::Advanced,
            Rank::Advanced => *self = Rank::Elite,
            Rank::Elite => *self = Rank::Legendary,
            Rank::Legendary => return false,
        }
        true
    }

    pub fn base_multiplier(&self) -> f32 {
        match self {
            Rank::Basic => 1.0,
            Rank::Advanced => 1.2,
            Rank::Elite => 1.4,
            Rank::Legendary => 1.6,
        }
    }

    pub fn accuracy_multiplier(&self) -> f32 {
        match self {
            Rank::Basic => 1.0,
            Rank::Advanced => 1.5,
            Rank::Elite => 2.0,
            Rank::Legendary => 2.5,
        }
    }

    pub fn evade_multiplier(&self) -> f32 {
        match self {
            Rank::Basic => 1.0,
            Rank::Advanced => 1.5,
            Rank::Elite => 2.0,
            Rank::Legendary => 2.5,
        }
    }

    pub fn health_multiplier(&self) -> f32 {
        match self {
            Rank::Basic => 1.0,
            Rank::Advanced => 1.75,
            Rank::Elite => 2.5,
            Rank::Legendary => 3.0,
        }
    }

    pub fn charge_multiplier(&self) -> f32 {
        match self {
            Rank::Basic => 1.0,
            Rank::Advanced => 1.5,
            Rank::Elite => 2.0,
            Rank::Legendary => 2.5,
        }
    }
}

impl ToString for Rank {
    fn to_string(&self) -> String {
        match self {
            Rank::Basic => "Basic".to_string(),
            Rank::Advanced => "Advanced".to_string(),
            Rank::Elite => "Elite".to_string(),
            Rank::Legendary => "Legendary".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CharacterSpawn {
    pub character_id: CharacterId,
    pub rank: Rank,
}

impl CharacterSpawn {
    pub fn new(character_id: CharacterId, rank: Rank) -> Self {
        Self {
            character_id,
            rank,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Team {
    Friendly,
    Enemy,
}

impl Component for Team {
    type Storage = DenseVecStorage<Self>;
}

impl Team {
    pub fn other(&self) -> Self {
        match self {
            Team::Friendly => Team::Enemy,
            Team::Enemy => Team::Friendly,
        }
    }

    pub fn get_team<'s>(
        parents: &impl GenericReadStorage<Component=Parent>, teams: &impl GenericReadStorage<Component=Team>,
        entity: Entity,
    ) -> Option<(Team, Entity)> {
        if let Some(team) = teams.get(entity).copied() {
            Some((team, entity))
        } else {
            let parent_ent: Entity = parents.get(entity)?.entity;
            Self::get_team(
                parents, teams,
                parent_ent,
            )
        }
    }

    pub fn get_local<'s>(
        entities: &Entities<'s>,
        parents: &impl GenericReadStorage<Component=Parent>,
        roots: &impl GenericReadStorage<Component=CombatRoot>,
        teams: &impl GenericReadStorage<Component=Team>,
        hierarchy: &ParentHierarchy,
        team: Team,
        entity: Entity,
    ) -> Option<Entity> {
        if let Some((_, root_ent)) = get_root::<CombatRoot, _, _>(parents, roots, entity) {
            for (entity, _) in (entities,  hierarchy.all_children(root_ent)).join() {
                if let Some(other_team) = teams.get(entity) {
                    if team == *other_team {
                        return Some(entity);
                    }
                }
            }
        }
        None
    }

    pub fn is_target_for(&self, target: &AbilityTargetType) -> bool {
        match target {
            AbilityTargetType::Enemy => *self == Team::Enemy,
            AbilityTargetType::Friendly => *self == Team::Friendly,
            AbilityTargetType::All => true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CombatState {
    Init,
    InTurn(Team),
    DoneTurn(Team),
    Victory(Team),
}

/// The tag which represents the root object of the combat.
#[derive(Debug, Clone)]
pub struct CombatRoot {
    pub data: CombatData,
    pub current_state: CombatState,
    pub turn_count: i32,
    pub current_wave: usize,
}

impl CombatRoot {
    pub fn new(data: CombatData) -> Self {
        Self {
            data,
            current_state: CombatState::Init,
            turn_count: 0,
            current_wave: 0,
        }
    }
    /// Searches for the combat root of the specified entity.
    /// This function is called recursively called until either a `CombatRoot` is located or we reach a top level entity in the hierarchy.
    pub fn get_root<'s>(
        parents: &impl GenericReadStorage<Component=Parent>, combat_roots: &impl GenericReadStorage<Component=CombatRoot>,
        entity: Entity,
    ) -> Option<Entity> {
        if combat_roots.get(entity).is_some() {
            Some(entity)
        } else {
            let parent_ent: Entity = parents.get(entity)?.entity;
            Self::get_root(
                parents, combat_roots,
                parent_ent,
            )
        }
    }
}

impl Component for CombatRoot {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TickTurn {
    pub count: i32,
    pub next_team: Team,
}
