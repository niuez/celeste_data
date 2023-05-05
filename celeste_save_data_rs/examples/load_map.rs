use celeste_save_data_rs::map_data::*;

fn main() {
    let yml = std::fs::read_to_string("../maps.yaml").unwrap();
    let yml: GameData = serde_yaml::from_str(&yml).unwrap();
    println!("{:#?}", yml);
}
