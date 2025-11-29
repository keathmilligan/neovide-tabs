//! Icon loading and caching for profile icons.
//!
//! Loads PNG icons from `~/.config/neovide-tabs/icons/` and caches them
//! as Win32 HBITMAP handles for efficient rendering.
//!
//! Note: This module uses thread-local storage since Win32 GDI handles
//! (HBITMAP) are not thread-safe and should not cross thread boundaries.

#![cfg(target_os = "windows")]

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, HBITMAP, HGDIOBJ, ReleaseDC, SetDIBits,
};

use crate::config::icons_dir_path;

/// Size of icons in the tab bar (16x16 pixels)
pub const ICON_SIZE: i32 = 16;

/// A cached icon bitmap
pub struct CachedIcon {
    /// The Win32 bitmap handle
    pub hbitmap: HBITMAP,
    /// Original width of the icon
    #[allow(dead_code)]
    pub width: i32,
    /// Original height of the icon
    #[allow(dead_code)]
    pub height: i32,
}

impl Drop for CachedIcon {
    fn drop(&mut self) {
        unsafe {
            if !self.hbitmap.is_invalid() {
                let _ = DeleteObject(HGDIOBJ(self.hbitmap.0));
            }
        }
    }
}

/// Icon cache storing loaded bitmaps by filename (thread-local)
struct IconCache {
    cache: HashMap<String, Option<CachedIcon>>,
    icons_dir: Option<PathBuf>,
    fallback_icon: Option<CachedIcon>,
}

impl IconCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            icons_dir: icons_dir_path(),
            fallback_icon: None,
        }
    }

    /// Get or load an icon by filename
    fn get_or_load(&mut self, filename: &str) -> Option<HBITMAP> {
        // Check if already cached
        if !self.cache.contains_key(filename) {
            // Try to load the icon
            let icon = self.load_icon(filename);
            self.cache.insert(filename.to_string(), icon);
        }

        self.cache
            .get(filename)
            .and_then(|opt| opt.as_ref())
            .map(|icon| icon.hbitmap)
    }

    /// Load an icon from the icons directory
    fn load_icon(&self, filename: &str) -> Option<CachedIcon> {
        let icons_dir = self.icons_dir.as_ref()?;
        let path = icons_dir.join(filename);

        if !path.exists() {
            eprintln!("Icon file not found: {:?}", path);
            return None;
        }

        load_png_as_bitmap(&path)
    }

    /// Get the fallback icon (creates it if needed)
    fn get_fallback(&mut self) -> Option<HBITMAP> {
        if self.fallback_icon.is_none() {
            self.fallback_icon = create_fallback_icon();
        }
        self.fallback_icon.as_ref().map(|icon| icon.hbitmap)
    }
}

// Thread-local icon cache (Win32 GDI handles are not thread-safe)
thread_local! {
    static ICON_CACHE: RefCell<IconCache> = RefCell::new(IconCache::new());
}

/// Get an icon bitmap handle for the given filename.
/// Returns a fallback icon if the specified icon cannot be loaded.
pub fn get_icon_bitmap(filename: &str) -> Option<HBITMAP> {
    ICON_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        // Try to get the requested icon
        if let Some(hbitmap) = cache.get_or_load(filename) {
            return Some(hbitmap);
        }

        // Fall back to default icon
        cache.get_fallback()
    })
}

/// Load a PNG file and convert it to a Win32 HBITMAP
fn load_png_as_bitmap(path: &Path) -> Option<CachedIcon> {
    // Load the image using the image crate
    let img = image::open(path).ok()?;

    // Resize to ICON_SIZE x ICON_SIZE
    let img = img.resize_exact(
        ICON_SIZE as u32,
        ICON_SIZE as u32,
        image::imageops::FilterType::Lanczos3,
    );

    // Convert to RGBA8
    let rgba = img.to_rgba8();
    let width = rgba.width() as i32;
    let height = rgba.height() as i32;

    // Create the bitmap
    create_bitmap_from_rgba(&rgba, width, height)
}

/// Create a Win32 HBITMAP from RGBA pixel data
fn create_bitmap_from_rgba(rgba: &image::RgbaImage, width: i32, height: i32) -> Option<CachedIcon> {
    unsafe {
        // Get a device context for the screen
        let screen_dc = GetDC(HWND::default());
        if screen_dc.is_invalid() {
            return None;
        }

        // Create a compatible DC
        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc.is_invalid() {
            ReleaseDC(HWND::default(), screen_dc);
            return None;
        }

        // Create a 32-bit bitmap
        let hbitmap = CreateCompatibleBitmap(screen_dc, width, height);
        if hbitmap.is_invalid() {
            let _ = DeleteDC(mem_dc);
            ReleaseDC(HWND::default(), screen_dc);
            return None;
        }

        // Set up bitmap info for 32-bit BGRA
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Top-down DIB (negative height)
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default()],
        };

        // Convert RGBA to BGRA (Win32 expects BGRA)
        let mut bgra_data: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
        for pixel in rgba.pixels() {
            bgra_data.push(pixel[2]); // B
            bgra_data.push(pixel[1]); // G
            bgra_data.push(pixel[0]); // R
            bgra_data.push(pixel[3]); // A
        }

        // Set the bitmap bits
        let result = SetDIBits(
            mem_dc,
            hbitmap,
            0,
            height as u32,
            bgra_data.as_ptr() as *const std::ffi::c_void,
            &bmi,
            DIB_RGB_COLORS,
        );

        // Clean up DCs
        let _ = DeleteDC(mem_dc);
        ReleaseDC(HWND::default(), screen_dc);

        if result == 0 {
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            return None;
        }

        Some(CachedIcon {
            hbitmap,
            width,
            height,
        })
    }
}

/// Create a simple fallback icon (a colored square)
fn create_fallback_icon() -> Option<CachedIcon> {
    // Create a simple 16x16 green square as fallback
    let mut rgba = image::RgbaImage::new(ICON_SIZE as u32, ICON_SIZE as u32);

    // Fill with a dark green color (Neovim-ish)
    for pixel in rgba.pixels_mut() {
        *pixel = image::Rgba([87, 166, 74, 255]); // Green
    }

    // Add a simple border
    for x in 0..ICON_SIZE as u32 {
        rgba.put_pixel(x, 0, image::Rgba([60, 120, 50, 255]));
        rgba.put_pixel(x, (ICON_SIZE - 1) as u32, image::Rgba([60, 120, 50, 255]));
    }
    for y in 0..ICON_SIZE as u32 {
        rgba.put_pixel(0, y, image::Rgba([60, 120, 50, 255]));
        rgba.put_pixel((ICON_SIZE - 1) as u32, y, image::Rgba([60, 120, 50, 255]));
    }

    create_bitmap_from_rgba(&rgba, ICON_SIZE, ICON_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icons_dir_path() {
        let path = icons_dir_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("icons"));
    }
}
