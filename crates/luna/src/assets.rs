//! Implements the asset loading mechanism for Luna.

use anyhow::Result;
use gpui::{AssetSource, SharedString};

/// Embedded asset provider implementing [`AssetSource`]
///
/// This struct bridges between GPUI's AssetSource trait and our
/// embedded assets from the assets crate.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Ok(assets::Assets::get_asset(path))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        // Filter assets that start with the given path
        let prefix = if path.is_empty() {
            String::new()
        } else if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };

        let items: Vec<SharedString> = assets::Assets::list()
            .filter(|asset_path| asset_path.starts_with(&prefix))
            .map(|asset_path| {
                // Get just the filename after the prefix
                let path_str = asset_path.to_string();
                path_str[prefix.len()..]
                    .split('/')
                    .next()
                    .unwrap_or("")
                    .to_string()
                    .into()
            })
            .filter(|name: &SharedString| !name.is_empty())
            .collect();

        Ok(items)
    }
}
