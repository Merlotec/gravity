use super::*;

define_character!(
    DefenderDrone,
    DefenderDroneSystem,
    DefenderDroneSystemDesc,
    "object/character/defender/defender.ron",
    "Defender",
    "A support drone.",
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