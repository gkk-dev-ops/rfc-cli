use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(key)
    }

    pub async fn read(&self, key: &str) -> Result<Option<String>> {
        let path = self.path_for(key);
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => Ok(Some(content)),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(Error::CacheIo { path, source }),
        }
    }

    pub async fn is_fresh(&self, key: &str, max_age: Duration) -> Result<bool> {
        let path = self.path_for(key);
        let metadata = match tokio::fs::metadata(&path).await {
            Ok(metadata) => metadata,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(source) => return Err(Error::CacheIo { path, source }),
        };
        let modified = metadata
            .modified()
            .map_err(|source| Error::CacheIo { path, source })?;
        Ok(SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default()
            <= max_age)
    }

    pub async fn write(&self, key: &str, content: &str) -> Result<()> {
        let path = self.path_for(key);
        let parent = path.parent().unwrap_or(&self.root);
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|source| Error::CacheIo {
                path: parent.to_path_buf(),
                source,
            })?;

        let temporary = path.with_extension("tmp");
        tokio::fs::write(&temporary, content)
            .await
            .map_err(|source| Error::CacheIo {
                path: temporary.clone(),
                source,
            })?;
        tokio::fs::rename(&temporary, &path)
            .await
            .map_err(|source| Error::CacheIo { path, source })
    }

    pub async fn clear(&self) -> Result<()> {
        match tokio::fs::remove_dir_all(&self.root).await {
            Ok(()) => Ok(()),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(Error::CacheIo {
                path: self.root.clone(),
                source,
            }),
        }
    }
}
