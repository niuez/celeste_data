use celeste_save_data_rs::save_data::*;

fn main() {
    let xml = std::fs::read_to_string("0.celeste").unwrap();
    let data = SaveData::from_str(&xml).unwrap();
    let target_level = "StrawberryJam2021/5-Grandmaster";
    println!("- level: {}", target_level);
    println!("  maps:");
    let mut sorted = data.map_stats.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|e| e.0.sid.to_lowercase());
    for (code, _map) in sorted.into_iter() {
        if code.level == target_level && code.side == 0 {
            println!("    - sid: '{}'", code.sid);
            println!("      name:");
            println!("        en: ''");
            println!("      sides: [0]");
        }
    }
}
