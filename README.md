# Abilities
**Main Rocket**
* Attack - fires the master drone's primary weapon.
* Summon Drone - only works if less than 3 drones on battlefield.
* Charge - charges energy banks for ultimate attack.

(All options display a select ui, except Charge and Flee)

Attack: Opens up a menu of rocket attacks. 
Laser - Expends 5% charge, deals moderate damage to a target, element is random. 
Imbine - Expends 5% charge, changes a drone's element (Cannot change drone to Quantum). 
Missile Barrage - Expends 10% charge, deals damage to all enemies. 
Gunner Drone

Abilities are represented by distinct entities. Each ability entity is a child of the `Character` entity which possesses it.\
Abilities can be added to characters using the `populate_abilities` method.\
The `Ability` struct represents an instance of an ability, which keeps track of any cooldowns, uses etc.
The `AbilityData` struct contains 'static/blueprint' data which is specific per ability but does not change during the lifecycle of the ability (e.g. `name`, `desc`, `charge` etc.)

Each ability has an `AbilityUsability` object. This describes which characters can use each ability.
````rust
pub enum AbilityUsability {
    All,
    Unique(CharacterId),
    Role(CharacterRole),
}
````
Abilities are stored inside a `HashMap<AbilityUsability, AbilityData>` object, which allows them to be grouped depending on their usability.
# Characters
* Master drone

**Slave Drones**
* Blitz (aggressive - kinetic and plasma moves)
* Spangles (more balanced - kinetic and ion moves as well as one quantum)
* Wee Lauren (support - repair, buff, hack and one radiation move)
* Sparky (specialist - moves which inflict status effects)

Enemy bosses will have the same model as the Main Rocket, however a different name for each of them and a new condition on the fight that the player doesn't have.

**Bosses**
* The Guardian

Characters are classified by their `CharacterRole` as follows:
````rust
pub enum CharacterRole {
    Master,
    Slave,
    Enemy,
    Boss,
}
````
Note: role does not necessarily describe what team the character is on (e.g. the character could be hacked but the role would remain the same).

Each character is uniquely identified by a `ChararacterId` object. This should simply be assigned to the `TypeId` of the character's unique component type.
````rust
pub type CharacterId = std::any::TypeId;
````
# Elements and Effects
* Kinetic – A base type element with basic attacks. Kinetic based attacks have an accuracy debuff associated with them. Kinetic attacks may cause enemies to miss next turn. 2nd highest base power.\
In game explanation: Kinetic attacks hit enemy drones so hard that their systems are scrambled and can’t aim well.
* Plasma – A fiery element that causes damage over time. 3rd highest base power.\
In game explanation: Plasma clings to anything it touches, it may turn invisible, but the heat slowly damages affected drones.
* Ion – An electrical element with low base power. 5th highest base power. Ion attacks may stun enemy drones.\
In game explanation: A shock to any system, Ion might totally disable an enemy drone by disabling its controls. The effect only lasts a turn though.
* Radiation – An invisible element that exposes drones. Causes defence debuffs. 4th highest base power.\
In game explanation: Radiation attacks are invisible but are no less useful. They penetrate Armor and find weaknesses in it, opening it up to more damage
* Quantum – The most powerful element. Highest base power with unpredictable and varying special effects.\
In game explanation: Quantum attacks allow for more damage than any other element! It is very rare.

# Control Flow
The game logic is control flow is managed using a `Principal` object with is stored as a component to the combat root (`CombatRoot`) entity.
If an action is carried out, the principal should be immediately set to the entity linked to the action.
Once any action that could be principal is done, it should be checked against the `Principal` component and set to `None` if it is the principal.
While this requires more manual work for the invoker, it does prevent conditions in which two principal actions are triggered in the same frame and are both processed later, causing one not to be executed (UB).

There are some things to keep in mind when using this system:
* Check current principal before invoking.
* If clear, set the principal to the 'actionable entity' - the entity which the action is performed on.
* Ensure that the system sets the principal to `None` once the action has finished, if the system is processing the current principal. If this is not done it could easily cause a 'soft lock' where a principal is never removed and is constantly waited on.

**Execution Order**

Correct execution order of systems is crucial to maintain efficiency and proper function.
The execution order of various types of systems is as follows:
* Core Combat
* Core control (user/enemy controller)
* Characters
* Abilities
* AI ability perform


# Enemy AI
Enemy abilities are chosen via a points based system, in which each ability is given a certain scalar rating value.
The Ability with the highest AI value gets chosen to be performed. There is possibility for more complex planning by adding various parameters to the AI selection components.
The AI choice logic is as follows:
* A particular character is chosen to perform an ability based on whether it has the `Spent` component (if so, no ability is performed - it may have already performed an ability this turn).
* An `AiAbilitySelectionQuery` component is added to each usable `Ability` entity.
* The system controlling the `Ability` can respond by setting the `score` field of the resultant `AiAbilitySelection` to the desired relative score (e.g. `Some(0.4)` if the rating was 0.4).
* The `Ability`/`AiAbilitySelectionQuery` entity with the highest rating will be performed.
* If there are equal scores, the first ability is chosen from those with the highest equal scores.

This implementation allows an incredibly powerful, customizable and modular approach, since each ability can customize the AI associated with it. Abilities can easily be removed without the AI breaking.
However there are some things to take into account.
* Since the ability scores are all compared relative to each other, there needs to be some accepted 'standard' to ensure that one ability is not constantly yielding a higher score than the rest.
* There are quite a lot of 'moving parts' - bugs/exploits could easily slip in as there would be many untestable conditions.
* It requires a discrete AI implementation for each ability.
* There could be a kind of 'race condition' where two abilities believe that they are ideal and both provide very high `score` values.

# Combat
For the player, abilities are selected using the selecion ui. A popup 'window' of all the abilities will appear when the player clicks on the character selection area.
The abilities are rendered from bottom up as a list. The 'window' floats with the main character UI root (it is a child entity of the character).
Once an ability is selected, the `AbilityTargetSystem` is triggered with the specified ability. This should occur as principal. This is removed once a target is selected, and the ability perform is invoked as principal.
Once the ability is triggered, the count stored by the team control system is incremented. Once a certain number of abilities per turn have been performed (3 by default), the `CombatRoot` is alerted, and the main combat system will then yield to the enemy controller.
