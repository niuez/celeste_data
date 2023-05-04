use serde::{ Deserialize };

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct SaveData {
    pub version: String,
    pub name: String,
    pub time: Time,
    pub total_deaths: u64,
    pub total_strawberries: u64,
    pub total_golden_strawberries: u64,
    pub total_jumps: u64,
    pub total_wall_jumps: u64,
    pub total_dashes: u64,
    pub last_area: LastArea,
    pub areas: Areas,
    pub level_sets: LevelSets,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct LastArea {
    #[serde(rename="@SID")]
    pub sid: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct Areas {
    #[serde(default)]
    pub area_stats: Vec<AreaStats>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct AreaStats {
    #[serde(rename="@Cassette")]
    pub casette: bool,
    #[serde(rename="@SID")]
    pub sid: String,
    pub modes: Modes
}
#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct Modes {
    pub area_mode_stats: Vec<AreaModeStats>
}
#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct AreaModeStats {
    #[serde(rename="@TotalStrawberries")]
    pub total_strawberries: u64,
    #[serde(rename="@Completed")]
    pub completed: bool,
    #[serde(rename="@SingleRunCompleted")]
    pub single_run_completed: bool,
    #[serde(rename="@FullClear")]
    pub full_clear: bool,
    #[serde(rename="@Deaths")]
    pub deaths: u64,
    #[serde(rename="@TimePlayed")]
    pub time_played: Time,
    #[serde(rename="@BestTime")]
    pub best_time: Time,
    #[serde(rename="@BestFullClearTime")]
    pub best_full_clear_time: Time,
    #[serde(rename="@BestDashes")]
    pub best_dashes: u64,
    #[serde(rename="@BestDeaths")]
    pub best_deaths: u64,
    #[serde(rename="@HeartGem")]
    pub heart_gem: bool,
}

#[derive(Deserialize, Debug)]
pub struct Time(u64);

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct LevelSets {
    #[serde(default)]
    pub level_set_stats: Vec<LevelSetStats>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct LevelSetStats {
    pub areas: Areas,
}
