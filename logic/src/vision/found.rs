pub struct FoundToggle {
    entry0: Vec<u8>,
    entry1: Vec<u8>,
    toggle: bool,
}

impl Default for FoundToggle {
    fn default() -> Self {
        Self::new()
    }
}

impl FoundToggle {
    pub fn new() -> Self {
        let frame = std::fs::read("static/metamon_desaturated.png").unwrap_or_default();

        Self {
            entry0: frame.clone(),
            entry1: frame,
            toggle: false,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        if self.toggle {
            self.entry1 = data.to_vec();
        } else {
            self.entry0 = data.to_vec();
        }
        self.toggle = !self.toggle;
    }

    pub fn latest(&self) -> &[u8] {
        if self.toggle {
            &self.entry0
        } else {
            &self.entry1
        }
    }

    pub fn last(&self) -> &[u8] {
        if self.toggle {
            &self.entry1
        } else {
            &self.entry0
        }
    }
}
