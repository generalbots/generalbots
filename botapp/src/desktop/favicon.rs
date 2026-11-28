//! `favicon.ico` for the embedded frontend.
//!
//! The webview asks the app protocol for `/favicon.ico` on every page. The
//! frontend folder has no such file, and Tauri answers a missing asset with
//! 500, so every page logged a server error. The app icon is served instead.

use std::borrow::Cow;
use tauri::utils::assets::{AssetKey, AssetsIter, CspHash};
use tauri::{Assets, Context, Runtime};

const FAVICON_KEY: &str = "/favicon.ico";
static APP_ICON: &[u8] = include_bytes!("../../icons/icon.ico");

/// The bundled assets, plus the app icon at `/favicon.ico` when the bundle
/// has none of its own.
struct WithFavicon<R: Runtime> {
    inner: Box<dyn Assets<R>>,
}

fn is_favicon(key: &AssetKey) -> bool {
    key.as_ref().eq_ignore_ascii_case(FAVICON_KEY)
}

impl<R: Runtime> Assets<R> for WithFavicon<R> {
    fn setup(&self, app: &tauri::App<R>) {
        self.inner.setup(app);
    }

    fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
        self.inner
            .get(key)
            .or_else(|| is_favicon(key).then_some(Cow::Borrowed(APP_ICON)))
    }

    fn iter(&self) -> Box<AssetsIter<'_>> {
        self.inner.iter()
    }

    fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
        self.inner.csp_hashes(html_path)
    }
}

/// `context` with `/favicon.ico` always resolvable.
pub fn with_app_icon<R: Runtime>(mut context: Context<R>) -> Context<R> {
    let placeholder: Box<dyn Assets<R>> = Box::new(NoAssets);
    let inner = context.set_assets(placeholder);
    context.set_assets(Box::new(WithFavicon { inner }));
    context
}

/// Holds the slot while the real assets are moved into the wrapper.
struct NoAssets;

impl<R: Runtime> Assets<R> for NoAssets {
    fn get(&self, _key: &AssetKey) -> Option<Cow<'_, [u8]>> {
        None
    }

    fn iter(&self) -> Box<AssetsIter<'_>> {
        Box::new(std::iter::empty())
    }

    fn csp_hashes(&self, _html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
        Box::new(std::iter::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_root_favicon_is_substituted() {
        assert!(is_favicon(&AssetKey::from("favicon.ico")));
        assert!(is_favicon(&AssetKey::from("/favicon.ico")));
        assert!(!is_favicon(&AssetKey::from("/suite/favicon.ico")));
        assert!(!is_favicon(&AssetKey::from("/index.html")));
    }

    #[test]
    fn the_app_icon_is_a_windows_icon_file() {
        assert_eq!(&APP_ICON[..4], &[0, 0, 1, 0]);
    }

    #[test]
    fn the_bundle_keeps_its_own_assets_and_gains_the_favicon() {
        let assets: WithFavicon<tauri::Wry> = WithFavicon { inner: Box::new(NoAssets) };
        assert_eq!(
            assets.get(&AssetKey::from("/favicon.ico")).as_deref(),
            Some(APP_ICON)
        );
        assert!(assets.get(&AssetKey::from("/missing.js")).is_none());
    }
}
