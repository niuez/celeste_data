use celeste_save_data_rs::save_data::*;

fn main() {
    let xml = std::fs::read_to_string("0.celeste").unwrap();
    let xml = SaveData::from_str(&xml).unwrap();
    println!("{:#?}", xml);
}
