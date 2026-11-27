#[must_use]
pub enum ModelState {
    Clean,
    Dirty,
}

impl ModelState {
    pub fn and_then(self, other: Self) -> Self {
        match other {
            ModelState::Dirty => ModelState::Dirty,
            _ => self,
        }
    }

    pub fn is_dirty(&self) -> bool {
        matches!(self, Self::Dirty)
    }
}
