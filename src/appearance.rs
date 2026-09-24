//! Personalisation: the picture behind the Fleet server list.
//!
//! A chosen picture is copied (shrunk to at most 1920px wide, re-encoded as
//! PNG) into `~/.config/crow/backgrounds/`, so moving or deleting the
//! original doesn't break it. GPUI can't blur, so each blur level is rendered
//! once with the `image` crate and cached next to it. Every stored picture
//! gets a unique name, which keeps GPUI's image cache from showing a stale
//! one.

use std::path::{Path, PathBuf};

/// Longest side kept for a background picture.
const MAX_WIDTH: u32 = 1920;

pub fn backgrounds_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".config").join("crow").join("backgrounds")
}

/// Copies `source` into the backgrounds folder, shrunk and as PNG, and
/// returns the stored path.
pub fn store_background(source: &Path) -> Result<PathBuf, String> {
    let img = image::open(source).map_err(|e| format!("{} isn't a picture Crow can read: {e}", source.display()))?;
    let img = if img.width() > MAX_WIDTH { img.resize(MAX_WIDTH, MAX_WIDTH * 4, image::imageops::FilterType::Triangle) } else { img };
    let dir = backgrounds_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("can't create {}: {e}", dir.display()))?;
    let dest = dir.join(format!("fleet-{}.png", chrono::Utc::now().timestamp_millis()));
    img.save(&dest).map_err(|e| format!("can't save {}: {e}", dest.display()))?;
    Ok(dest)
}

/// The picture to draw for `stored` at `blur`: the stored one when blur is 0,
/// else a cached blurred copy (rendered now if missing).
pub fn rendered_background(stored: &Path, blur: u32) -> Result<PathBuf, String> {
    if blur == 0 {
        return Ok(stored.to_path_buf());
    }
    let stem = stored.file_stem().and_then(|s| s.to_str()).unwrap_or("fleet");
    let cached = stored.with_file_name(format!("{stem}-blur{blur}.png"));
    if cached.exists() {
        return Ok(cached);
    }
    let img = image::open(stored).map_err(|e| format!("can't read {}: {e}", stored.display()))?;
    // Blur a half-size copy (four times less work; a blurred picture loses
    // nothing), then scale back up.
    let (w, h) = (img.width(), img.height());
    let small = img.resize_exact((w / 2).max(1), (h / 2).max(1), image::imageops::FilterType::Triangle);
    let blurred = small.blur(blur as f32 / 2.0).resize_exact(w, h, image::imageops::FilterType::Triangle);
    blurred.save(&cached).map_err(|e| format!("can't save {}: {e}", cached.display()))?;
    Ok(cached)
}

/// Removes stored pictures (and their blurred copies) other than `keep`.
pub fn prune_backgrounds(keep: Option<&Path>) {
    let keep_stem = keep.and_then(|k| k.file_stem()).and_then(|s| s.to_str()).map(str::to_string);
    let Ok(entries) = std::fs::read_dir(backgrounds_dir()) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("fleet-") && keep_stem.as_ref().is_none_or(|k| !name.starts_with(k.as_str())) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blurred_copies_are_cached_next_to_the_picture() {
        let dir = std::env::temp_dir().join(format!("crow-bg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("fleet-1.png");
        let mut img = image::RgbImage::new(64, 32);
        for (x, _, p) in img.enumerate_pixels_mut() {
            *p = image::Rgb([if x < 32 { 255 } else { 0 }, 0, 0]);
        }
        img.save(&src).unwrap();
        assert_eq!(rendered_background(&src, 0).unwrap(), src, "no blur: the picture itself");
        let blurred = rendered_background(&src, 8).unwrap();
        assert_eq!(blurred, dir.join("fleet-1-blur8.png"));
        let b = image::open(&blurred).unwrap().to_rgb8();
        assert_eq!((b.width(), b.height()), (64, 32), "same size as the picture");
        let edge = b.get_pixel(32, 16)[0];
        assert!(edge > 20 && edge < 235, "the hard red/black edge is softened, got {edge}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
