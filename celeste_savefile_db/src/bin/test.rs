use celeste_savefile_db::{ CelesteSavefileDB, Savefile };

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let db = CelesteSavefileDB::new().await.expect("cant new");
    let sf1 = Savefile { 
        discord_id: "a".to_string(),
        filename: "1.celeste".to_string(),
        xml: "hoge".to_string(),
    };
    let sf2 = Savefile { 
        discord_id: "a".to_string(),
        filename: "1.celeste".to_string(),
        xml: "nya".to_string(),
    };
    db.update_savefile(sf1).await.expect("cant update sf1");
    eprintln!("{:?}", db.get_savefiles("a").await.expect("cant get"));
    db.update_savefile(sf2).await.expect("cant update sf2");
    eprintln!("{:?}", db.get_savefiles("a").await.expect("cant get"));
    //db.debug_all_savefiles().await.expect("???");
    Ok(())
}
