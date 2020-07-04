use amethyst::{
    assets::{
        AssetPrefab,
        PrefabData,
        ProgressCounter,
    },
    core::{
        Transform,
        math::{
            Vector2,
            Vector3,
        },
        SystemBundle,
        SystemDesc,
        Named,
    },
    ecs::{
        prelude::*,
        storage::GenericReadStorage,
    },
    Error,
    gltf::{
        GltfSceneAsset,
        GltfSceneFormat,
    },
    renderer::{
        formats::GraphicsPrefab,
        light::LightPrefab,
        rendy::mesh::{
            Normal,
            Position,
            Tangent,
            TexCoord,
        },
        rendy::mesh::MeshBuilder,
        shape::FromShape,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use space_render::{
    Atmosphere,
    Star,
};

use crate::game::combat::{CombatData, Wave, CharacterSpawn, Rank, Difficulty};
use std::collections::HashMap;

use crate::game::character::spacebot::{
    GunnerSpacebotDrone,
    ModelXDrone,
};
use crate::game::ui::dialogue::{Dialogue, DialogueSegment, DialogueText};
use std::fs::File;

pub mod systems;
pub mod dialogues;
pub mod combats;

pub struct MapBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for MapBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        // Controller systems.
        builder.add(
            systems::movement::MapMovementSystemDesc::default()
            .build(world),
        "map_movement",
        &[],
        );
        builder.add(
            systems::control::MapControlSystemDesc::default()
                .build(world),
            "map_control",
            &[],
        );
        builder.add(
            crate::game::ui::map_notification::MapNotificationUiSystemDesc::default()
                .build(world),
            "map_notification_ui",
            &[],
        );
        builder.add(
            systems::terminate::TerminateSystemDesc::default()
                .build(world),
            "map_terminate",
            &[],
        );

        world.insert(combats::combats());

        world.insert(load_current());

        let dialogues = dialogues::dialogues(world);
        world.insert(dialogues);

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum MapStage {
    PreDialogue,
    Combat,
    PostDialogue,
    Complete,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentState {
    pub max_point: usize,
    pub max_stage: MapStage,
    pub master_charge_mul: f32,
    pub master_health_mul: f32,
    pub master_health: f32,

    pub difficulty: Difficulty,
    pub has_cheats: bool,
}

impl Default for CurrentState {
    fn default() -> Self {
        Self {
            max_point: 0,
            max_stage: MapStage::PreDialogue,
            master_charge_mul: 1.0,
            master_health_mul: 1.0,
            master_health: 1.0,

            difficulty: Difficulty::Normal,
            has_cheats: true,
        }
    }
}

#[derive(Serialize, Deserialize, PrefabData)]
#[serde(deny_unknown_fields)]
pub struct WorldPrefabData {
    //pub graphics: Option<GraphicsPrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>, Vec<Tangent>)>>,
    pub gltf: Option<AssetPrefab<GltfSceneAsset, GltfSceneFormat>>,
    pub transform: Option<Transform>,
    pub light: Option<LightPrefab>,
    pub atmosphere: Option<Atmosphere>,
    pub star: Option<Star>,
}

#[derive(Serialize, Deserialize, PrefabData)]
#[serde(deny_unknown_fields)]
pub struct MapPrefabData {
    //pub graphics: Option<GraphicsPrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>, Vec<Tangent>)>>,
    pub map_point: Option<MapPoint>,
    pub gltf: Option<AssetPrefab<GltfSceneAsset, GltfSceneFormat>>,
    pub transform: Option<Transform>,
    pub light: Option<LightPrefab>,
    pub atmosphere: Option<Atmosphere>,
    pub star: Option<Star>,
    pub name: Option<Named>,
}

#[derive(Debug, Clone, PartialEq, Component, Serialize, Deserialize, PrefabData)]
#[prefab(Component)]
pub struct MapPoint {
    pub index: usize,
    pub position: [f32; 3],
    pub combat: Option<String>,
    pub pre_dialogue: Option<String>,
    pub post_dialogue: Option<String>,
}

impl MapPoint {

    pub fn position_of<'s>(
        points: &ReadStorage<'s, MapPoint>,
        index: usize,
    ) -> Option<Vector3<f32>> {
        for point in points.join() {
            if point.index == index {
                return Some(point.position.into());
            }
        }
        None
    }

    pub fn combat<'s, 'a>(
        points: &ReadStorage<'s, MapPoint>,
        combat_store: &'a CombatStore,
        index: usize,
    ) -> Option<&'a CombatData> {
        for point in points.join() {
            if point.index == index {
                if let Some(combat) = &point.combat {
                    return combat_store.combat_list.get(combat);
                }
            }
        }
        None
    }

    pub fn name_of<'s>(
        points: &ReadStorage<'s, MapPoint>,
        names: &ReadStorage<'s, Named>,
        index: usize,
    ) -> Option<String> {
        for (point, name) in (points, names).join() {
            if point.index == index {
                return Some(name.name.to_string());
            }
        }
        None
    }

    pub fn pre_dialogue<'s, 'a>(
        points: &ReadStorage<'s, MapPoint>,
        dialogue_store: &'a DialogueStore,
        index: usize,
    ) -> Option<&'a Dialogue> {
        for point in points.join() {
            if point.index == index {
                if let Some(dialogue) = &point.pre_dialogue {
                    return dialogue_store.dialogue_list.get(dialogue);
                }
            }
        }
        None
    }

    pub fn post_dialogue<'s, 'a>(
        points: &ReadStorage<'s, MapPoint>,
        dialogue_store: &'a DialogueStore,
        index: usize,
    ) -> Option<&'a Dialogue> {
        for point in points.join() {
            if point.index == index {
                if let Some(dialogue) = &point.post_dialogue {
                    return dialogue_store.dialogue_list.get(dialogue);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct EngageCombat {
    pub combat_data: CombatData,
    pub point_idx: usize,
}

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct MapPawn(pub bool);

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct MapRoot {
    pub point_idx: usize,
    pub pawn_ent: Entity,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CombatStore {
    pub combat_list: HashMap<String, CombatData>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DialogueStore {
    pub dialogue_list: HashMap<String, Dialogue>,
}

pub fn load_current() -> CurrentState {
    if let Ok(mut savedata) = std::fs::read_to_string("save.json") {
        if let Ok(state) = serde_json::from_str(&savedata) {
            println!("Loaded savefile from `save.json`");
            return state;
        }
    }
    println!("Creating new savefile...");
    CurrentState::default()
}

pub fn save_current(data: &CurrentState) {
    if let Ok(savedata) = serde_json::to_string(data) {
        std::fs::write("save.json", savedata);
    }
}