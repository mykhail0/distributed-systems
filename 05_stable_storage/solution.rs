use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::fs::{self, File};

#[async_trait::async_trait]
pub trait StableStorage: Send + Sync {
    /// Stores `value` under `key`.
    ///
    /// Detailed requirements are specified in the description of the assignment.
    async fn put(&mut self, key: &str, value: &[u8]) -> Result<(), String>;

    /// Retrieves value stored under `key`.
    ///
    /// Detailed requirements are specified in the description of the assignment.
    async fn get(&self, key: &str) -> Option<Vec<u8>>;

    /// Removes `key` and the value stored under it.
    ///
    /// Detailed requirements are specified in the description of the assignment.
    async fn remove(&mut self, key: &str) -> bool;
}

struct Storage {
    root_dir: PathBuf,
    tmp_dir: PathBuf,
    // INVARIANT: if there is an entry here, then a file for sure exists
    key_files: HashMap<String, TempFilePair>,
}

// A file and a temporary file associated with it.
struct TempFilePair {
    tmp: PathBuf,
    file: PathBuf,
}

#[async_trait::async_trait]
impl StableStorage for Storage {
    async fn put(&mut self, key: &str, value: &[u8]) -> Result<(), String> {
        if value.len() > 65535 || key.len() > 255 {
            return Err(String::from("Invalid storage data."));
        }

        let key_str = String::from(key);

        let r = self.key_files.get(&key_str);

        if let Some(file) = r {
            self.atomic_write(file, value).await;
        } else {
            let file = self.new_file(key);
            self.atomic_write(&file, value).await;
            self.key_files.insert(key_str, file);
        }

        Ok(())
    }

    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(file) = self.key_files.get(key) {
            return Some(fs::read(&file.file).await.expect("File should exist"));
        }
        let file = self.root_dir.join(file_hash(key));
        match fs::read(&file).await {
            Ok(data) => Some(data),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => panic!("Failed to read file: {e}"),
        }
    }

    async fn remove(&mut self, key: &str) -> bool {
        let file = self
            .key_files
            .remove(key)
            .unwrap_or_else(|| self.new_file(key));

        // Maybe there was some unsuccessful attempt to overwrite,
        // (even though a value file exists).
        match fs::remove_file(file.tmp).await {
            Ok(()) => sync(&self.tmp_dir).await,
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => panic!("{e}"),
            _ => {}
        };

        match fs::remove_file(file.file).await {
            Ok(()) => {
                sync(&self.root_dir).await;
                true
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => panic!("{e}"),
            _ => false,
        }
    }
}

impl Storage {
    async fn atomic_write(&self, file: &TempFilePair, value: &[u8]) {
        fs::write(&file.tmp, value).await.unwrap();
        sync(&file.tmp).await;
        fs::rename(&file.tmp, &file.file).await.unwrap();
        sync(&self.root_dir).await;
    }

    fn new_file(&self, key: &str) -> TempFilePair {
        let sha = file_hash(key);
        TempFilePair {
            tmp: self.tmp_dir.join(sha.clone()),
            file: self.root_dir.join(sha),
        }
    }
}

/// Creates a new instance of stable storage.
pub async fn build_stable_storage(root_storage_dir: PathBuf) -> Box<dyn StableStorage> {
    let root_dir = root_storage_dir;
    let tmp = root_dir.join(Path::new("tmp"));

    // Delete everything in `tmp`.
    if fs::try_exists(&tmp)
        .await
        .expect("Can't check a path's existence")
    {
        fs::remove_dir_all(tmp.clone()).await.unwrap();
    }

    let key_files = HashMap::new();
    fs::create_dir_all(tmp.clone())
        .await
        .expect("Can't create tmp directory");
    Box::new(Storage {
        root_dir,
        tmp_dir: tmp,
        key_files,
    })
}

fn file_hash(key: &str) -> String {
    let sha = Sha256::new_with_prefix(key).finalize();
    URL_SAFE.encode(sha)
}

async fn sync(entity: &Path) {
    File::open(entity).await.unwrap().sync_data().await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::timeout;
    use tempfile::tempdir;

    #[tokio::test]
    #[timeout(500)]
    #[should_panic = "Failed to read file: Is a directory (os error 21)"]
    async fn fail_get() {
        let root_storage_dir = tempdir().unwrap();
        let storage = build_stable_storage(root_storage_dir.path().to_path_buf()).await;
        fs::create_dir(root_storage_dir.path().join(file_hash("d")))
            .await
            .unwrap();
        assert!(storage.get("d").await.is_none());
    }
}
