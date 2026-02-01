#![forbid(unsafe_code)]

use rustorio::Recipe;
use rustorio::buildings::Furnace;
use rustorio::recipes::CopperSmelting;
use rustorio::territory::MINING_TICK_LENGTH;
use rustorio::{self, Bundle, Tick, gamemodes::Tutorial, resources::Copper};

type GameMode = Tutorial;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;

fn main() {
    rustorio::play::<GameMode>(user_main);
}

fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, Bundle<Copper, 4>) {
    tick.log(true);

    let StartingResources {
        iron,
        iron_territory: _,
        mut copper_territory,
        guide: _,
    } = starting_resources;

    let copper_ore = copper_territory.hand_mine::<4>(&mut tick);
    tick.advance_by(MINING_TICK_LENGTH * 4);

    let mut furnace = Furnace::build(&tick, CopperSmelting {}, iron);
    furnace.inputs(&tick).0.add(copper_ore);
    tick.advance_by(CopperSmelting::TIME * 4);

    let copper = furnace.outputs(&tick).0.bundle().unwrap();

    (tick, copper)
}
