use core::fmt;

#[derive(Debug, Clone)]
pub struct AccountBalance {
    pub client: u16,
    pub available: f32,
    pub held: f32,
    pub locked: bool,
}

impl AccountBalance {
    fn get_total(&self) -> f32 {
        self.available + self.held
    }
}

impl fmt::Display for AccountBalance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {:.4}, {:.4}, {:.4}, {}",
            self.client,
            self.available,
            self.held,
            self.get_total(),
            self.locked
        )
    }
}