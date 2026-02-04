#![forbid(unsafe_code)]

use std::{ops::AddAssign, u64};

use rustorio::{
    self, Bundle, HandRecipe, Technology, Tick,
    buildings::{Assembler, Furnace, Lab},
    gamemodes::Standard,
    recipes::{
        CopperSmelting, CopperWireRecipe, ElectronicCircuitRecipe, IronSmelting, RedScienceRecipe,
    },
    resources::Point,
    territory::Miner,
};
use rustorio_game::{
    furnace::FlexibleFurnace,
    smelting::{SmeltCopper, SmeltIron, Smelting as _},
};

type GameMode = Standard;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;

fn main() {
    rustorio::play::<GameMode>(user_main);
}

fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, Bundle<Point, 200>) {
    let StartingResources {
        iron,
        mut iron_territory,
        mut copper_territory,
        steel_technology,
    } = starting_resources;

    let mut furnace = FlexibleFurnace::new(Furnace::build(&tick, IronSmelting, iron));
    println!("INFO: got first furnace at tick {tick}");

    // Iron Miner
    let iron = SmeltIron
        .mine_and_smelt(&mut tick, &mut iron_territory, &mut furnace)
        .unwrap();
    let copper = SmeltCopper
        .mine_and_smelt(&mut tick, &mut copper_territory, &mut furnace)
        .unwrap();
    let miner = Miner::build(iron, copper);
    iron_territory.add_miner(&tick, miner).unwrap();
    println!("INFO: got iron miner at tick {tick}");

    // Copper Miner
    let iron = SmeltIron
        .mine_and_smelt(&mut tick, &mut iron_territory, &mut furnace)
        .unwrap();
    let copper = SmeltCopper
        .mine_and_smelt(&mut tick, &mut copper_territory, &mut furnace)
        .unwrap();
    let miner = Miner::build(iron, copper);
    copper_territory.add_miner(&tick, miner).unwrap();
    println!("INFO: got copper miner at tick {tick}");

    // Assembler
    let copper = SmeltCopper
        .mine_and_smelt(&mut tick, &mut copper_territory, &mut furnace)
        .unwrap();
    let mut copper_wires = CopperWireRecipe::craft(&mut tick, (copper,))
        .0
        .to_resource();
    for _ in 0..5 {
        let copper = SmeltCopper
            .mine_and_smelt(&mut tick, &mut copper_territory, &mut furnace)
            .unwrap();
        let new_copper_wires = CopperWireRecipe::craft(&mut tick, (copper,))
            .0
            .to_resource();
        copper_wires.add_assign(new_copper_wires);
    }
    let copper_wires = copper_wires.bundle().unwrap();
    let iron = SmeltIron
        .mine_and_smelt(&mut tick, &mut iron_territory, &mut furnace)
        .unwrap();
    let mut assembler = Assembler::build(&tick, RedScienceRecipe, copper_wires, iron);
    println!("INFO: got assembler at tick {tick}");

    // Lab
    let iron = SmeltIron
        .mine_and_smelt(&mut tick, &mut iron_territory, &mut furnace)
        .unwrap();
    let copper = SmeltCopper
        .mine_and_smelt(&mut tick, &mut copper_territory, &mut furnace)
        .unwrap();
    let mut lab = Lab::build(&tick, &steel_technology, iron, copper);
    println!("INFO: got lab at tick {tick}");

    // optimisation: start building some Electronic Circuits and Red Science earlier

    // Research steel
    for _ in 1..=20 {
        // Electronic circuit
        let copper = SmeltCopper
            .mine_and_smelt(&mut tick, &mut copper_territory, &mut furnace)
            .unwrap();
        let copper_wires = CopperWireRecipe::craft(&mut tick, (copper,)).0;
        let iron = SmeltIron
            .mine_and_smelt(&mut tick, &mut iron_territory, &mut furnace)
            .unwrap();
        let electronic_circuit = ElectronicCircuitRecipe::craft(&mut tick, (iron, copper_wires)).0;

        // Red Science
        let iron = SmeltIron
            .mine_and_smelt::<1>(&mut tick, &mut iron_territory, &mut furnace)
            .unwrap();
        assembler.inputs(&tick).0.add_bundle(iron);
        assembler.inputs(&tick).1.add_bundle(electronic_circuit);
        tick.advance_until(|tick| assembler.outputs(&tick).0.amount().ge(&1), u64::MAX);
        let red_science = assembler.outputs(&tick).0.bundle::<1>().unwrap();

        // Feed science to lab
        lab.inputs(&tick).0.add(red_science);
    }

    tick.advance_until(|tick| lab.outputs(&tick).0.amount().ge(&20), u64::MAX);
    let research_points = lab.outputs(&tick).0.bundle().unwrap();
    let (steel_smelting, points_technology) = steel_technology.research(research_points);
    println!("INFO: researched steel at tick {tick}");

    // Research Points
    let mut lab = lab.change_technology(&points_technology).unwrap();
    for _ in 0..50 {
        // Electronic circuit
        let copper = SmeltCopper
            .mine_and_smelt(&mut tick, &mut copper_territory, &mut furnace)
            .unwrap();
        let copper_wires = CopperWireRecipe::craft(&mut tick, (copper,)).0;
        let iron = SmeltIron
            .mine_and_smelt(&mut tick, &mut iron_territory, &mut furnace)
            .unwrap();
        let electronic_circuit = ElectronicCircuitRecipe::craft(&mut tick, (iron, copper_wires)).0;

        // Red Science
        let iron = SmeltIron
            .mine_and_smelt::<1>(&mut tick, &mut iron_territory, &mut furnace)
            .unwrap();
        assembler.inputs(&tick).0.add_bundle(iron);
        assembler.inputs(&tick).1.add_bundle(electronic_circuit);
        tick.advance_until(|tick| assembler.outputs(&tick).0.amount().ge(&1), u64::MAX);
        let red_science = assembler.outputs(&tick).0.bundle::<1>().unwrap();

        // Feed science to lab
        lab.inputs(&tick).0.add(red_science);
    }

    tick.advance_until(|tick| lab.outputs(&tick).0.amount().ge(&50), u64::MAX);
    let research_points = lab.outputs(&tick).0.bundle().unwrap();
    let points_recipe = points_technology.research(research_points);
    println!("INFO: researched points at tick {tick}");

    // Dedicated steel furnace
    // optimise: build at the same time as something else ticking
    let iron = SmeltIron
        .mine_and_smelt(&mut tick, &mut iron_territory, &mut furnace)
        .unwrap();
    let mut steel_furnace = Furnace::build(&tick, steel_smelting, iron);
    println!("INFO: got steel furnace at tick {tick}");

    let mut assembler = assembler.change_recipe(points_recipe).unwrap();
    for _ in 1..=200 {
        // Iron in steel furnace ASAP to save some time
        let iron = SmeltIron
            .mine_and_smelt::<5>(&mut tick, &mut iron_territory, &mut furnace)
            .unwrap();
        steel_furnace.inputs(&tick).0.add_bundle(iron);

        // 4 Electronic circuits
        for _ in 1..=4 {
            let copper = SmeltCopper
                .mine_and_smelt(&mut tick, &mut copper_territory, &mut furnace)
                .unwrap();
            let copper_wires = CopperWireRecipe::craft(&mut tick, (copper,)).0;
            let iron = SmeltIron
                .mine_and_smelt(&mut tick, &mut iron_territory, &mut furnace)
                .unwrap();
            let electronic_circuit =
                ElectronicCircuitRecipe::craft(&mut tick, (iron, copper_wires)).0;
            assembler.inputs(&tick).0.add(electronic_circuit);
        }

        // Finish smelting steel
        tick.advance_until(
            |tick| steel_furnace.outputs(&tick).0.amount().ge(&1),
            u64::MAX,
        );
        let steel = steel_furnace.outputs(&tick).0.bundle::<1>().unwrap();
        assembler.inputs(&tick).1.add(steel);
    }

    tick.advance_until(
        |tick| assembler.outputs(&tick).0.amount().ge(&200),
        u64::MAX,
    );
    let points = assembler.outputs(&tick).0.bundle().unwrap();
    println!("INFO: built 200 points at tick {tick}");

    (tick, points)
}
