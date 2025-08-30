//! Embedded assets for Luna
//!
//! This module provides compile-time embedded assets using rust-embed.
//! All assets are loaded from the assets directory and embedded into the binary.

use rust_embed::RustEmbed;
use std::borrow::Cow;

/// Embedded assets from the assets directory
///
/// This includes all CSS files, SVG icons, and other resources needed by Luna.
/// The assets are embedded at compile time and accessed through this struct.
#[derive(RustEmbed)]
#[folder = "assets"]
#[prefix = ""]
pub struct Assets;

impl Assets {
    /// Get an asset by its path
    ///
    /// Returns None if the asset doesn't exist.
    ///
    /// # Example
    /// ```no_run
    /// use assets::Assets;
    ///
    /// if let Some(css) = Assets::get_asset("css/buttons.css") {
    ///     let content = std::str::from_utf8(&css).unwrap();
    ///     println!("CSS content: {}", content);
    /// }
    /// ```
    pub fn get_asset(path: &str) -> Option<Cow<'static, [u8]>> {
        Self::get(path).map(|file| file.data)
    }

    /// Get an asset as a string
    ///
    /// Returns None if the asset doesn't exist or isn't valid UTF-8.
    ///
    /// # Example
    /// ```no_run
    /// use assets::Assets;
    ///
    /// if let Some(css) = Assets::get_text("css/buttons.css") {
    ///     println!("CSS content: {}", css);
    /// }
    /// ```
    pub fn get_text(path: &str) -> Option<String> {
        Self::get_asset(path).and_then(|data| String::from_utf8(data.to_vec()).ok())
    }

    /// Get an SVG icon by name
    ///
    /// This is a convenience method for accessing SVG icons.
    /// The .svg extension is added automatically.
    ///
    /// # Example
    /// ```no_run
    /// use assets::Assets;
    ///
    /// if let Some(icon) = Assets::get_icon("arrow_pointer") {
    ///     println!("Icon SVG: {}", icon);
    /// }
    /// ```
    pub fn get_icon(name: &str) -> Option<String> {
        let path = format!("svg/{}.svg", name);
        Self::get_text(&path)
    }

    /// Get a CSS file by name
    ///
    /// This is a convenience method for accessing CSS files.
    /// The .css extension is added automatically.
    ///
    /// # Example
    /// ```no_run
    /// use assets::Assets;
    ///
    /// if let Some(css) = Assets::get_css("buttons") {
    ///     println!("CSS: {}", css);
    /// }
    /// ```
    pub fn get_css(name: &str) -> Option<String> {
        let path = format!("css/{}.css", name);
        Self::get_text(&path)
    }

    /// List all available assets
    ///
    /// Returns an iterator over all embedded asset paths.
    ///
    /// # Example
    /// ```no_run
    /// use assets::Assets;
    ///
    /// for path in Assets::list() {
    ///     println!("Asset: {}", path);
    /// }
    /// ```
    pub fn list() -> impl Iterator<Item = Cow<'static, str>> {
        Self::iter()
    }

    /// List all SVG icons
    ///
    /// Returns an iterator over all SVG icon names (without extension).
    ///
    /// # Example
    /// ```no_run
    /// use assets::Assets;
    ///
    /// for icon_name in Assets::list_icons() {
    ///     println!("Icon: {}", icon_name);
    /// }
    /// ```
    pub fn list_icons() -> impl Iterator<Item = String> {
        Self::iter()
            .filter(|path| path.starts_with("svg/") && path.ends_with(".svg"))
            .map(|path| {
                path.strip_prefix("svg/")
                    .and_then(|p| p.strip_suffix(".svg"))
                    .unwrap_or("")
                    .to_string()
            })
    }

    /// List all CSS files
    ///
    /// Returns an iterator over all CSS file names (without extension).
    ///
    /// # Example
    /// ```no_run
    /// use assets::Assets;
    ///
    /// for css_name in Assets::list_css() {
    ///     println!("CSS file: {}", css_name);
    /// }
    /// ```
    pub fn list_css() -> impl Iterator<Item = String> {
        Self::iter()
            .filter(|path| path.starts_with("css/") && path.ends_with(".css"))
            .map(|path| {
                path.strip_prefix("css/")
                    .and_then(|p| p.strip_suffix(".css"))
                    .unwrap_or("")
                    .to_string()
            })
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::Assets;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_assets() {
        // Just verify we can list assets without panicking
        let assets: Vec<_> = Assets::list().collect();
        // We should have some assets
        assert!(!assets.is_empty(), "No assets found");
    }

    #[test]
    fn test_get_css() {
        // Try to load buttons.css
        let css = Assets::get_css("buttons");
        assert!(css.is_some(), "buttons.css should exist");
    }

    #[test]
    fn test_list_icons() {
        let icons: Vec<_> = Assets::list_icons().collect();
        // We should have some SVG icons
        assert!(!icons.is_empty(), "No SVG icons found");

        // Check for some known icons
        assert!(icons.contains(&"arrow_pointer".to_string()));
        assert!(icons.contains(&"frame".to_string()));
    }
}
