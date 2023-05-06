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
                })
            }
        }
        codes.into_iter()
    }
}

pub struct MapData {
    pub code: MapCode,
    pub name: Name,
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
