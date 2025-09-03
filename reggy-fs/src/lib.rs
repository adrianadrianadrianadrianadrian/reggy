use reggy_core::point_read_store::PointReadPersistence;
use std::fs;

pub struct FsPersistence {
    pub root_dir: String,
}

impl PointReadPersistence for FsPersistence {
    async fn read(&self, id: String) -> Result<Option<Vec<u8>>, String> {
        let path = path(&self.root_dir, &id);
        match fs::exists(&path) {
            Ok(false) => return Ok(None),
            Err(error) => return Err(error.to_string()),
            _ => {}
        }

        Ok(Some(fs::read(&path).map_err(|e| e.to_string())?))
    }

    async fn write(&self, id: String, data: &Vec<u8>) -> Result<(), String> {
        fs::write(path(&self.root_dir, &id), data).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn path(root_dir: &str, id: &str) -> String {
    format!("{}/{}", root_dir, id)
}
