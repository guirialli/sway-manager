//! Wallpaper thumbnails, preprocessed by the daemon.
//!
//! Full-res wallpapers (up to ~7MB each) decoded straight into the WebView
//! made the gallery slow and heavy. The daemon generates 320px JPEG
//! previews once per `(path, mtime)` into
//! `~/.cache/sway-manager/wallpaper_thumbs/` and serves each one as a
//! `data:` URL — the display-ready format the frontend `<img>` expects,
//! with no `asset://` scope involved.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use base64::Engine as _;

/// Thumbnail width in px (height follows aspect ratio).
pub const THUMB_WIDTH: u32 = 320;

/// Cache subdir under `~/.cache/sway-manager/`.
pub const THUMB_CACHE_DIR: &str = "wallpaper_thumbs";

/// Display-ready placeholder (1px transparent PNG) for undecodable sources.
pub const PLACEHOLDER_DATA_URL: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

/// Cache key: hash of the source path + its mtime, so edits regenerate.
fn cache_name(source: &Path, mtime_secs: u64) -> String {
    let mut hasher = DefaultHasher::new();
    source.to_string_lossy().hash(&mut hasher);
    mtime_secs.hash(&mut hasher);
    format!("{:016x}-{mtime_secs}.jpg", hasher.finish())
}

/// Generate (or reuse) a thumbnail for `source`, stored in `cache_dir`.
///
/// Returns the thumbnail path on success, `None` when the source can't be
/// decoded. Best-effort throughout: cache write failures also yield `None`.
pub fn thumbnail_for(source: &Path, cache_dir: &Path) -> Option<PathBuf> {
    let mtime = std::fs::metadata(source)
        .and_then(|m| m.modified())
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let cached = cache_dir.join(cache_name(source, mtime));
    if cached.is_file() {
        return Some(cached);
    }
    let img = image::open(source).ok()?;
    let thumb = img.thumbnail(THUMB_WIDTH, THUMB_WIDTH * 9 / 16);
    std::fs::create_dir_all(cache_dir).ok()?;
    // Write to temp + rename so concurrent readers never see a partial file.
    let tmp = cache_dir.join(format!(".tmp-{}", std::process::id()));
    thumb.save_with_format(&tmp, image::ImageFormat::Jpeg).ok()?;
    std::fs::rename(&tmp, &cached).ok()?;
    Some(cached)
}

/// Display-ready `data:` URL for `source` (cached thumbnail, base64).
///
/// Falls back to [`PLACEHOLDER_DATA_URL`] when the source can't be decoded,
/// so the grid never shows a broken image.
pub fn thumbnail_data_url(source: &Path, cache_dir: &Path) -> String {
    let bytes = thumbnail_for(source, cache_dir)
        .and_then(|p| std::fs::read(p).ok())
        .unwrap_or_default();
    if bytes.is_empty() {
        return PLACEHOLDER_DATA_URL.to_string();
    }
    let mut url = String::from("data:image/jpeg;base64,");
    base64::engine::general_purpose::STANDARD.encode_string(&bytes, &mut url);
    url
}

/// Default cache dir: `~/.cache/sway-manager/wallpaper_thumbs/`.
pub fn default_cache_dir(home: &str) -> PathBuf {
    Path::new(home)
        .join(".cache")
        .join("sway-manager")
        .join(THUMB_CACHE_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 16:9 landscape keeps width 320.
    #[test]
    fn generates_320px_thumbnail() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("wall.png");
        let big = image::RgbImage::from_fn(1920, 1080, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        big.save(&src).unwrap();
        let cache = dir.path().join("cache");

        let thumb = thumbnail_for(&src, &cache).expect("thumbnail");
        let decoded = image::open(&thumb).unwrap();
        assert_eq!(decoded.width(), THUMB_WIDTH);
        assert!(decoded.height() <= THUMB_WIDTH * 9 / 16 + 1);
        assert!(thumb.starts_with(&cache));
    }

    /// Second call reuses the cached file (same path, no regen).
    #[test]
    fn reuses_cached_thumbnail() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("wall.png");
        image::RgbImage::new(800, 600).save(&src).unwrap();
        let cache = dir.path().join("cache");

        let first = thumbnail_for(&src, &cache).unwrap();
        let first_meta = std::fs::metadata(&first).unwrap();
        let second = thumbnail_for(&src, &cache).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            std::fs::metadata(&second).unwrap().modified().unwrap(),
            first_meta.modified().unwrap()
        );
    }

    /// Corrupt source → None (caller falls back to full image).
    #[test]
    fn corrupt_source_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("broken.png");
        std::fs::write(&src, b"not an image").unwrap();
        assert!(thumbnail_for(&src, &dir.path().join("cache")).is_none());
    }

    /// Missing source → None, no panic.
    #[test]
    fn missing_source_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            thumbnail_for(&dir.path().join("nope.png"), &dir.path().join("cache")).is_none()
        );
    }

    /// Data URL decodes back to the cached JPEG bytes.
    #[test]
    fn data_url_roundtrips_cached_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("wall.png");
        image::RgbImage::from_fn(640, 360, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 64])
        })
        .save(&src)
        .unwrap();
        let cache = dir.path().join("cache");

        let url = thumbnail_data_url(&src, &cache);
        let b64 = url.strip_prefix("data:image/jpeg;base64,").expect("prefix");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .expect("base64");
        let cached = thumbnail_for(&src, &cache).expect("cached");
        assert_eq!(decoded, std::fs::read(cached).unwrap());
    }

    /// Undecodable source → placeholder, never empty/broken.
    #[test]
    fn corrupt_source_yields_placeholder() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("broken.png");
        std::fs::write(&src, b"not an image").unwrap();
        assert_eq!(
            thumbnail_data_url(&src, &dir.path().join("cache")),
            PLACEHOLDER_DATA_URL
        );
    }
}
