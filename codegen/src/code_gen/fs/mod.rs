//! Abstract filesystem for code generation
//!
//! Provides both a real filesystem implementation and an in-memory
//! implementation for testing.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::Result;

/// Abstract filesystem trait
pub trait FileSystem: Send + Sync {
    /// Write content to a file, creating parent directories as needed
    fn write_file(&self, path: &str, content: &[u8]) -> Result<()>;

    /// Append content to a file, creating it if it doesn't exist
    fn append_file(&self, path: &str, content: &[u8]) -> Result<()>;

    /// Read a file's contents
    fn read_file(&self, path: &str) -> Result<Vec<u8>>;

    /// Check if a file exists
    fn exists(&self, path: &str) -> bool;

    /// Create a directory and all parent directories
    fn create_dir_all(&self, path: &str) -> Result<()>;

    /// Recursively remove a directory and everything under it.
    ///
    /// A missing directory is not an error, so `clean` followed by a generate
    /// works on a fresh checkout.
    fn remove_dir_all(&self, path: &str) -> Result<()>;
}

/// Real filesystem implementation
#[derive(Debug, Default)]
pub struct RealFileSystem;

impl RealFileSystem {
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for RealFileSystem {
    fn write_file(&self, path: &str, content: &[u8]) -> Result<()> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(path)?;
        file.write_all(content)?;
        Ok(())
    }

    fn append_file(&self, path: &str, content: &[u8]) -> Result<()> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(content)?;
        Ok(())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        Ok(fs::read(path)?)
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn create_dir_all(&self, path: &str) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }

    fn remove_dir_all(&self, path: &str) -> Result<()> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// In-memory filesystem for testing
#[derive(Debug, Clone)]
pub struct MemoryFileSystem {
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get all files in the filesystem (for testing)
    #[allow(clippy::unwrap_used)]
    pub fn files(&self) -> HashMap<String, Vec<u8>> {
        self.files.read().unwrap().clone()
    }

    /// Get content of a file as a string (for testing)
    #[allow(clippy::unwrap_used)]
    pub fn get_string(&self, path: &str) -> Option<String> {
        self.files
            .read()
            .unwrap()
            .get(path)
            .map(|b| String::from_utf8_lossy(b).to_string())
    }
}

impl Default for MemoryFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::unwrap_used)]
impl FileSystem for MemoryFileSystem {
    fn write_file(&self, path: &str, content: &[u8]) -> Result<()> {
        let mut files = self.files.write().unwrap();
        files.insert(path.to_string(), content.to_vec());
        Ok(())
    }

    fn append_file(&self, path: &str, content: &[u8]) -> Result<()> {
        let mut files = self.files.write().unwrap();
        let entry = files.entry(path.to_string()).or_default();
        entry.extend_from_slice(content);
        Ok(())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let files = self.files.read().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", path))
    }

    fn exists(&self, path: &str) -> bool {
        self.files.read().unwrap().contains_key(path)
    }

    fn create_dir_all(&self, _path: &str) -> Result<()> {
        // No-op for memory filesystem
        Ok(())
    }

    fn remove_dir_all(&self, path: &str) -> Result<()> {
        let prefix = format!("{}/", path.trim_end_matches('/'));
        let mut files = self.files.write().unwrap();
        files.retain(|p, _| p != path && !p.starts_with(&prefix));
        Ok(())
    }
}

/// Writer that writes to abstract filesystem
pub struct FsWriter {
    fs: Arc<dyn FileSystem>,
    path: String,
    buffer: Vec<u8>,
    append: bool,
}

impl FsWriter {
    pub fn new(fs: Arc<dyn FileSystem>, path: String, append: bool) -> Self {
        Self {
            fs,
            path,
            buffer: Vec::new(),
            append,
        }
    }
}

impl Write for FsWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let result = if self.append {
            self.fs.append_file(&self.path, &self.buffer)
        } else {
            self.fs.write_file(&self.path, &self.buffer)
        };

        result.map_err(io::Error::other)?;
        self.buffer.clear();
        Ok(())
    }
}

impl Drop for FsWriter {
    fn drop(&mut self) {
        // Flush on drop, ignore errors
        let _ = self.flush();
    }
}
