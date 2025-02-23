use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use super::LogHandler;
use crate::log_error::{Error, FormattedLog, LogFormatter, Result};

pub struct FileHandler {
    writer: Mutex<BufWriter<File>>,
    path: PathBuf,
    max_size: usize,
    rotation_count: usize,
}

impl FileHandler {
    #[cfg(debug_assertions)]
    pub fn new(path: impl Into<PathBuf>, max_size: usize, rotation_count: usize) -> Result<Self> {
        let path = path.into();

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| Error::FileCreation(e.to_string()))?;

        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
            path,
            max_size,
            rotation_count,
        })
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub fn new(
        _path: impl Into<PathBuf>,
        _max_size: usize,
        _rotation_count: usize,
    ) -> Result<Self> {
        unreachable!()
    }

    #[cfg(debug_assertions)]
    fn rotate_logs(&self) -> Result<()> {
        // Remove the oldest log file if it exists
        if self.rotation_count > 0 {
            let last_log = self
                .path
                .with_extension(format!("log.{}", self.rotation_count));
            let _ = fs::remove_file(last_log);

            // Rotate existing log files
            for i in (1..self.rotation_count).rev() {
                let src = self.path.with_extension(format!("log.{}", i));
                let dst = self.path.with_extension(format!("log.{}", i + 1));
                if src.exists() {
                    fs::rename(src, dst).map_err(|e| Error::FileRotation(e.to_string()))?;
                }
            }

            // Rename current log file
            if self.path.exists() {
                let backup = self.path.with_extension("log.1");
                fs::rename(&self.path, backup).map_err(|e| Error::FileRotation(e.to_string()))?;
            }
        }

        Ok(())
    }

    #[cfg(debug_assertions)]
    fn check_rotation(&self) -> Result<()> {
        if let Ok(metadata) = fs::metadata(&self.path) {
            if metadata.len() as usize >= self.max_size {
                self.rotate_logs()?;
            }
        }
        Ok(())
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn check_rotation(&self) -> Result<()> {
        Ok(())
    }
}

impl LogHandler for FileHandler {
    #[cfg(debug_assertions)]
    fn write_log(&self, log: &FormattedLog) -> Result<()> {
        self.check_rotation()?;

        let mut writer = self
            .writer
            .lock()
            .map_err(|_| Error::System("Failed to acquire log file lock".to_string()))?;

        writeln!(writer, "{}", log).map_err(|e| Error::FileWrite(e.to_string()))?;

        Ok(())
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn write_log(&self, _log: &FormattedLog) -> Result<()> {
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            let mut writer = self
                .writer
                .lock()
                .map_err(|_| Error::System("Failed to acquire log file lock".to_string()))?;

            writer
                .flush()
                .map_err(|e| Error::FileWrite(e.to_string()))?;
        }
        Ok(())
    }
}
