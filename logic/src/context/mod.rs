pub mod species;

use std::sync::OnceLock;

use species::SpeciesProvider;

use crate::app::Game;

const PKMN: &str = include_str!("../../../reference/pokeapi/data/v2/csv/pokemon_species.csv");
const PKMN_NAMES: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/pokemon_species_names.csv");
const EXP: &str = include_str!("../../../reference/pokeapi/data/v2/csv/experience.csv");
const TYPES: &str = include_str!("../../../reference/pokeapi/data/v2/csv/type_names.csv");
const MOVES: &str = include_str!("../../../reference/pokeapi/data/v2/csv/moves.csv");

pub struct PkContext {
    species: species::SpeciesProvider,
}

impl PkContext {
    fn new() -> PkContext {
        PkContext {
            species: SpeciesProvider::new(),
        }
    }

    pub fn get() -> &'static PkContext {
        static CONTEXT: OnceLock<PkContext> = OnceLock::new();
        CONTEXT.get_or_init(Self::new)
    }

    pub fn species(&self) -> &species::SpeciesProvider {
        &self.species
    }

    pub fn sprite_path(&self, game: &Game, species: u32, shiny: bool) -> String {
        let dir = match game {
            Game::FireRedLeafGreen => "frlg",
            Game::DiamondPearl => "dp",
            Game::RubySapphire => "rs",
            Game::HeartGoldSoulSilver => "hgss",
            Game::Black2White2 => "bw",
            Game::BlackWhite => "bw",
            _ => panic!("Unimplemented game"), // TODO other games
        };
        if shiny {
            format!("../reference/images/{}/{:03}_shiny.png", dir, species)
        } else {
            format!("../reference/images/{}/{:03}.png", dir, species)
        }
    }
}
