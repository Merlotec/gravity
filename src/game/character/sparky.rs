use std::any::TypeId;

use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::{Character, CharacterData, CharacterId, CharacterPrefabData, CharacterRole, Stats, UnassignedCharacter};

use super::*;

define_character!(
    SparkyDrone,
    SparkyDroneSystem,
    SparkyDroneSystemDesc,
    "object/character/sparky/sparky.ron",
    "Sparky",
    "An all rounder.",
    100.0,
    150.0,
    20.0,
    5.0,
    20.0,
    Some(1.0),
    CharacterRole::Slave,
    Team::Friendly,
    1,
    1.0,
);