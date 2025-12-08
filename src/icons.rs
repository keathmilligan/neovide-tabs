//! Icon loading and caching for profile icons.
//!
//! Two icons are bundled into the executable:
//! - `neovide.png` - the default tab icon (Neovide logo for profiles)
//! - `neovide-tabs.png` - the application window icon (for taskbar/Alt-Tab)
//!
//! Both are extracted to `~/.local/share/neovide-tabs/` at runtime.
//! User-defined icons are loaded from full paths specified in the config.
//!
//! Note: This module uses thread-local storage since Win32 GDI handles
//! (HBITMAP) are not thread-safe and should not cross thread boundaries.

#![cfg(target_os = "windows")]

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC,
    CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, HBITMAP, HGDIOBJ, ReleaseDC,
    SetDIBits,
};
use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO};

use crate::config::{APP_ICON, DEFAULT_ICON, data_dir_path};

/// Size of icons in the tab bar (16x16 pixels)
pub const ICON_SIZE: i32 = 16;

/// Size of window icons (32x32 pixels for better quality in taskbar/Alt-Tab)
pub const WINDOW_ICON_SIZE: i32 = 32;

/// The bundled default tab icon - Neovide logo (embedded at compile time)
const BUNDLED_TAB_ICON_BYTES: &[u8] = include_bytes!("../neovide.png");

/// The bundled application window icon (embedded at compile time)
const BUNDLED_APP_ICON_BYTES: &[u8] = include_bytes!("../neovide-tabs.png");

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

/// Icon cache storing loaded bitmaps by path/filename (thread-local)
struct IconCache {
    cache: HashMap<String, Option<CachedIcon>>,
    data_dir: Option<PathBuf>,
    fallback_icon: Option<CachedIcon>,
}

impl IconCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            data_dir: data_dir_path(),
            fallback_icon: None,
        }
    }

    /// Get or load an icon by path or filename.
    /// For the default icon (neovide-tabs.png), loads from data directory.
    /// For user icons, treats the string as a full path.
    fn get_or_load(&mut self, icon_path: &str) -> Option<HBITMAP> {
        // Check if already cached
        if !self.cache.contains_key(icon_path) {
            // Try to load the icon
            let icon = self.load_icon(icon_path);
            self.cache.insert(icon_path.to_string(), icon);
        }

        self.cache
            .get(icon_path)
            .and_then(|opt| opt.as_ref())
            .map(|icon| icon.hbitmap)
    }

    /// Load an icon from the appropriate location.
    /// Default icon: loaded from data directory (~/.local/share/neovide-tabs/)
    /// User icon: loaded from the full path specified
    /// Supports both PNG and SVG formats (detected by file extension).
    fn load_icon(&self, icon_path: &str) -> Option<CachedIcon> {
        let path = if icon_path == DEFAULT_ICON {
            // Default icon - load from data directory
            let data_dir = self.data_dir.as_ref()?;
            data_dir.join(icon_path)
        } else {
            // User-defined icon - treat as full path
            PathBuf::from(icon_path)
        };

        if !path.exists() {
            eprintln!("Icon file not found: {:?}", path);
            return None;
        }

        // Check file extension to determine loader
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("svg") => load_svg_as_bitmap(&path),
            _ => load_png_as_bitmap(&path),
        }
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

/// Ensure bundled icons are extracted to the data directory.
/// Creates the directory if it doesn't exist.
/// Does NOT overwrite if files already exist.
pub fn ensure_default_icon_extracted() {
    let Some(data_dir) = data_dir_path() else {
        eprintln!("Warning: Could not determine data directory path");
        return;
    };

    // Create data directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&data_dir) {
        eprintln!(
            "Warning: Failed to create data directory {:?}: {}",
            data_dir, e
        );
        return;
    }

    // Extract the default tab icon (neovide.png)
    let tab_icon_path = data_dir.join(DEFAULT_ICON);
    if !tab_icon_path.exists()
        && let Err(e) = fs::write(&tab_icon_path, BUNDLED_TAB_ICON_BYTES)
    {
        eprintln!(
            "Warning: Failed to extract default tab icon to {:?}: {}",
            tab_icon_path, e
        );
    }

    // Extract the application window icon (neovide-tabs.png)
    let app_icon_path = data_dir.join(APP_ICON);
    if !app_icon_path.exists()
        && let Err(e) = fs::write(&app_icon_path, BUNDLED_APP_ICON_BYTES)
    {
        eprintln!(
            "Warning: Failed to extract app icon to {:?}: {}",
            app_icon_path, e
        );
    }
}

/// Get an icon bitmap handle for the given path or filename.
/// For default icon (neovide-tabs.png), loads from data directory.
/// For user icons, loads from the full path.
/// Returns a fallback icon if the specified icon cannot be loaded.
pub fn get_icon_bitmap(icon_path: &str) -> Option<HBITMAP> {
    ICON_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        // Try to get the requested icon
        if let Some(hbitmap) = cache.get_or_load(icon_path) {
            return Some(hbitmap);
        }

        // Fall back to default icon
        cache.get_fallback()
    })
}

