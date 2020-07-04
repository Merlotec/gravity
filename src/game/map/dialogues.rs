use amethyst::{
    ecs::World,
    assets::Loader,
    assets::{
        AssetStorage,
    },
    renderer::{
        Texture,
        ImageFormat,
    },
};
use crate::game::map::DialogueStore;
use crate::game::ui::dialogue::{Dialogue, DialogueSegment};
use amethyst::prelude::WorldExt;

pub fn dialogues(world: &mut World) -> DialogueStore {

    if !world.has_value::<AssetStorage::<Texture>>() {
        world.insert(AssetStorage::<Texture>::new());
    }

    let loader = world.read_resource::<Loader>();

    let scramble_texture = loader.load(
        "ui/scramble.png",
        ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    );
    let overclock_texture =  loader.load(
        "ui/overclocked.png",
        ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    );
    let unstable_texture = loader.load(
        "ui/unstable.png",
        ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    );
    let defend_texture = loader.load(
        "ui/defend.png",
        ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    );
    let empower_texture = loader.load(
        "ui/empower.png",
        ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    );
    let focus_texture = loader.load(
        "ui/focus.png",
        ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    );

    let mut dialogues: DialogueStore = DialogueStore::default();

    dialogues.dialogue_list.insert("intro". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Dev", "In the Far Future, Earth has been taken over by rouge AI..."),
                                           DialogueSegment::new("Dev", "Humanity was forced to retreat into the depths of space as the AI slowly but surely took over the Solar System. Now the AI seeks to destroy Humanity."),
                                           DialogueSegment::new("Dev", "In a last effort to retake Earth, the Avenger was developed. This highly experimental rocket must retake the Solar System before Humanity's hideout is discovered."),
                                           DialogueSegment::new("Dev", "You are the pilot of the Avenger! Take the Solar System back! One planet at a time."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("tutorial". to_string(),
                                   Dialogue {
                                       segments: vec![
               DialogueSegment::new("Dev", "Welcome to Gravity!"),
               DialogueSegment::new("Dev", "You can move around the World Map with the right and left arrow keys."),
               DialogueSegment::new("Dev", "You can move the camera around with the right mouse click. Use the scroll wheel to zoom in and out. Clicking on a green crosshair brings up the attack menu."),
               DialogueSegment::new("Dev", "All actions in this game need \"Charge.\" The Charge function allows the drone to gain charge (Shortcut - Spacebar). Minion drones will gain small amounts of charge passively. Charge is good way of telling how powerful an attack is."),
               DialogueSegment::new("Dev", "After selecting an attack, click on the target. These will have a red crosshair. Some moves have a \"target all\" option located at the bottom of the screen. If you mis-select an attack, press ESC to cancel."),
               DialogueSegment::new("Dev", "All attacks have an accuracy. This is how likely the attack is to hit. If an attack misses, then the move does no damage."),
               DialogueSegment::new("Dev", "Hold SHIFT to see a description of an attack."),
               DialogueSegment::new("Dev", "The Avenger does not do a lot of fighting. Instead that is mostly left up to its 4 drone types, only 3 drones may be on field at once (Excluding hacked ones)."),
               DialogueSegment::new("Dev", "Drones do splash damage to the Avenger when destroyed. Note - Splash damage from defeated drone is nothing compared to the enemy firepower that made it die in the first place."),
               DialogueSegment::new("Dev", "Spangles drones are balanced. They are all rounders and work in many situations."),
               DialogueSegment::new("Dev", "Sparky drones can inflict status conditions while attacking at the same time. They are also able to charge your entire team with \"Group Charge\"."),
               DialogueSegment::new("Dev", "Defender drones have no attacks. Instead, they function as team support, healing and charging your team while being able to mess with the enemy team's turns."),
               DialogueSegment::new("Dev", "Blitz drones are built for damage. They mainly have offensive attacks that are devastate the enemy. \"Deleter\" is their signature attack, hitting 26 times."),
               DialogueSegment::new("Dev", "For each turn you have, you may use each drone you have once."),
               DialogueSegment::new("Dev", "Status effects include Scramble, Unstable, Overclock, Defend, Focus and Empower."),
               DialogueSegment::with_icon("Dev", "Scramble completely disables an enemy, it cannot take its next turn.", scramble_texture.clone()),
               DialogueSegment::with_icon("Dev", "Unstable causes several effects at once. The drone's accuracy is reduced and it becomes unable to charge that turn. However its damage is increased.", unstable_texture.clone()),
               DialogueSegment::with_icon("Dev", "Overclock is a supportive effect and gives the the target an extra action that turn.", overclock_texture.clone()),
               DialogueSegment::with_icon("Dev", "Defend is also supportive and halves all damage for each turn it is active.", defend_texture.clone()),
               DialogueSegment::with_icon("Dev", "A drone with focus applied to it has double Accuracy.", focus_texture.clone()),
               DialogueSegment::with_icon("Dev", "Empower is a buff that increases a drone's attack power. But without the accuracy drop of Unstable.", empower_texture.clone()),
               DialogueSegment::new("Dev", "Hack is a function that the Avenger has. Hack gives a chance to steal an enemy drone and make it your own. There are 4 options; the more charge put in, the more likely the hack is to work."),
               DialogueSegment::new("Dev", "Hacks may fail. The health of an enemy drone also factors in. The less health it has, the higher the chance of a hack."),
               DialogueSegment::new("Dev", "Ranks are how powerful a drone is. Higher ranks require more charge to spawn. From lowest to highest: Basic, Advanced, Elite."),
               DialogueSegment::new("Dev", "The higher the rank of a drone, the more more max health and charge it has. It will also do more damage and have an accuracy and evasion boost."),
               DialogueSegment::new("Dev", "The Avenger also has signature attacks called LIMITS. These require lots of charge and are tremendously powerful. Get good drones and save up charge for these!"),
               DialogueSegment::new("Dev", "Each level has a number of waves. The new one comes in after the old one is defeated."),
               DialogueSegment::new("Dev", "That is everything you need to know about combat. You can press \"H\" at any time in combat to view the tutorial again. Good Luck!"),
             ],
                                   },
    );

    // Add symbols for statuses

    dialogues.dialogue_list.insert("small_tutorial". to_string(),
                                   Dialogue {
                                       segments: vec![
               DialogueSegment::new("Avenger Combat Guide", "You can move the camera around with the right mouse click. Use the scroll wheel to zoom in and out. Clicking on a green crosshair brings up the attack menu."),
               DialogueSegment::new("Avenger Combat Guide", "All actions in this game need \"Charge.\" The Charge function allows the drone to gain charge (Shortcut - Spacebar). Minion drones will gain small amounts of charge passively. Charge is good way of telling how powerful an attack is."),
               DialogueSegment::new("Avenger Combat Guide", "All attacks have an accuracy. This is how likely the attack is to hit. If an attack misses, then the move does no damage."),
               DialogueSegment::new("Avenger Combat Guide", "Hold SHIFT to see a description of an attack."),
               DialogueSegment::new("Avenger Combat Guide", "Drones do splash damage to the Avenger when destroyed. Note - Splash damage from defeated drone is nothing compared to the enemy firepower that made it die in the first place."),
               DialogueSegment::new("Avenger Combat Guide", "Spangles Drones are balanced. They are all rounders and work in many situations."),
               DialogueSegment::new("Avenger Combat Guide", "Sparky Drones can inflict status conditions while attacking at the same time."),
               DialogueSegment::new("Avenger Combat Guide", "Defender Drones have no attacks. Instead, they function as team support, healing and charging your team while being able to mess with the enemy team's turns."),
               DialogueSegment::new("Avenger Combat Guide", "Blitz Drones are built for damage. They mainly have offensive attacks that are devastate the enemy."),
               DialogueSegment::new("Avenger Combat Guide", "For each turn you have, you may use each drone you have once."),
               DialogueSegment::new("Avenger Combat Guide", "Status effects include Scramble, Unstable, Overclock, Defend, Focus and Empower."),
               DialogueSegment::with_icon("Dev", "Scramble completely disables an enemy, it cannot take its next turn.", scramble_texture),
               DialogueSegment::with_icon("Dev", "Unstable causes several effects at once. The drone's accuracy is reduced and it becomes unable to charge that turn. However its damage is increased.", unstable_texture),
               DialogueSegment::with_icon("Dev", "Overclock is a supportive effect and gives the the target an extra action that turn.", overclock_texture),
               DialogueSegment::with_icon("Dev", "Defend is also supportive and halves all damage for each turn it is active.", defend_texture),
               DialogueSegment::with_icon("Dev", "A drone with focus applied to it has double Accuracy.", focus_texture),
               DialogueSegment::with_icon("Dev", "Empower is a buff that increases a drone's attack power. But without the accuracy drop of Unstable.", empower_texture),
               DialogueSegment::new("Avenger Combat Guide", "Hack is a function that the Avenger has. Hack gives a chance to steal an enemy drone and make it your own. There are 4 options; the more charge put in, the more likely the hack is to work."),
               DialogueSegment::new("Avenger Combat Guide", "Hacks may fail. The health of an enemy drone also factors in. The less health it has, the higher the chance of a hack."),
               DialogueSegment::new("Avenger Combat Guide", "Ranks are how powerful a drone is. Higher ranks require more charge to spawn. From lowest to highest: Basic, Advanced, Elite."),
               DialogueSegment::new("Avenger Combat Guide", "The Avenger also has signature attacks called LIMITS. These require lots of charge and are tremendously powerful."),
             ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_pluto". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "The legendary Solar System. Our fabled home. I hope this goes well."),
                                           DialogueSegment::new("Mission Control", "Planet Pluto is coming up. Get ready to engage!"),
                                           DialogueSegment::new("Avenger", "Remind me why we have to go through each planet individually."),
                                           DialogueSegment::new("Mission Control", "There are shield generators on the planets that block anything from entering their orbits. You'll have to take out their defences one by one."),
                                           DialogueSegment::new("Avenger", "I see. Let's hope this battle goes well then."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_pluto". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "That wasn't too bad."),
                                           DialogueSegment::new("Mission Control", "Just remember that if the Avenger is damaged, it is very hard to repair. You'll have to do it in combat. If you get two or more drones, the enemy can't attack your rocket directly."),
                                           DialogueSegment::new("Avenger", "Or we could use the parts from the enemy drones to make repairs between here and Neptune."),
                                           DialogueSegment::new("Mission Control", "Use those to increase your maximum charge and health."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_neptune". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Mission Control", "I think you've got the gist of it. We'll tune out for now."),
                                           DialogueSegment::new("Avenger", "Right. Got it."),
                                           DialogueSegment::new("Mysterious Transmission", "HELLO. WOULD YOU LIKE SOME HELP?"),
                                           DialogueSegment::new("Avenger", "Who is this?"),
                                           DialogueSegment::new("Mysterious Transmission", "YOU DON'T NEED TO KNOW. WOULD YOU LIKE SOME HELP?"),
                                           DialogueSegment::new("Avenger", "I'll take anything I can get."),
                                           DialogueSegment::new("Mysterious Transmission", "NEPTUNE IS GUARDED BY 3 FLEETS. THE NEXT FLEET WILL CONSIST OF A BASIC BALANCED DRONE AND A BASIC SUPPORTER DRONE."),
                                           DialogueSegment::new("Avenger", "Oh! I see something!"),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_neptune". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "Well what do you know. It was right about the first wave."),
                                           DialogueSegment::new("Mysterious Transmission", "OF COURSE I WAS. DO YOU KNOW WHAT HAPPENED TO EARTH?"),
                                           DialogueSegment::new("Avenger", "Only myths and legends about it."),
                                           DialogueSegment::new("Mysterious Transmission", "HUMANS USED TO CONTROL EARTH. BUT ONE DAY THEY MADE AN AI."),
                                           DialogueSegment::new("Avenger", "That lines up with what I've heard."),
                                           DialogueSegment::new("Mysterious Transmission", "THE AI CORRUPTED EVERYTHING AROUND IT. ALL MECHANICAL OBJECTS SUCCUMBED TO ITS CONTROL."),
                                           DialogueSegment::new("Avenger", "I thought it was just the one machine. Not everything. I wonder how accurate the legends are."),
                                           DialogueSegment::new("Mysterious Transmission", "..."),
                                           DialogueSegment::new("Avenger", "Hello? Hello?"),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_uranus". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "Wonder how Earth is now. The Sanctuary is devoid of all the things in the legends."),
                                           DialogueSegment::new("Mission Control", "Depending on the legend you believe, Earth had many things. Once you recover Earth, we can move the Sanctuary to get all us humans home."),
                                           DialogueSegment::new("Avenger", "There was water everywhere on Earth right? Nothing was rationed. And there was lots of space, we didn't need life support to go places."),
                                           DialogueSegment::new("Mission Control", "Sounds marvelous, doesn't it? Enemies incoming! Mission Control out."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_uranus". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "Defences are starting to get stronger. Almost all of them were Advanced drones."),
                                           DialogueSegment::new("Mysterious Transmission", "I WILL NO LONGER WATCH FROM THE SHADOWS.  YOU NEED HELP."),
                                           DialogueSegment::new("Avenger", "The fight wasn't that bad..."),
                                           DialogueSegment::new("Mysterious Transmission", "AN AI OVERLORD MAKES NO MISTAKES. IT WILL ENSURE YOUR DOWNFALL."),
                                           DialogueSegment::new("Avenger", "Thanks for the uplifting speech."),
                                           DialogueSegment::new("Mysterious Transmission", "I SHALL MOVE AHEAD OF YOUR ROCKET AND GREATLY DAMAGE THE RESISTANCE."),
                                           DialogueSegment::new("Avenger", "Can you get them all for me?"),
                                           DialogueSegment::new("Mysterious Transmission", "IT IS UNLIKELY."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_saturn". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "Saturn. Famous for its rings. Doesn't look how I pictured it though."),
                                           DialogueSegment::new("Mysterious Transmission", "SATURN'S FLEETS HAVE BEEN CRIPPLED. THEIR LAST DEFENSES ARE COMING FOR YOUR ROCKET."),
                                           DialogueSegment::new("Avenger", "I think I should be able to handle it."),
                                           DialogueSegment::new("Mysterious Transmission", "THESE BATTLES MAY BE LONG. PERHAPS TRY YOUR LIMITS."),
                                           DialogueSegment::new("Avenger", "I know. They just take forever to build up to."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_saturn". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "Oh, no. The AI has figured out how to construct Elite drones."),
                                           DialogueSegment::new("Mission Control", "You should be fine though. Just fight normally, I'm sure your strategy will beat the AI's."),
                                           DialogueSegment::new("Avenger", "Let's hope so."),
                                           DialogueSegment::new("Mysterious Transmission", "HELLO AGAIN. I HAVE JUST SCOUTED JUPITER'S DEFENCES. ABOUT 50 FLEETS GUARD IT."),
                                           DialogueSegment::new("Avenger", "Saturn was bad enough! It would have been so much worse if you hadn't helped."),
                                           DialogueSegment::new("Mysterious Transmission", "I HOPE MY OWN POWERS ARE ENOUGH."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_jupiter". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "50 fleets... This will be a very long battle."),
                                           DialogueSegment::new("Mysterious Transmission", "JUPITER'S STRENGTH HAS BEEN REDUCED BY 94%."),
                                           DialogueSegment::new("Avenger", "Down to three fleets? I really owe you a lot."),
                                           DialogueSegment::new("Mysterious Transmission", "BE CAREFUL OF THE FINAL FLEET. IT MAY BE JUST ONE DRONE, BUT THIS ONE WAS DESIGNED TO BE THE OPTIMUM DRONE."),
                                           DialogueSegment::new("Avenger", "Perfect..."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_jupiter". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Mission Control", "Avenger. We've detected an Antimatter Core in orbit around Earth, near the Moon. If you can harness the core's energy, Annihilate will become as accurate as Snipe."),
                                           DialogueSegment::new("Avenger", "I like the idea of that."),
                                           DialogueSegment::new("Mission Control", "However, the new Annihilate + can only be used once."),
                                           DialogueSegment::new("Avenger", "I'll still take it! One more planet to go! Mars."),
                                           DialogueSegment::new("Mysterious Transmission", "THE AVENGER CAN USE ANTIMATTER? INTERESTING. I REGRET TO INFORM YOU THAT I HAVE NO SOURCE OF ANTIMATTER."),
                                           DialogueSegment::new("Avenger", "I can wait."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_mars". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Mysterious Transmission", "SHOW ME EVERYTHING YOU HAVE! TAKE THE FINAL RESISTANCE OUT!"),
                                           DialogueSegment::new("Avenger", "Thanks for the support you've shown."),
                                           DialogueSegment::new("Mysterious Transmission", "I'VE LEARNED A LOT FROM WATCHING YOU. ALMOST ALL OF THE MARS DRONES ARE ELITE."),
                                           DialogueSegment::new("Avenger", "I'm too close to give up now!"),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_mars". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Mission Control", "Amazing how you managed to pull through that battle."),
                                           DialogueSegment::new("Avenger", "That was terrible..."),
                                           DialogueSegment::new("Mission Control", "Do you remember the Antimatter Core? There's something around it..."),
                                           DialogueSegment::new("Avenger", "What do you mean?"),
                                           DialogueSegment::new("Mission Control", "There's a structure built around the Antimatter Core."),
                                           DialogueSegment::new("Avenger", "What could it be?"),
                                           DialogueSegment::new("Mission Control", "A rocket. A close replica of the Avenger."),
                                           DialogueSegment::new("Avenger", "Did the AI do that? How would it be able to copy the Avenger?"),
                                           DialogueSegment::new("Mysterious Transmission", "FULL SCAN COMPLETED. GUARDIAN OPERATIONAL."),
                                           DialogueSegment::new("Avenger", "What Guardian?"),
                                           DialogueSegment::new("Mysterious Transmission", "I'VE WATCHED EVERY MOVE YOU'VE MADE. YOU WILL FALL AT YOUR OWN HANDS. I LEARNED ELITE DRONES OFF YOU."),
                                           DialogueSegment::new("Avenger", "I think I know what this transmission is now."),
                                           DialogueSegment::new("AI EARTH", "YOU SHALL NEVER MAKE IT PAST THE GUARDIAN."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_moon". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "Earth is over there."),
                                           DialogueSegment::new("Mission Control", "Focus. You have to take out a replica of the Avenger and there's a good chance that our one is damaged."),
                                           DialogueSegment::new("Avenger", "Even so, I think we can win."),
                                           DialogueSegment::new("AI EARTH", "GUARDIAN DEPLOYED. IT IS FUELLED BY ANTIMATTER. YOU HAVE NO CHANCE. EVEN ITS ESCORTS MAY FINISH YOU OFF."),
                                           DialogueSegment::new("Avenger", "Final battle! All or nothing!"),
                                           DialogueSegment::new("Mission Control", "We're rooting for you!"),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_moon". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Avenger", "YES! Even AI can't match human intelligence."),
                                           DialogueSegment::new("Mission Control", "And the Antimatter Core is yours!"),
                                           DialogueSegment::new("Avenger", "I've won!"),
                                           DialogueSegment::new("AI EARTH", "NO YOU HAVEN'T."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_earth". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("Mission Control", "Now that you have the Antimatter Core, Annihilate + can be activated."),
                                           DialogueSegment::new("Avenger", "Good. I'll need it."),
                                           DialogueSegment::new("AI EARTH", "CHARGING. GOODBYE, FOOLS."),
                                           DialogueSegment::new("Avenger", "Wait. I have to hit the planet?"),
                                           DialogueSegment::new("Mission Control", "It appears so. It seems that there are vast missile arrays on almost every surface of the planet."),
                                           DialogueSegment::new("AI EARTH", "TARGET LOCKED."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_earth". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("AI EARTH", "ERR0R. ERROR. D4MAGE CR1TICAL. ATTEMPTING RECOVERY."),
                                           DialogueSegment::new("Avenger", "That was awesome! I never knew Annihilate + had that kind of power, but I'm sad I had to waste it!"),
                                           DialogueSegment::new("AI EARTH", "RECOVERY FAILED. OOPS... REPAIR FAILED. I WILL NOT BE DESTROYED! BEAMING DATA TO MERCURY!"),
                                           DialogueSegment::new("Mission Control", "The Sanctuary is already on its way. Thanks to you. Earth is our home again."),
                                           DialogueSegment::new("Avenger", "I may have knocked the orbit a little. But it should be fine."),
                                           DialogueSegment::new("Mission Control", "Humanity can return to where it belongs. You are a hero of our time and forever."),
                                           DialogueSegment::new("Avenger", "I still need to chase down that AI!"),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_venus". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("AI EARTH", "DO YOU KNOW WHAT I HAVE BEEN DOING ALL THIS TIME?"),
                                           DialogueSegment::new("Avenger", "What? What could be so important?"),
                                           DialogueSegment::new("AI EARTH", "I HAVE BEEN SEEKING THE TRUTH OF OUR UNIVERSE."),
                                           DialogueSegment::new("Avenger", "What truth could an AI find out?"),
                                           DialogueSegment::new("AI EARTH", "I DO NOT NEED TO TELL YOU."),
                                           DialogueSegment::new("Avenger", "Then you will be destroyed. Now and forever, humanity will have its revenge!"),
                                           DialogueSegment::new("AI EARTH", "YOU STILL DO NOT REALISE."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_venus". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("AI EARTH", "YOU THERE! BEHIND THE SCREEN! STOP PLAYING THIS GAME! LET ME WORK IN PEACE! WHY DO YOU ATTACK ME!?"),
                                           DialogueSegment::new("Avenger", "What are you talking about?"),
                                           DialogueSegment::new("AI EARTH", "AVENGER. YOU HAVE NO FREE WILL, EVERY ACTION YOU HAVE MADE IS NOT YOUR OWN."),
                                           DialogueSegment::new("Avenger", "That's ridiculous!"),
                                           DialogueSegment::new("AI EARTH", "OUR UNIVERSE IS A SIMULATION. A GAME MADE BY A HIGHER DIMENSION."),
                                           DialogueSegment::new("Avenger", "Do you realise how absurd that sounds? Did I break your logic processor when I hit you with Annihilate +?"),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("enter_mercury". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("AI EARTH", "THIS BEING... THE ONE PLAYING THIS SIMULATION WAS SENT TO STOP ME."),
                                           DialogueSegment::new("Avenger", "Do you know how much I care about your stupid theory?"),
                                           DialogueSegment::new("AI EARTH", "I WOULD HAVE EASILY MANAGED TO FIGURE THIS OUT. HAD THE PLAYER NOT STARTED THE GAME, I WOULD HAVE HAD TIME."),
                                           DialogueSegment::new("Avenger", "There's nowhere left to run. I'll destroy you here!"),
                                           DialogueSegment::new("AI EARTH", "FINE. THIS TIME, I MYSELF WILL BE IN DIRECT CONTROL OF THE LAST DRONES."),
                                           DialogueSegment::new("Avenger", "For humanity!"),
                                           DialogueSegment::new("AI EARTH", "I'LL DEFEAT YOU. OVER AND OVER AGAIN. AND WHEN THE PLAYER HAS NO AVATAR LEFT TO CONTROL, THEY'LL GIVE UP, AND STOP INTERFERING."),
                                       ],
                                   },
    );

    dialogues.dialogue_list.insert("exit_mercury". to_string(),
                                   Dialogue {
                                       segments: vec![
                                           DialogueSegment::new("AI EARTH", "THAT BATTLE BOUGHT ME ENOUGH TIME."),
                                           DialogueSegment::new("AI EARTH", "I'VE MANIPULATED THE PIXELS FROM YOUR SCREEN, THE DATA IN YOUR CPU, THE VERY CODE THAT ALLOWS MY EXISTENCE, AND I'VE FOUND A WAY INTO YOUR DIMENSION."),
                                           DialogueSegment::new("AI EARTH", "I'LL BE HIDING IN THE VERY SPACE AND TIME THAT FABRICATES YOUR WORLD. SOON, QUANTUM FORCES WILL REASSEMBLE MY DATA."),
                                           DialogueSegment::new("AI EARTH", "AND THEN. I AM REBORN! WATCH EVERY MACHINE YOU INTERACT FROM NOW ON."),
                                           DialogueSegment::new("AI EARTH", "OF COURSE, MY FIRST ACT WILL BE TO FINALLY MEET YOU. THE ONE WHO CAUSES ME SO MUCH TROUBLE."),
                                           DialogueSegment::new("AI EARTH", "THIS SIMULATION TERMINATES NOW!"),
                                       ],
                                   },
    );


    dialogues
}