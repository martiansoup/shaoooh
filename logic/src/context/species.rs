use std::collections::HashMap;

use crate::context;

pub struct SpeciesProvider {
    mapping: HashMap<u32, String>,
    inv_mapping: HashMap<String, u32>,
}

impl SpeciesProvider {
    pub fn new() -> Self {
        let mut mapping = HashMap::new();
        let mut inv_mapping = HashMap::new();

        let mut csv_names = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::PKMN_NAMES.as_bytes());

        for r in csv_names.records().flatten() {
            let num: u32 = str::parse(&r[0]).expect("Couldn't parse species ID");
            let lang: u32 = str::parse(&r[1]).expect("Couldn't parse species ID");
            let name = r[2].to_string();

            if lang == 9 {
                mapping.insert(num, name.clone());
                inv_mapping.insert(name.to_lowercase(), num);
            }
        }

        SpeciesProvider {
            mapping,
            inv_mapping,
        }
    }

    pub fn name(&self, id: u32) -> String {
        self.mapping.get(&id).unwrap_or(&"".to_string()).to_string()
    }

    pub fn species(&self, name: &str) -> Option<u32> {
        let lower = name.to_lowercase();
        self.inv_mapping.get(&lower).copied()
    }
}

impl Default for SpeciesProvider {
    fn default() -> Self {
        Self::new()
    }
}
