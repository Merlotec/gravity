use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::Character;
use crate::game::character::*;

use super::*;

define_ability!(
    SolidLaserAbility,
    SolidLaserAbilitySystem,
    SolidLaserAbilitySystemDesc,
    "Solid Laser",
    "Strikes a single target with a laser. May cause Unstable",
    50.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[SparkyDrone::character_id(),
    ChargeSpacebotDrone::character_id(),
    ModelXDrone::character_id(),
    ]),
    [
        AbilityActionDesc::Fire(
           FireDesc {
                ty: FireType::Laser(Element::Plasma, 1.3, false),
                accuracy: 0.75,
                power: 70.0,
                effect: Some (
                    StatusInflictDesc {
                        turns: 1,
                        ty: StatusType::Unstable,
                        chance: 0.5,
                    }
                ),
           },
           0,
       ),
        AbilityActionDesc::Wait(1.5)
    ]
);