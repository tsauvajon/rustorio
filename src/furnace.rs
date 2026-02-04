use rustorio::{
    buildings::Furnace,
    recipes::{CopperSmelting, FurnaceRecipe, IronSmelting, SteelSmelting},
};
use rustorio_engine::machine::MachineNotEmptyError;

#[derive(Debug)]
pub enum FurnaceState {
    Iron(Furnace<IronSmelting>),
    Copper(Furnace<CopperSmelting>),
    Steel(Furnace<SteelSmelting>),
    #[doc(hidden)]
    Vacant,
}

impl FurnaceState {
    fn change_recipe<NewRecipe: SupportedFurnaceRecipe>(
        self,
        recipe: NewRecipe,
    ) -> Result<Self, MachineNotEmptyError<Self>> {
        match self {
            FurnaceState::Iron(furnace) => furnace
                .change_recipe(recipe)
                .map(NewRecipe::wrap)
                .map_err(|err| err.map_machine(FurnaceState::Iron)),
            FurnaceState::Copper(furnace) => furnace
                .change_recipe(recipe)
                .map(NewRecipe::wrap)
                .map_err(|err| err.map_machine(FurnaceState::Copper)),
            FurnaceState::Steel(furnace) => furnace
                .change_recipe(recipe)
                .map(NewRecipe::wrap)
                .map_err(|err| err.map_machine(FurnaceState::Steel)),
            FurnaceState::Vacant => unreachable!("temporary variant is never exposed"),
        }
    }
}

pub trait SupportedFurnaceRecipe: FurnaceRecipe + Sized {
    fn wrap(furnace: Furnace<Self>) -> FurnaceState;
    fn try_get(state: &mut FurnaceState) -> Option<&mut Furnace<Self>>;
}

impl SupportedFurnaceRecipe for IronSmelting {
    fn wrap(furnace: Furnace<Self>) -> FurnaceState {
        FurnaceState::Iron(furnace)
    }

    fn try_get(state: &mut FurnaceState) -> Option<&mut Furnace<Self>> {
        match state {
            FurnaceState::Iron(furnace) => Some(furnace),
            _ => None,
        }
    }
}

impl SupportedFurnaceRecipe for CopperSmelting {
    fn wrap(furnace: Furnace<Self>) -> FurnaceState {
        FurnaceState::Copper(furnace)
    }

    fn try_get(state: &mut FurnaceState) -> Option<&mut Furnace<Self>> {
        match state {
            FurnaceState::Copper(furnace) => Some(furnace),
            _ => None,
        }
    }
}

impl SupportedFurnaceRecipe for SteelSmelting {
    fn wrap(furnace: Furnace<Self>) -> FurnaceState {
        FurnaceState::Steel(furnace)
    }

    fn try_get(state: &mut FurnaceState) -> Option<&mut Furnace<Self>> {
        match state {
            FurnaceState::Steel(furnace) => Some(furnace),
            _ => None,
        }
    }
}

pub struct FlexibleFurnace {
    furnace: FurnaceState,
}

impl FlexibleFurnace {
    pub fn new<R: SupportedFurnaceRecipe>(furnace: Furnace<R>) -> Self {
        Self {
            furnace: R::wrap(furnace),
        }
    }

    pub fn as_recipe_mut<R: SupportedFurnaceRecipe>(&mut self) -> Option<&mut Furnace<R>> {
        R::try_get(&mut self.furnace)
    }

    pub fn change_recipe<NewRecipe: SupportedFurnaceRecipe>(
        &mut self,
        new_recipe: NewRecipe,
    ) -> Result<(), MachineNotEmptyError<()>> {
        let current = std::mem::replace(&mut self.furnace, FurnaceState::Vacant);
        match current.change_recipe(new_recipe) {
            Ok(next) => {
                self.furnace = next;
                Ok(())
            }
            Err(err) => Err(err.map_machine(|state| {
                self.furnace = state;
                ()
            })),
        }
    }
}
