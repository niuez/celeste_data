use celeste_save_data_rs::save_data::{ SaveData, MapCode };
use celeste_save_data_rs::map_data::GameData;
use std::collections::HashMap;

pub struct SaveDataDiff {
    stats_diffs: HashMap<MapCode, StatsDiff>,
}

impl SaveDataDiff {
    pub fn new() -> Self {
        Self {
            stats_diffs: HashMap::new(),
        }
    }
    pub fn create_diff(game_data: &GameData, before: &SaveData, after: &SaveData) -> Self {
        let mut diff = Self::new();
        for level in game_data.levels() {
            for map_data in game_data.get_level_data(level).unwrap().maps() {
                let stats_diff = match (before.map_stats.get(&map_data.code), after.map_stats.get(&map_data.code)) {
                    (None, None) => StatsDiff::Same,
                    (Some(_), None) => StatsDiff::BeforeOnly,
                    (None, Some(_)) => StatsDiff::AfterOnly,
                    (Some(before), Some(after)) => {
                        let strawberries = {
                            let b = before.total_strawberries();
                            let a = after.total_strawberries();
                            if b == a { DiffParam::Same }
                            else if b < a { DiffParam::Normal(format!("+{}", a - b)) }
                            else { DiffParam::Outlier(format!("-{}", b - a)) }
                        };
                        let best_deaths = {
                            let bsr = before.single_run_completed;
                            let asr = after.single_run_completed;
                            if bsr && asr {
                                let b = before.best_deaths;
                                let a = after.best_deaths;
                                if b == a { DiffParam::Same }
                                else if b > a { DiffParam::Normal(format!("-{}", b - a)) }
                                else { DiffParam::Outlier(format!("+{}", a - b)) }
                            }
                            else if !bsr && !asr {
                                DiffParam::Same
                            }
                            else if !bsr && asr {
                                DiffParam::Normal("new".to_string())
                            }
                            else {
                                DiffParam::Outlier("degrate".to_string())
                            }
                        };
                        let deaths = {
                            let b = before.deaths;
                            let a = after.deaths;
                            if b == a { DiffParam::Same }
                            else if b < a { DiffParam::Normal(format!("+{}", a - b)) }
                            else { DiffParam::Outlier(format!("-{}", b - a)) }
                        };
                        let clr = {
                            let bsr = before.single_run_completed;
                            let asr = after.single_run_completed;
                            if bsr && asr {
                                let b = before.best_time;
                                let a = after.best_time;
                                if b == a { DiffParam::Same }
                                else if b > a { DiffParam::Normal(format!("-{}", b - a)) }
                                else { DiffParam::Outlier(format!("+{}", a - b)) }
                            }
                            else if !bsr && !asr {
                                let bsr = before.completed;
                                let asr = after.completed;
                                if (bsr && asr) || (!bsr && !asr) {
                                    let b = before.time_played;
                                    let a = after.time_played;
                                    if b == a { DiffParam::Same }
                                    else if b > a { DiffParam::Normal(format!("-{}", b - a)) }
                                    else { DiffParam::Outlier(format!("+{}", a - b)) }
                                }
                                else if !bsr && asr {
                                    DiffParam::Normal("new".to_string())
                                }
                                else {
                                    DiffParam::Outlier("degrate".to_string())
                                }
                            }
                            else if !bsr && asr {
                                DiffParam::Normal("new".to_string())
                            }
                            else {
                                DiffParam::Outlier("degrate".to_string())
                            }
                        };
                        let fc = {
                            let bsr = before.full_clear;
                            let asr = after.full_clear;
                            if bsr && asr {
                                let b = before.best_full_clear_time;
                                let a = after.best_full_clear_time;
                                if b == a { DiffParam::Same }
                                else if b > a { DiffParam::Normal(format!("-{}", b - a)) }
                                else { DiffParam::Outlier(format!("+{}", a - b)) }
                            }
                            else if !bsr && !asr {
                                DiffParam::Same
                            }
                            else if !bsr && asr {
                                DiffParam::Normal("new".to_string())
                            }
                            else {
                                DiffParam::Outlier("degrate".to_string())
                            }
                        };
                        StatsDiff::Diff { strawberries, best_deaths, deaths, clr, fc }.same_check()
                    }
                };
                diff.stats_diffs.insert(map_data.code.clone(), stats_diff);
            }
        }
        diff
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatsDiff {
    Same,
    AfterOnly,
    BeforeOnly,
    Diff {
        strawberries: DiffParam,
        best_deaths: DiffParam,
        deaths: DiffParam,
        clr: DiffParam,
        fc: DiffParam,
    }
}

impl StatsDiff {
    pub fn same_check(self) -> Self {
        if let StatsDiff::Diff { strawberries, best_deaths, deaths, clr, fc } = self {
            if strawberries == DiffParam::Same &&
                best_deaths == DiffParam::Same &&
                deaths == DiffParam::Same &&
                clr == DiffParam::Same &&
                fc == DiffParam::Same {
                    StatsDiff::Same
                }
            else {
                StatsDiff::Diff { strawberries, best_deaths, deaths, clr, fc }
            }
        }
        else {
            self
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffParam {
    Same,
    Normal(String),
    Outlier(String),
}

impl Default for DiffParam {
    fn default() -> Self {
        DiffParam::Same
    }
}

