use serde::Deserialize;
use std::collections::HashMap;
use crate::save_data::MapCode;

#[derive(Deserialize, Debug)]
pub struct GameData(Vec<LevelData>);

impl GameData {
    pub fn from_str(yml_str: &str) -> Result<Self, String> {
        serde_yaml::from_str(yml_str).map_err(|e| format!("cannot parse yaml: {:?}", e))
    }
    pub fn levels<'a>(&'a self) -> impl Iterator<Item=&'a str> {
        self.0.iter().map(|d| d.level.as_str())
    }
    pub fn get_level_data<'a, 'b>(&'a self, level: &'b str) -> Option<&'a LevelData> {
        self.0.iter().find(|d| d.level == level)
    }
}

#[derive(Deserialize, Debug)]
pub struct LevelData {
    pub level: String,
    maps: Vec<MapDataRaw>,
}

impl LevelData {
    pub fn maps(&self) -> impl ExactSizeIterator + Iterator<Item=MapData> {
        let mut codes = Vec::new();
        for map in self.maps.iter() {
            for side in map.sides.iter() {
                codes.push( MapData {
                    code: MapCode {
                        level: self.level.clone(),
                        sid: map.sid.clone(),
                        side: *side,
                    },
                    name: map.name.clone(),
                    multi_side: map.sides.len() > 1,
                })
            }
        }
        codes.into_iter()
    }
}

#[derive(Debug)]
pub struct MapData {
    pub code: MapCode,
    pub name: Name,
    pub multi_side: bool,
}

impl MapData {
    fn side_name(&self) -> String {
        if self.multi_side {
            ["-A", "-B", "-C"][self.code.side].to_owned()
        }
        else {
            "".to_owned()
        }
    }
    pub fn get_name(&self) -> String {
        format!("{}{}", self.name.get_name(), self.side_name())
    }
    pub fn try_local_name<'a, 'b>(&'a self, lang: &'b str) -> String {
        format!("{}{}", self.name.try_local_name(lang), self.side_name())
    }
}

#[derive(Deserialize, Debug)]
struct MapDataRaw {
    sid: String,
    name: Name,
    sides: Vec<usize>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Name(HashMap<String, String>);

impl Name {
    pub fn get_name(&self) -> &str {
        self.0.get("en").unwrap().as_str()
    }
    pub fn try_local_name<'a, 'b>(&'a self, lang: &'b str) -> &'a str {
        self.0.get(lang).unwrap().as_str()
    }
}
