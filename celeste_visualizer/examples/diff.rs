use celeste_visualizer::diff::{ diff_svg_chart };
use celeste_save_data_rs::save_data::SaveData;
use celeste_save_data_rs::map_data::GameData;

fn main() {
    //let xml = std::fs::read_to_string("../celeste_save_data_rs/0.celeste").unwrap();
    let xml = std::fs::read_to_string("0.celeste").unwrap();
    let before = SaveData::from_str(&xml).unwrap();
    let xml = std::fs::read_to_string("1.celeste").unwrap();
    let after = SaveData::from_str(&xml).unwrap();

    let yml = std::fs::read_to_string("../maps.yaml").unwrap();
    let game_data: GameData = GameData::from_str(&yml).unwrap();
    let (chart, _w, _h) = diff_svg_chart(&game_data, &before, &after, "en");
    println!("{}", chart.to_string());
}
