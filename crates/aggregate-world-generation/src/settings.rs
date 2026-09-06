use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldInitializationSettings {
    pub population_per_province: u64,
    pub workforce_percent: u64,
    pub food_reserve_days: u64,
    pub construction_goods_per_province: u64,
}

impl Default for WorldInitializationSettings {
    fn default() -> Self {
        Self { population_per_province: 200, workforce_percent: 50, food_reserve_days: 30, construction_goods_per_province: 100 }
    }
}

impl WorldInitializationSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !(20..=1_000_000).contains(&self.population_per_province) {
            return Err("population per province must be between 20 and 1000000".into());
        }
        if !(40..=80).contains(&self.workforce_percent) {
            return Err("workforce percentage must be between 40 and 80".into());
        }
        if !(1..=3650).contains(&self.food_reserve_days) || self.construction_goods_per_province > 1_000_000_000 {
            return Err("initial reserves exceed the supported range".into());
        }
        Ok(())
    }
}
