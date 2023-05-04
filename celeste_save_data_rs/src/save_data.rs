use std::collections::HashMap;
use crate::time::Time;
use serde::{ Deserialize };
use quick_xml::de::from_str;

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct SaveData {
    pub version: String,
    pub name: String,
    pub time: Time,
    pub total_deaths: u64,
    //pub total_strawberries: u64,
    //pub total_golden_strawberries: u64,
    pub total_jumps: u64,
    pub total_wall_jumps: u64,
    pub total_dashes: u64,
    pub last_area: LastArea,
    areas: Areas,
    level_sets: LevelSets,
    #[serde(skip)]
    pub map_stats: HashMap<MapCode, AreaModeStats>
}

impl SaveData {
    fn build_map_stats(&mut self) {
        self.map_stats.clear();
        for area in self.areas.area_stats.iter() {
            for (i, mode) in area.modes.area_mode_stats.iter().enumerate() {
                self.map_stats.insert(
                    MapCode {
                        level: "Celeste".to_string(),
                        sid: area.sid.clone(),
                        side: i,
                    },
                    mode.clone());
            }
        }
        for level in self.level_sets.level_set_stats.iter() {
            for area in level.areas.area_stats.iter() {
                for (i, mode) in area.modes.area_mode_stats.iter().enumerate() {
                    self.map_stats.insert(
                        MapCode {
                            level: level.name.to_string(),
                            sid: area.sid.clone(),
                            side: i,
                        },
                        mode.clone());
                }
            }
        }
    }
    // Create SaveData instance from xml string
    pub fn from_str(xml_string: &str) -> Result<SaveData, String> {
        let mut data: SaveData = from_str(xml_string).map_err(|e| format!("cannot load save data, \"{:?}\"", e))?;
        data.build_map_stats();
        Ok(data)
    }
    // merge two SaveDatas for those who are separating save data for multi mods
    pub fn merge(&mut self, right: SaveData) {
        self.time += right.time;
        self.total_deaths += right.total_deaths;
        //self.total_strawberries += right.total_strawberries
        //self.total_golden_strawberries += right.total_golden_strawberries
        self.total_jumps += right.total_jumps;
        self.total_wall_jumps += right.total_wall_jumps;
        self.total_dashes += right.total_dashes;
        for (code, stats) in right.map_stats.into_iter() {
            if let Some(left_stats) = self.map_stats.get_mut(&code) {
                left_stats.merge(stats);
            }
            else {
                self.map_stats.insert(code, stats);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MapCode {
    pub level: String,
    pub sid: String,
    pub side: usize,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="PascalCase")]
pub struct AreaModeStats {
    //#[serde(rename="@TotalStrawberries")]
    //pub total_strawberries: u64,
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

impl AreaModeStats {
    fn merge(&mut self, right: Self) {
        //self.total_strawberries += right.total_strawberries;
        self.completed |= right.completed;
        self.single_run_completed |= right.single_run_completed;
        self.full_clear |= right.full_clear;
        self.deaths += right.deaths;
        self.time_played += right.time_played;
        self.best_time = std::cmp::min(self.best_time, right.best_time);
        self.best_full_clear_time = std::cmp::min(self.best_full_clear_time, right.best_full_clear_time);
        self.best_dashes = std::cmp::min(self.best_dashes, right.best_dashes);
        self.best_deaths = std::cmp::min(self.best_deaths, right.best_deaths);
        self.heart_gem |= right.heart_gem;
    }
}


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
