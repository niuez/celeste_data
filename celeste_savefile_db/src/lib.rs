use futures::{AsyncWriteExt, AsyncReadExt};
use futures::stream::TryStreamExt;
use mongodb::{options::ClientOptions, Client, Database, Collection};
use mongodb::bson::{ doc, oid::ObjectId };
use mongodb::options::{ FindOneAndReplaceOptions, GridFsBucketOptions, GridFsUploadOptions };
use mongodb::GridFsBucket;

pub struct CelesteSavefileDB {
    client: Client,
    db: Database,
    gridfs: GridFsBucket,
}

#[derive(Debug, Clone)]
pub struct Savefile {
    pub discord_id: String,
    pub filename: String,
    pub xml: String,
}

impl CelesteSavefileDB {
    pub async fn new() -> Result<Self, String> {
        let client_options = ClientOptions::parse("mongodb://localhost:27017").await
            .map_err(|e| format!("cant parse mongodb url {:?}", e))?;
        let client = Client::with_options(client_options)
            .map_err(|e| format!("cant connect mongodb {:?}", e))?;
        let db = client.database("celeste_savefile");
        let opts = GridFsBucketOptions::builder()
            .bucket_name("fs".to_string())
            .build();
        let gridfs = db.gridfs_bucket(opts);
        Ok(Self {
            client,
            db,
            gridfs,
        })
    }

    pub async fn debug_all_savefiles(&self) -> Result<(), String> {
        let mut cursor = self.gridfs.find(doc!{}, None)
            .await.map_err(|e| format!("cant find {:?}", e))?;
        while let Some(result) = cursor.try_next()
            .await.map_err(|e| format!("cant try next {:?}", e))? {
                eprintln!("{:?}", result);
        }
        Ok(())
    }

    pub async fn get_savefiles(&self, discord_id: &str) -> Result<Vec<Savefile>, String> {
        let mut cursor = self.gridfs.find(doc!{ "metadata.discord_id": discord_id }, None)
            .await.map_err(|e| format!("cant find {:?}", e))?;
        let mut savefiles = vec![];
        while let Some(result) = cursor.try_next()
            .await.map_err(|e| format!("cant try next {:?}", e))? {
                let mut stream = self.gridfs.open_download_stream(result.id)
                    .await.map_err(|e| format!("cant open download stream {:?}", e))?;
                let mut xml = Vec::new();
                let _ = stream.read_to_end(&mut xml)
                    .await.map_err(|e| format!("fail read_to_end {:?}", e))?;
                savefiles.push(Savefile {
                    discord_id: discord_id.to_string(),
                    filename: result.metadata.unwrap().get_str("filename").unwrap().to_string(),
                    xml: String::from_utf8(xml.to_vec()).unwrap(),
                });
        }
        Ok(savefiles)
    }

    pub async fn update_savefile(&self, savefile: Savefile) -> Result<(), String> {
        let fsname = format!("{}_{}", savefile.discord_id, savefile.filename);
        {
            // delete if exist
            let mut cursor = self.gridfs.find(doc!{ "filename": fsname.clone() }, None)
                .await.map_err(|e| format!("cant find {:?}", e))?;
            while let Some(result) = cursor.try_next()
                .await.map_err(|e| format!("cant try next {:?}", e))? {
                    self.gridfs.delete(result.id)
                        .await.map_err(|e| format!("cant delete {:?}", e))?;
            }
        }
        {
            let opts = GridFsUploadOptions::builder()
                .metadata( Some(doc!{ "discord_id": savefile.discord_id, "filename": savefile.filename }) )
                .build();
            let mut stream = self.gridfs.open_upload_stream(fsname, opts);
            stream.write_all(savefile.xml.as_bytes()).await
                .map_err(|e| format!("write_all error {:?}", e))?;
            stream.close().await.map_err(|e| format!("cant close stream {:?}", e))?;
        }
        Ok(())
    }
}
