use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::Character;
use crate::game::character::*;

use super::*;

define_ability!(
    ShockAbility,
    ShockAbilitySystem,
    ShockAbilitySystemDesc,
    "Shock",
    "Scramble an enemy robot. High Accuracy.",
    80.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[ModelXDrone::character_id()]),
    [
        AbilityActionDesc::Fire(FireDesc::laser(5.0, Element::Plasma, 0.8, 0.2), 0),
        AbilityActionDesc::Wait(0.4),
        AbilityActionDesc::Fire(FireDesc::laser(5.0, Element::Plasma, 0.8, 0.2), 0),
        AbilityActionDesc::Wait(0.4),
        AbilityActionDesc::Fire(
           FireDesc {
                ty: FireType::Laser(Element::Plasma, 0.2, false),
                accuracy: 0.7,
                power: 10.0,
                effect: Some (
                    StatusInflictDesc {
                        turns: 1,
                        ty: StatusType::Scramble,
                        chance: 1.0,
                    }
                ),
           },
           0,
       ),
       AbilityActionDesc::Wait(0.7)
    ]
);