/// Create an HICON from the bundled app icon bytes for use as window icon.
/// Returns both large (32x32) and small (16x16) icons.
pub fn create_window_icons() -> Option<(HICON, HICON)> {
    // Load the bundled app icon from memory
    let img = image::load_from_memory(BUNDLED_APP_ICON_BYTES).ok()?;

    // Create large icon (32x32)
    let large_img = img.resize_exact(
        WINDOW_ICON_SIZE as u32,
        WINDOW_ICON_SIZE as u32,
        image::imageops::FilterType::Lanczos3,
    );
    let large_rgba = large_img.to_rgba8();
    let large_icon = create_hicon_from_rgba(&large_rgba, WINDOW_ICON_SIZE)?;

    // Create small icon (16x16)
    let small_img = img.resize_exact(
        ICON_SIZE as u32,
        ICON_SIZE as u32,
        image::imageops::FilterType::Lanczos3,
    );
    let small_rgba = small_img.to_rgba8();
    let small_icon = create_hicon_from_rgba(&small_rgba, ICON_SIZE)?;

    Some((large_icon, small_icon))
}

/// Create an HICON from RGBA pixel data
fn create_hicon_from_rgba(rgba: &image::RgbaImage, size: i32) -> Option<HICON> {
    unsafe {
        let screen_dc = GetDC(HWND::default());
        if screen_dc.is_invalid() {
            return None;
        }

        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc.is_invalid() {
            ReleaseDC(HWND::default(), screen_dc);
            return None;
        }

        // Create color bitmap
        let color_bitmap = CreateCompatibleBitmap(screen_dc, size, size);
        if color_bitmap.is_invalid() {
            let _ = DeleteDC(mem_dc);
            ReleaseDC(HWND::default(), screen_dc);
            return None;
        }

        // Create mask bitmap (monochrome)
        let mask_bitmap = CreateCompatibleBitmap(screen_dc, size, size);
        if mask_bitmap.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(color_bitmap.0));
            let _ = DeleteDC(mem_dc);
            ReleaseDC(HWND::default(), screen_dc);
            return None;
        }

        // Set up bitmap info
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size, // Top-down DIB
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

        // Convert RGBA to BGRA
        let mut bgra_data: Vec<u8> = Vec::with_capacity((size * size * 4) as usize);
        for pixel in rgba.pixels() {
            bgra_data.push(pixel[2]); // B
            bgra_data.push(pixel[1]); // G
            bgra_data.push(pixel[0]); // R
            bgra_data.push(pixel[3]); // A
        }

        // Set color bitmap bits
        let result = SetDIBits(
            mem_dc,
            color_bitmap,
            0,
            size as u32,
            bgra_data.as_ptr() as *const std::ffi::c_void,
            &bmi,
            DIB_RGB_COLORS,
        );

        if result == 0 {
            let _ = DeleteObject(HGDIOBJ(color_bitmap.0));
            let _ = DeleteObject(HGDIOBJ(mask_bitmap.0));
            let _ = DeleteDC(mem_dc);
            ReleaseDC(HWND::default(), screen_dc);
            return None;
        }

        // Create icon info
        let icon_info = ICONINFO {
            fIcon: true.into(),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask_bitmap,
            hbmColor: color_bitmap,
        };

        let hicon = CreateIconIndirect(&icon_info);

        // Clean up bitmaps (CreateIconIndirect makes copies)
        let _ = DeleteObject(HGDIOBJ(color_bitmap.0));
        let _ = DeleteObject(HGDIOBJ(mask_bitmap.0));
        let _ = DeleteDC(mem_dc);
        ReleaseDC(HWND::default(), screen_dc);

        hicon.ok()
    }
}

/// Render size multiplier for high-quality SVG rasterization.
/// SVG is rendered at this multiple of the target size, then downsampled.
const SVG_RENDER_SCALE: u32 = 4;

