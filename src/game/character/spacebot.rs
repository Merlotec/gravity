use std::any::TypeId;

use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::{Character, CharacterData, CharacterId, CharacterPrefabData, CharacterRole, Stats, UnassignedCharacter};

use super::*;

define_character!(
    GunnerSpacebotDrone,
    GunnerSpacebotDroneSystem,
    GunnerSpacebotDroneSystemDesc,
    "object/character/enemy/gunner_spacebot/gunner_spacebot.ron",
    "GunnerBot",
    "An gunner variation of the standard enemy drone.",
    100.0,
    150.0,
    20.0,
    5.0,
    20.0,
    Some(1.2),
    CharacterRole::Slave,
    Team::Enemy,
    1,
    1.0,
);

define_character!(
    SpacebotDrone,
    SpacebotDroneSystem,
    SpacebotDroneSystemDesc,
    "object/character/enemy/balanced_spacebot/balanced_spacebot.ron",
    "SpaceBot",
    "The basic balanced enemy spacebot.",
    100.0,
    150.0,
    20.0,
    5.0,
    20.0,
    Some(1.2),
    CharacterRole::Slave,
    Team::Enemy,
    1,
    1.0,
);

define_character!(
    ChargeSpacebotDrone,
    ChargeSpacebotDroneSystem,
    ChargeSpacebotDroneSystemDesc,
    "object/character/enemy/charge_spacebot/charge_spacebot.ron",
    "ChargeBot",
    "The charge enemy spacebot.",
    100.0,
    150.0,
    20.0,
    5.0,
    20.0,
    Some(1.2),
    CharacterRole::Slave,
    Team::Enemy,
    1,
    1.0,
);

define_character!(
    SupporterSpacebotDrone,
    SupporterSpacebotDroneSystem,
    SupporterSpacebotDroneSystemDesc,
    "object/character/enemy/supporter_spacebot/supporter_spacebot.ron",
    "SupportBot",
    "The supporter enemy bot.",
    100.0,
    150.0,
    20.0,
    5.0,
    20.0,
    Some(1.2),
    CharacterRole::Slave,
    Team::Enemy,
    1,
    1.0,
);

define_character!(
    ModelXDrone,
    ModelXDroneSystem,
    ModelXDroneSystemDesc,
    "object/character/enemy/model_x/model_x.ron",
    "Model X",
    "A superelite version of the standard spacebot.",
    300.0,
    200.0,
    150.0,
    10.0,
    40.0,
    Some(2.8),
    CharacterRole::Independent,
    Team::Enemy,
    2,
    1.2,
);