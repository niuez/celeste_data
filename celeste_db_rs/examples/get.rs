use celeste_db_rs::CelesteDB;
use celeste_save_data_rs::save_data::SaveData;
use celeste_save_data_rs::map_data::GameData;

#[tokio::main]
async fn main() {
    let db = CelesteDB::new().await.unwrap();
    let yml = std::fs::read_to_string("../maps.yaml").unwrap();
    let game_data: GameData = GameData::from_str(&yml).unwrap();
    let save_data = db.get_save_data(&game_data, "0").await.unwrap();
    println!("{:#?}", save_data);
}
