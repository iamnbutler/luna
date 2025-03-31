//! # Asset Loading System
//!
//! This module implements the asset loading mechanism for Luna, providing access
//! to static files like images, SVGs, and other resources needed by the application.
//!
//! The implementation bridges GPUI's asset system with Luna's file structure by
//! implementing the `AssetSource` trait, allowing components to load assets through
//! GPUI's asset handling infrastructure.

use anyhow::Result;
use gpui::{AssetSource, SharedString};
use std::{fs, path::PathBuf};

/// Filesystem-based asset provider for Luna
///
/// Assets implements GPUI's AssetSource trait to load files from the local filesystem,
/// enabling access to static resources like images, SVGs, and other files needed by
/// the application. It serves as the bridge between GPUI's asset loading system
/// and Luna's file organization.
pub struct Assets {
    /// Base directory path where assets are located
    pub base: PathBuf,
}

/// Implementation of GPUI's AssetSource trait for filesystem-based assets
///
/// This implementation enables Luna to:
/// - Load asset files from the local filesystem
/// - List directory contents for dynamic asset discovery
/// - Integrate with GPUI's asset handling system
impl AssetSource for Assets {
    /// Loads a single asset from the filesystem
    ///
    /// This method reads a file from disk relative to the base directory and returns
    /// its contents as a byte array. It's used by GPUI to load assets on demand.
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        fs::read(self.base.join(path))
            .map(|data| Some(std::borrow::Cow::Owned(data)))
            .map_err(|err| err.into())
    }

    /// Lists available assets in a directory
    ///
    /// This method enumerates the contents of a directory relative to the base path,
    /// allowing for dynamic asset discovery and enumeration. It filters out any
    /// entries that can't be converted to valid string filenames.
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|err| err.into())
    }
}
