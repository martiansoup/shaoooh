use std::collections::HashMap;

use strum::EnumCount;

use crate::{app::Game, context};

pub struct VersionsProvider {
    game_mapping: HashMap<Game, u32>,
    group_mapping: HashMap<u32, Vec<u32>>,
    name_mapping: HashMap<u32, String>,
}

impl VersionsProvider {
    fn game_from_group(version: &str) -> Option<Game> {
        match version {
            "ruby-sapphire" => Some(Game::RubySapphire),
            "emerald" => Some(Game::Emerald),
            "firered-leafgreen" => Some(Game::FireRedLeafGreen),
            "diamond-pearl" => Some(Game::DiamondPearl),
            "platinum" => Some(Game::Platinum),
            "heartgold-soulsilver" => Some(Game::HeartGoldSoulSilver),
            "black-white" => Some(Game::BlackWhite),
            "black-2-white-2" => Some(Game::Black2White2),
            "ultra-sun-ultra-moon" => Some(Game::UltraSunUltraMoon),
            _ => None,
        }
    }

    pub fn new() -> Self {
        let mut game_mapping = HashMap::new();
        let mut group_mapping = HashMap::new();
        let mut name_mapping = HashMap::new();

        let mut csv_groups = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::VERSION_GROUPS.as_bytes());

        for r in csv_groups.records().flatten() {
            let id: u32 = str::parse(&r[0]).expect("Couldn't parse version ID");
            let game = Self::game_from_group(&r[1]);

            match game {
                Some(g) => {
                    game_mapping.insert(g, id);
                    group_mapping.insert(id, vec![]);
                }
                None => {}
            }
        }

        let mut csv_versions = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::VERSIONS.as_bytes());

        for r in csv_versions.records().flatten() {
            let id: u32 = str::parse(&r[0]).expect("Couldn't parse version ID");
            let group: u32 = str::parse(&r[1]).expect("Couldn't parse group ID");
            let name = r[2].to_string();

            match group_mapping.get_mut(&group) {
                Some(ids) => {
                    ids.push(id);
                    name_mapping.insert(id, name);
                }
                None => {}
            }
        }

        game_mapping.insert(Game::None, 0);

        if game_mapping.len() != Game::COUNT {
            log::warn!("Not all games mapped to versions");
            log::warn!("{:?}", game_mapping);
        }

        VersionsProvider {
            game_mapping,
            group_mapping,
            name_mapping,
        }
    }

    pub fn all_ids(&self) -> Vec<u32> {
        self.name_mapping.keys().copied().collect()
    }

    pub fn get_ids(&self, game: &Game) -> Vec<u32> {
        let group_id = self
            .game_mapping
            .get(game)
            .expect("Couldn't find group for game");

        self.group_mapping
            .get(group_id)
            .expect("Couldn't get IDs for group")
            .to_vec()
    }

    pub fn get_name(&self, id: u32) -> String {
        self.name_mapping
            .get(&id)
            .expect("Couldn't get name")
            .to_string()
    }
}

impl Default for VersionsProvider {
    fn default() -> Self {
        Self::new()
    }
}
