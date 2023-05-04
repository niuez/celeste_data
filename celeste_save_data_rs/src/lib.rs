pub mod save_data;
use quick_xml::de::from_str;

pub fn load_save_data(xml_string: &str) -> Result<save_data::SaveData, String> {
    from_str(xml_string).map_err(|e| format!("cannot load save data, \"{:?}\"", e))
}
