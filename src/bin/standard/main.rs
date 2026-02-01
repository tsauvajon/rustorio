#![forbid(unsafe_code)]

use std::{ops::AddAssign, u64};

use rustorio::{
    self, Bundle, HandRecipe, InsufficientResourceError, Resource, ResourceType, Technology, Tick,
    buildings::{Assembler, Furnace, Lab},
    gamemodes::Standard,
    recipes::{
        CopperSmelting, CopperWireRecipe, ElectronicCircuitRecipe, FurnaceRecipe, IronSmelting,
        RedScienceRecipe,
    },
    resources::{Copper, CopperOre, Iron, IronOre, Point},
    territory::{Miner, Territory},
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

    // First Furnace
    let mut mix_furnace = Furnace::build(&tick, IronSmelting, iron);

    // Iron Miner
    let iron = SmeltIron
        .mine_and_smelt(&mut tick, &mut iron_territory, &mut mix_furnace)
        .unwrap();
    let mut mix_furnace = mix_furnace.change_recipe(CopperSmelting).unwrap();
    let copper = SmeltCopper
        .mine_and_smelt(&mut tick, &mut copper_territory, &mut mix_furnace)
        .unwrap();
    let miner = Miner::build(iron, copper);
    iron_territory.add_miner(&tick, miner).unwrap();

    // Copper Miner
    let mut mix_furnace = mix_furnace.change_recipe(IronSmelting).unwrap();
    let iron = SmeltIron
        .mine_and_smelt(&mut tick, &mut iron_territory, &mut mix_furnace)
        .unwrap();
    let mut mix_furnace = mix_furnace.change_recipe(CopperSmelting).unwrap();
    let copper = SmeltCopper
        .mine_and_smelt(&mut tick, &mut copper_territory, &mut mix_furnace)
        .unwrap();
    let miner = Miner::build(iron, copper);
    copper_territory.add_miner(&tick, miner).unwrap();

    // Dedicated copper furnace
    let iron_ore = iron_territory.resources(&tick).bundle::<10>().unwrap();
    let mut iron_furnace = mix_furnace.change_recipe(IronSmelting).unwrap();
    iron_furnace.inputs(&tick).0.add_bundle(iron_ore);
    tick.advance_until(|_tick| iron_furnace.output_amounts().0 == 10, u64::MAX);
    let iron = iron_furnace.outputs(&tick).0.bundle().unwrap();
    let mut copper_furnace = Furnace::build(&tick, CopperSmelting, iron);

    // Assembler
    let copper = SmeltCopper
        .mine_and_smelt(&mut tick, &mut copper_territory, &mut copper_furnace)
        .unwrap();
    let mut copper_wires = CopperWireRecipe::craft(&mut tick, (copper,))
        .0
        .to_resource();
    for _ in 0..5 {
        let copper = SmeltCopper
            .mine_and_smelt(&mut tick, &mut copper_territory, &mut copper_furnace)
            .unwrap();
        let new_copper_wires = CopperWireRecipe::craft(&mut tick, (copper,))
            .0
            .to_resource();
        copper_wires.add_assign(new_copper_wires);
    }
    let copper_wires = copper_wires.bundle().unwrap();
    let iron = SmeltIron
        .mine_and_smelt(&mut tick, &mut iron_territory, &mut iron_furnace)
        .unwrap();
    let mut assembler = Assembler::build(&tick, RedScienceRecipe, copper_wires, iron);

    // Lab
    let iron = SmeltIron
        .mine_and_smelt(&mut tick, &mut iron_territory, &mut iron_furnace)
        .unwrap();
    let copper = SmeltCopper
        .mine_and_smelt(&mut tick, &mut copper_territory, &mut copper_furnace)
        .unwrap();
    let mut lab = Lab::build(&tick, &steel_technology, iron, copper);

    // optimisation: start building some Electronic Circuits and Red Science earlier

    // Research steel
    for _ in 0..20 {
        // Electronic circuit
        let copper = SmeltCopper
            .mine_and_smelt(&mut tick, &mut copper_territory, &mut copper_furnace)
            .unwrap();
        let copper_wires = CopperWireRecipe::craft(&mut tick, (copper,)).0;
        let iron = SmeltIron
            .mine_and_smelt(&mut tick, &mut iron_territory, &mut iron_furnace)
            .unwrap();
        let electronic_circuit = ElectronicCircuitRecipe::craft(&mut tick, (iron, copper_wires)).0;

        // Red Science
        let iron = SmeltIron
            .mine_and_smelt::<1>(&mut tick, &mut iron_territory, &mut iron_furnace)
            .unwrap();
        assembler.inputs(&tick).0.add_bundle(iron);
        assembler.inputs(&tick).1.add_bundle(electronic_circuit);
        tick.advance_until(|_tick| assembler.output_amounts().0.gt(&0), u64::MAX);
        let red_science = assembler.outputs(&tick).0.bundle::<1>().unwrap();

        // Feed science to lab
        lab.inputs(&tick).0.add(red_science);
    }

    tick.advance_until(|tick| lab.outputs(&tick).0.amount().ge(&20), u64::MAX);
    let research_points = lab.outputs(&tick).0.bundle().unwrap();
    let (steel_smelting, points_technology) = steel_technology.research(research_points);

    // Research Points
    let mut lab = lab.change_technology(&points_technology).unwrap();
    for _ in 0..50 {
        // Electronic circuit
        let copper = SmeltCopper
            .mine_and_smelt(&mut tick, &mut copper_territory, &mut copper_furnace)
            .unwrap();
        let copper_wires = CopperWireRecipe::craft(&mut tick, (copper,)).0;
        let iron = SmeltIron
            .mine_and_smelt(&mut tick, &mut iron_territory, &mut iron_furnace)
            .unwrap();
        let electronic_circuit = ElectronicCircuitRecipe::craft(&mut tick, (iron, copper_wires)).0;

        // Red Science
        let iron = SmeltIron
            .mine_and_smelt::<1>(&mut tick, &mut iron_territory, &mut iron_furnace)
            .unwrap();
        assembler.inputs(&tick).0.add_bundle(iron);
        assembler.inputs(&tick).1.add_bundle(electronic_circuit);
        tick.advance_until(|_tick| assembler.output_amounts().0.gt(&0), u64::MAX);
        let red_science = assembler.outputs(&tick).0.bundle::<1>().unwrap();

        // Feed science to lab
        lab.inputs(&tick).0.add(red_science);
    }

    tick.advance_until(|tick| lab.outputs(&tick).0.amount().ge(&50), u64::MAX);
    let research_points = lab.outputs(&tick).0.bundle().unwrap();
    let points_recipe = points_technology.research(research_points);

    // Dedicated steel furnace
    // optimise: build at the same time as something else ticking
    let iron_ore = iron_territory.resources(&tick).bundle::<10>().unwrap();
    iron_furnace.inputs(&tick).0.add_bundle(iron_ore);
    tick.advance_until(|_tick| iron_furnace.output_amounts().0 == 10, u64::MAX);
    let iron = iron_furnace.outputs(&tick).0.bundle().unwrap();
    let mut steel_furnace = Furnace::build(&tick, steel_smelting, iron);

    let mut assembler = assembler.change_recipe(points_recipe).unwrap();
    for _ in 0..200 {
        // Iron in oven to save some time
        let iron_ore = iron_territory.resources(&tick).bundle::<5>().unwrap();
        iron_furnace.inputs(&tick).0.add_bundle(iron_ore);

        // 4 Electronic circuits
        for _ in 0..4 {
            let copper = SmeltCopper
                .mine_and_smelt(&mut tick, &mut copper_territory, &mut copper_furnace)
                .unwrap();
            let copper_wires = CopperWireRecipe::craft(&mut tick, (copper,)).0;
            let iron = SmeltIron
                .mine_and_smelt(&mut tick, &mut iron_territory, &mut iron_furnace)
                .unwrap();
            let electronic_circuit =
                ElectronicCircuitRecipe::craft(&mut tick, (iron, copper_wires)).0;
            assembler.inputs(&tick).0.add(electronic_circuit);
        }

        // Finish smelting steel
        let steel = steel_furnace.outputs(&tick).0.bundle::<1>().unwrap();
        assembler.inputs(&tick).1.add(steel);
    }
    let points = assembler.outputs(&tick).0.bundle().unwrap();

    (tick, points)
}

