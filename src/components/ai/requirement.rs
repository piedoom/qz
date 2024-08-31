#[derive(Debug)]
pub enum Requirement {
    EnemiesInView,
    TargetInWeaponsRange,
    TargetDestroyed,
}

impl Requirement {
    pub const fn name(&self) -> &'static str {
        match self {
            Requirement::EnemiesInView => "enemies_in_range",
            Requirement::TargetInWeaponsRange => "target_in_weapons_range",
            Requirement::TargetDestroyed => "target_destroyed",
        }
    }
}
