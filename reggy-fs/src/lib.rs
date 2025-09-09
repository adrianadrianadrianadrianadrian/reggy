use reggy_core::point_read_store::PointReadPersistence;
use std::{fs, path::Path};

pub struct FsPersistence {
    pub root_dir: String,
}

impl PointReadPersistence for FsPersistence {
    async fn read(&self, id: String) -> Result<Option<Vec<u8>>, String> {
        let path = path(&self.root_dir, &id);
        match fs::exists(&path) {
            Ok(true) => Ok(Some(fs::read(&path).map_err(|e| e.to_string())?)),
            Ok(false) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    async fn write(&self, id: String, data: &Vec<u8>) -> Result<(), String> {
        let raw_path = path(&self.root_dir, &id);
        let path = Path::new(&raw_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
        }

        fs::write(path, data).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn path(root_dir: &str, id: &str) -> String {
    format!("{}/{}", root_dir, id)
}