trait Smelting {
    type Ore: ResourceType;
    type Smelted: ResourceType;
    type Recipe: FurnaceRecipe;

    fn first_input<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Ore>;
    fn first_output<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Smelted>;

    fn mine_and_smelt<const AMOUNT: u32>(
        &self,
        tick: &mut Tick,
        territory: &mut Territory<Self::Ore>,
        mut furnace: &mut Furnace<Self::Recipe>,
    ) -> Result<Bundle<Self::Smelted, AMOUNT>, InsufficientResourceError<Self::Smelted>> {
        self.mine_into_furnace::<AMOUNT>(tick, territory, &mut furnace);

        // optimise: only ever tick by hand mining, never by just waiting
        tick.advance_until(
            |tick| self.first_output(tick, &mut furnace).amount().ge(&AMOUNT),
            u64::MAX,
        );
        self.first_output(tick, &mut furnace).bundle()
    }

    // optimise: (step1) parallelise many miners - this should take care of ticks naturally IMO
    // optimise: (step2) use a 'yield' system to make progress on all things at once
    fn mine_into_furnace<const AMOUNT: u32>(
        &self,
        tick: &mut Tick,
        territory: &mut Territory<Self::Ore>,
        mut furnace: &mut Furnace<Self::Recipe>,
    ) {
        let mut remaining = AMOUNT;
        for _ in 0..AMOUNT {
            let resources = territory.resources(tick);
            let resources = resources.split_off_max(remaining);

            remaining = remaining - resources.amount();
            self.first_input(tick, &mut furnace).add(resources);

            if remaining == 0 {
                return;
            }

            let ore = territory.hand_mine::<1>(tick);
            // For manual optimisation
            eprintln!("WARN: hand mining {} at tick {tick}", Self::Ore::NAME);
            remaining = remaining - 1;
            self.first_input(tick, &mut furnace).add_bundle(ore);
        }
    }
}

// TODO: make it hold the territory and a furnace array so it's easier to call?
struct SmeltIron;

impl Smelting for SmeltIron {
    type Ore = IronOre;

    type Smelted = Iron;

    type Recipe = IronSmelting;

    fn first_input<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Ore> {
        &mut furnace.inputs(tick).0
    }

    fn first_output<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Smelted> {
        &mut furnace.outputs(tick).0
    }
}

struct SmeltCopper;

impl Smelting for SmeltCopper {
    type Ore = CopperOre;

    type Smelted = Copper;

    type Recipe = CopperSmelting;

    fn first_input<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Ore> {
        &mut furnace.inputs(tick).0
    }

    fn first_output<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Smelted> {
        &mut furnace.outputs(tick).0
    }
}
