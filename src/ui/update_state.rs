#[derive(enum_as_inner::EnumAsInner)]
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
}
