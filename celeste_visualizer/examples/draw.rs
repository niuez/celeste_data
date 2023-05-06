use celeste_visualizer::{ generate_png, generate_svg_str };
use celeste_save_data_rs::save_data::SaveData;
use celeste_save_data_rs::map_data::GameData;

fn main() {
    //let xml = std::fs::read_to_string("../celeste_save_data_rs/0.celeste").unwrap();
    let xml = std::fs::read_to_string("../../Downloads/hir.celeste").unwrap();
    let save_data = SaveData::from_str(&xml).unwrap();
    let yml = std::fs::read_to_string("../maps.yaml").unwrap();
    let game_data: GameData = GameData::from_str(&yml).unwrap();
    let chart = generate_svg_str(&save_data, game_data.get_level_data("Celeste").unwrap().maps(), "en");
    println!("{}", chart.to_string());
    println!("{:?}", generate_png(&save_data, game_data.get_level_data("Celeste").unwrap().maps(), "test.png", "en"));
}
