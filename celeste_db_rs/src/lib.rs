use serde::{Deserialize, Serialize};
use mongodb::{options::ClientOptions, Client, Database, Collection};
use mongodb::bson::{ doc, oid::ObjectId };
use mongodb::options::{ FindOneAndReplaceOptions };
use celeste_save_data_rs::time::Time;
use celeste_save_data_rs::save_data::{ SaveData, AreaModeStats, MapCode, Strawberries, EntityID, };
use celeste_save_data_rs::map_data::GameData;
use futures::stream::TryStreamExt;

#[derive(Serialize, Deserialize)]
struct MapRecord {
    #[serde(rename = "_id", skip_serializing)]
    id: Option<ObjectId>,
    discord_id: String,
    level: String,
    sid: String,
    side: i64,

    // values
    completed: bool,
    single_run_completed: bool,
    full_clear: bool,
    deaths: i64,
    time_played: i64,
    best_time: i64,
    best_full_clear_time: i64,
    best_dashes: i64,
    best_deaths: i64,
    heart_gem: bool,
    strawberries: Vec<String>,
}

impl MapRecord {
    pub fn from_stats(code: &MapCode, stats: &AreaModeStats, discord_id: &str) -> Self {
        Self {
            id: None,
            discord_id: discord_id.to_string(),
            level: code.level.clone(),
            sid: code.sid.clone(),
            side: code.side as i64,

            // values
            completed: stats.completed,
            single_run_completed: stats.single_run_completed,
            full_clear: stats.full_clear,
            deaths: stats.deaths as i64,
            time_played: stats.time_played.0 as i64,
            best_time: stats.best_time.0 as i64,
            best_full_clear_time: stats.best_full_clear_time.0 as i64,
            best_dashes: stats.best_dashes as i64,
            best_deaths: stats.best_deaths as i64,
            heart_gem: stats.heart_gem,
            strawberries: stats.strawberries.entity_id.iter().map(|e| e.key.clone()).collect(),
        }
    }
    pub fn to_code_stats(&self) -> (MapCode, AreaModeStats) {
        (
            MapCode {
                level: self.level.clone(),
                sid: self.sid.clone(),
                side: self.side as usize,
            },
            AreaModeStats {
                completed: self.completed,
                single_run_completed: self.single_run_completed,
                full_clear: self.full_clear,
                deaths: self.deaths as u64,
                time_played: Time(self.time_played as u64),
                best_time: Time(self.best_time as u64),
                best_full_clear_time: Time(self.best_full_clear_time as u64),
                best_dashes: self.best_dashes as u64,
                best_deaths: self.best_deaths as u64,
                heart_gem: self.heart_gem,
                strawberries: Strawberries {
                    entity_id: self.strawberries.iter()
                        .map(|s| EntityID { key: s.clone() })
                        .collect()
                },
            },
        )
    }
}

pub struct CelesteDB {
    client: Client,
    db: Database,
    record_col: Collection<MapRecord>,
}

impl CelesteDB {
    pub async fn new() -> Result<Self, String> {
        let client_options = ClientOptions::parse("mongodb://localhost:27017").await
            .map_err(|e| format!("cant parse mongodb url {:?}", e))?;
        let client = Client::with_options(client_options)
            .map_err(|e| format!("cant connect mongodb {:?}", e))?;
        let db = client.database("celeste");
        let record_col = db.collection::<MapRecord>("record");
        Ok(CelesteDB {
            client,
            db,
            record_col,
        })
    }
    
    pub async fn update_record(&self, save_data: &SaveData, game_data: &GameData, discord_id: &str) -> Result<(), String> {
        let upsert_option = FindOneAndReplaceOptions::builder().upsert(true).build();
        for level in game_data.levels() {
            for map_data in game_data.get_level_data(level).unwrap().maps() {
                if let Some(stats) = save_data.map_stats.get(&map_data.code) {
                    let map_record = MapRecord::from_stats(&map_data.code, stats, discord_id);
                    self.record_col.find_one_and_replace(
                        doc!{
                            "discord_id": discord_id.to_string(),
                            "level": map_data.code.level.clone(),
                            "sid": map_data.code.sid.clone(),
                            "side": map_data.code.side as i64,
                        },
                        map_record,
                        upsert_option.clone(),
                    ).await
                        .map_err(|e| format!("replace error {:?}", e))?;

                }
            }
        }
        Ok(())
    }

    pub async fn get_save_data(&self, game_data: &GameData, discord_id: &str) -> Result<SaveData, String> {
        let mut save_data = SaveData::new();
        let mut cursor = self.record_col.find(doc! { "discord_id": discord_id }, None).await
            .map_err(|e| format!("find error {:?}", e))?;
        while let Some(record) = cursor.try_next().await
            .map_err(|e| format!("try next error {:?}", e))? {
                let (code, stats) = record.to_code_stats();
                save_data.map_stats.insert(code, stats);
            }
        Ok(save_data)
    }
}