/// Load an SVG file, rasterize it, and convert to a Win32 HBITMAP
fn load_svg_as_bitmap(path: &Path) -> Option<CachedIcon> {
    // Read the SVG file
    let svg_data = fs::read(path).ok()?;

    // Parse the SVG using resvg
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg_data, &options).ok()?;

    // Render at higher resolution for quality, then downsample
    let render_size = ICON_SIZE as u32 * SVG_RENDER_SCALE;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(render_size, render_size)?;

    // Calculate the transform to fit the SVG into the render size
    let svg_size = tree.size();
    let scale_x = render_size as f32 / svg_size.width();
    let scale_y = render_size as f32 / svg_size.height();
    let scale = scale_x.min(scale_y);

    // Center the SVG in the pixmap
    let offset_x = (render_size as f32 - svg_size.width() * scale) / 2.0;
    let offset_y = (render_size as f32 - svg_size.height() * scale) / 2.0;

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale)
        .post_translate(offset_x, offset_y);

    // Render the SVG
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert to image::RgbaImage (resvg uses premultiplied alpha, need to unpremultiply)
    let mut rgba = image::RgbaImage::new(render_size, render_size);
    for (i, pixel) in pixmap.pixels().iter().enumerate() {
        let x = (i % render_size as usize) as u32;
        let y = (i / render_size as usize) as u32;

        // Unpremultiply alpha for the image crate
        let a = pixel.alpha();
        let (r, g, b) = if a == 0 {
            (0, 0, 0)
        } else {
            let a_f = a as f32 / 255.0;
            (
                (pixel.red() as f32 / a_f).min(255.0) as u8,
                (pixel.green() as f32 / a_f).min(255.0) as u8,
                (pixel.blue() as f32 / a_f).min(255.0) as u8,
            )
        };

        rgba.put_pixel(x, y, image::Rgba([r, g, b, a]));
    }

    // Downsample to target size using high-quality filter
    let img = image::DynamicImage::ImageRgba8(rgba);
    let resized = img.resize_exact(
        ICON_SIZE as u32,
        ICON_SIZE as u32,
        image::imageops::FilterType::Lanczos3,
    );
    let rgba = resized.to_rgba8();

    create_bitmap_from_rgba(&rgba, ICON_SIZE, ICON_SIZE)
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

/// Create a Win32 HBITMAP from RGBA pixel data.
/// Uses a DIB section with premultiplied alpha for AlphaBlend compatibility.
fn create_bitmap_from_rgba(rgba: &image::RgbaImage, width: i32, height: i32) -> Option<CachedIcon> {
    unsafe {
        // Get a device context for the screen
        let screen_dc = GetDC(HWND::default());
        if screen_dc.is_invalid() {
            return None;
        }

        // Set up bitmap info for 32-bit BGRA DIB section
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

        // Create a DIB section (supports alpha channel properly)
        let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbitmap = CreateDIBSection(
            screen_dc,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0,
        );

        ReleaseDC(HWND::default(), screen_dc);

        let hbitmap = hbitmap.ok()?;
        if hbitmap.is_invalid() || bits_ptr.is_null() {
            return None;
        }

        // Copy pixel data with premultiplied alpha (required for AlphaBlend)
        let bits = std::slice::from_raw_parts_mut(
            bits_ptr as *mut u8,
            (width * height * 4) as usize,
        );

        for (i, pixel) in rgba.pixels().enumerate() {
            let offset = i * 4;
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            // Premultiply alpha for AlphaBlend
            let a_f = a as f32 / 255.0;
            bits[offset] = (b as f32 * a_f) as u8;     // B
            bits[offset + 1] = (g as f32 * a_f) as u8; // G
            bits[offset + 2] = (r as f32 * a_f) as u8; // R
            bits[offset + 3] = a;                       // A
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
    fn test_bundled_tab_icon_is_valid_png() {
        // Verify the bundled tab icon can be loaded as an image
        let img = image::load_from_memory(BUNDLED_TAB_ICON_BYTES);
        assert!(img.is_ok(), "Bundled tab icon should be a valid image");
    }

    #[test]
    fn test_bundled_app_icon_is_valid_png() {
        // Verify the bundled app icon can be loaded as an image
        let img = image::load_from_memory(BUNDLED_APP_ICON_BYTES);
        assert!(img.is_ok(), "Bundled app icon should be a valid image");
    }

    #[test]
    fn test_data_dir_path() {
        let path = data_dir_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("neovide-tabs"));
    }

    #[test]
    fn test_svg_parsing_valid() {
        // Test that valid SVG data can be parsed
        let svg_data = br#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">
            <rect width="16" height="16" fill="green"/>
        </svg>"#;

        let options = resvg::usvg::Options::default();
        let result = resvg::usvg::Tree::from_data(svg_data, &options);
        assert!(result.is_ok(), "Valid SVG should parse successfully");
    }

    #[test]
    fn test_svg_parsing_invalid() {
        // Test that invalid SVG data fails gracefully
        let invalid_svg = b"not valid svg content at all";

        let options = resvg::usvg::Options::default();
        let result = resvg::usvg::Tree::from_data(invalid_svg, &options);
        assert!(result.is_err(), "Invalid SVG should fail to parse");
    }

    #[test]
    fn test_svg_rasterization() {
        // Test that SVG can be rasterized to a pixmap
        let svg_data = br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <circle cx="50" cy="50" r="40" fill="blue"/>
        </svg>"#;

        let options = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(svg_data, &options).unwrap();

        let size = ICON_SIZE as u32;
        let pixmap = resvg::tiny_skia::Pixmap::new(size, size);
        assert!(pixmap.is_some(), "Pixmap should be created");

        let mut pixmap = pixmap.unwrap();

        // Scale SVG to fit the icon size (same logic as load_svg_as_bitmap)
        let svg_size = tree.size();
        let scale = (size as f32 / svg_size.width()).min(size as f32 / svg_size.height());
        let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);

        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // Verify the pixmap has some non-zero pixels (was rendered)
        let has_content = pixmap.pixels().iter().any(|p| p.alpha() > 0);
        assert!(has_content, "Rendered SVG should have visible content");
    }
}
