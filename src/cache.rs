use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime},
};

use crate::error::{Error, Result};

static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

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

        let counter = TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temporary = path.with_extension(format!("{}.{counter}.tmp", std::process::id()));
        tokio::fs::write(&temporary, content)
            .await
            .map_err(|source| Error::CacheIo {
                path: temporary.clone(),
                source,
            })?;
        if let Err(source) = tokio::fs::rename(&temporary, &path).await {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(Error::CacheIo { path, source });
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn concurrent_writers_use_independent_temporary_files() {
        let directory = tempfile::tempdir().unwrap();
        let cache = Cache::new(directory.path().to_path_buf());

        let (first, second) = tokio::join!(
            cache.write("documents/rfc9110.txt", "first"),
            cache.write("documents/rfc9110.txt", "second")
        );

        first.unwrap();
        second.unwrap();
        let content = cache.read("documents/rfc9110.txt").await.unwrap().unwrap();
        assert!(content == "first" || content == "second");
    }
}
