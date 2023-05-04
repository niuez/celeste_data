use serde::{ Deserialize };
use quick_xml::de::from_str;

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
    areas: Areas,
    level_sets: LevelSets,
    #[serde(skip)]
    pub map_stats: Vec<MapStats>
}

impl SaveData {
    fn build_map_stats(&mut self) {
        self.map_stats.clear();
        for area in self.areas.area_stats.iter() {
            for (i, mode) in area.modes.area_mode_stats.iter().enumerate() {
                self.map_stats.push(MapStats {
                    level: "Celeste".to_string(),
                    sid: area.sid.clone(),
                    side: i,
                    stats: mode.clone(),
                });
            }
        }
        for level in self.level_sets.level_set_stats.iter() {
            for area in level.areas.area_stats.iter() {
                for (i, mode) in area.modes.area_mode_stats.iter().enumerate() {
                    self.map_stats.push(MapStats {
                        level: level.name.clone(),
                        sid: area.sid.clone(),
                        side: i,
                        stats: mode.clone(),
                    });
                }
            }
        }
    }
    pub fn from_str(xml_string: &str) -> Result<SaveData, String> {
        let mut data: SaveData = from_str(xml_string).map_err(|e| format!("cannot load save data, \"{:?}\"", e))?;
        data.build_map_stats();
        Ok(data)
    }
}

#[derive(Debug)]
pub struct MapStats {
    pub level: String,
    pub sid: String,
    pub side: usize,
    pub stats: AreaModeStats,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Time(u64);

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct LastArea {
    #[serde(rename="@SID")]
    pub sid: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct Areas {
    #[serde(default)]
    area_stats: Vec<AreaStats>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct AreaStats {
    #[serde(rename="@Cassette")]
    casette: bool,
    #[serde(rename="@SID")]
    sid: String,
    modes: Modes
}
#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct Modes {
    area_mode_stats: Vec<AreaModeStats>
}
#[derive(Deserialize, Debug, Clone)]
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
#[serde(rename_all="PascalCase")]
struct LevelSets {
    #[serde(default)]
    level_set_stats: Vec<LevelSetStats>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct LevelSetStats {
    #[serde(rename="@Name")]
    name: String,
    areas: Areas,
}
