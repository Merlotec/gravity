use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    AnnihilateAbility,
    AnnihilateAbilitySystem,
    AnnihilateAbilitySystemDesc,
    "Annihilate",
    "Instantly destroys an enemy drone. Very Low Accuracy.",
    1000.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[MasterDrone::character_id()]),
    [
       AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.4),
       AbilityActionDesc::Fire(FireDesc::laser(500.0, Element::Plasma, 0.2, 2.0),0),
       AbilityActionDesc::Wait(2.0)
    ]
);

define_ability!(
    AnnihilatePlusAbility,
    AnnihilatePlusAbilitySystem,
    AnnihilatePlusAbilitySystemDesc,
    "Annihilate +",
    "Releases all the energy of the antimatter reactor at once...",
    1000.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[EarthCharacter::character_id()]),
    [
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.05, None),
        AbilityActionDesc::Wait(0.05),
        AbilityActionDesc::Wave(0, 0.05, None),
        AbilityActionDesc::Wait(0.05),
        AbilityActionDesc::Wave(0, 0.05, None),
        AbilityActionDesc::Wait(0.05),
        AbilityActionDesc::Wave(0, 0.05, None),
        AbilityActionDesc::Wait(0.05),
        AbilityActionDesc::Wave(0, 0.05, None),
        AbilityActionDesc::Wait(0.05),
        AbilityActionDesc::Wave(0, 0.05, None),
        AbilityActionDesc::Wait(0.7),
       AbilityActionDesc::Fire(FireDesc::laser(999999999999999.0, Element::Plasma, 1.0, 5.0),0),
       AbilityActionDesc::Wait(5.0)
    ]
);