// ImageModel — CPU-side data for a raster image quad.
//
// Holds the world-space quad geometry derived from the RasterImage entity
// (insertion point, u/v vectors, pixel size) plus a *lazily* decoded RGBA
// pixel buffer. Decoding is deferred to the first GPU upload (Phase 1.6): an
// off-screen image whose quad never renders is never decoded, so opening an
// image-heavy drawing no longer pays the decode + memory cost up front.

use std::path::Path;
use std::sync::{Arc, OnceLock};

/// Decoded RGBA8 pixels plus their true (file) dimensions. Produced on first
/// access to `ImageModel::decoded`.
#[derive(Clone, Debug)]
pub struct Decoded {
    /// RGBA8 pixel data in row-major order. Arc-wrapped so cloning is O(1).
    pub pixels: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct ImageModel {
    /// Original file path — used for the deferred decode, reload, and the
    /// properties panel.
    pub file_path: String,
    /// Opacity: 1.0 = opaque, 0.0 = transparent.
    pub opacity: f32,
    /// World-space quad corners (CCW), same order as image_corners() helper:
    ///   [0] origin (bottom-left)
    ///   [1] origin + U*W (bottom-right)
    ///   [2] origin + U*W + V*H (top-right)
    ///   [3] origin + V*H (top-left)
    pub corners: [[f32; 3]; 4],
    /// Optional world-space XY rect [x0, y0, x1, y1] for paper-space
    /// viewport clipping. Mirrors `WireModel.vp_scissor` /
    /// `HatchModel.vp_scissor`.
    pub vp_scissor: Option<[f32; 4]>,
    /// Normalized draw-order depth in (0,1); higher draws on top. Fed to the
    /// image pipeline as a small clip-z bias so the raster orders correctly
    /// against other entity types.
    pub draw_depth: f32,
    /// Lazily-decoded pixels. An initialized-but-`None` value means the file
    /// was missing or undecodable. `Arc<OnceLock>` so every clone of this
    /// model (e.g. the per-frame `images_arc` snapshot) shares one decode.
    decoded: Arc<OnceLock<Option<Decoded>>>,
}

impl ImageModel {
    /// Build an ImageModel from a DXF RasterImage entity. The pixel data is
    /// **not** decoded here — that happens lazily in [`ImageModel::decoded`].
    /// Always returns `Some`; a bad file path surfaces later as a `None` from
    /// `decoded()` (the quad simply doesn't render), matching the old
    /// "excluded on decode failure" behaviour without the up-front decode.
    /// `offset` is the scene's world_offset (the precision-preserving WCS
    /// shift) for a model-space image, or `[0.0; 3]` for a paper-space one. It
    /// must match the offset applied to sibling geometry (hatches/meshes) or
    /// the raster renders displaced from everything else in the drawing.
    pub fn from_raster_image(
        img: &acadrust::entities::RasterImage,
        offset: [f64; 3],
    ) -> Option<Self> {
        let w = img.size.x;
        let h = img.size.y;
        let ox = (img.insertion_point.x + offset[0]) as f32;
        let oy = (img.insertion_point.y + offset[1]) as f32;
        let oz = (img.insertion_point.z + offset[2]) as f32;
        let ux = (img.u_vector.x * w) as f32;
        let uy = (img.u_vector.y * w) as f32;
        let uz = (img.u_vector.z * w) as f32;
        let vx = (img.v_vector.x * h) as f32;
        let vy = (img.v_vector.y * h) as f32;
        let vz = (img.v_vector.z * h) as f32;
        let corners = [
            [ox, oy, oz],
            [ox + ux, oy + uy, oz + uz],
            [ox + ux + vx, oy + uy + vy, oz + uz + vz],
            [ox + vx, oy + vy, oz + vz],
        ];
        let opacity = 1.0 - img.fade as f32 / 100.0;

        Some(Self {
            file_path: img.file_path.clone(),
            opacity,
            corners,
            vp_scissor: None,
            draw_depth: 0.0,
            decoded: Arc::new(OnceLock::new()),
        })
    }

    /// Decode the pixels on first call and cache the result; later calls — and
    /// clones sharing the same `Arc<OnceLock>` — reuse it. Returns `None` if
    /// the file is missing or cannot be decoded.
    pub fn decoded(&self) -> Option<&Decoded> {
        self.decoded
            .get_or_init(|| {
                load_pixels(&self.file_path).map(|(pixels, width, height)| Decoded {
                    pixels: Arc::new(pixels),
                    width,
                    height,
                })
            })
            .as_ref()
    }
}

/// Decode a raster image file into RGBA8 pixels.
/// Returns `None` if the file does not exist or cannot be decoded.
pub fn load_pixels(path_str: &str) -> Option<(Vec<u8>, u32, u32)> {
    let img = image::open(Path::new(path_str)).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some((rgba.into_raw(), w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_png(name: &str, w: u32, h: u32) -> String {
        let mut path = std::env::temp_dir();
        path.push(format!("ocs_imgmodel_{}_{}", std::process::id(), name));
        let buf = image::RgbaImage::from_pixel(w, h, image::Rgba([10, 20, 30, 255]));
        buf.save(&path).expect("write temp png");
        path.to_string_lossy().into_owned()
    }

    fn model_for(path: &str) -> ImageModel {
        ImageModel {
            file_path: path.to_string(),
            opacity: 1.0,
            corners: [[0.0; 3]; 4],
            vp_scissor: None,
            draw_depth: 0.0,
            decoded: Arc::new(OnceLock::new()),
        }
    }

    #[test]
    fn decode_is_deferred_until_first_access() {
        let path = write_temp_png("defer.png", 4, 2);
        let model = model_for(&path);
        // Nothing decoded yet — construction must not touch the file.
        assert!(model.decoded.get().is_none());

        let dec = model.decoded().expect("decodes on demand");
        assert_eq!((dec.width, dec.height), (4, 2));
        assert_eq!(dec.pixels.len(), 4 * 2 * 4); // w*h*RGBA
        assert!(model.decoded.get().is_some());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn clones_share_a_single_decode() {
        let path = write_temp_png("share.png", 2, 2);
        let a = model_for(&path);
        let b = a.clone(); // shares the Arc<OnceLock>
        assert!(a.decoded().is_some());
        // Deleting the file now must not matter — b reuses a's cached decode.
        let _ = std::fs::remove_file(&path);
        assert!(b.decoded.get().is_some());
        assert!(b.decoded().is_some());
    }

    #[test]
    fn world_offset_shifts_the_quad() {
        use acadrust::entities::RasterImage;
        use acadrust::Vector3;
        // 4x2-pixel image inserted at WCS (1000, 2000, 0) with unit u/v.
        let img = RasterImage::new("x.png", Vector3::new(1000.0, 2000.0, 0.0), 4.0, 2.0);
        let offset = [-1000.0f64, -2000.0, 0.0];
        let m = ImageModel::from_raster_image(&img, offset).unwrap();
        // Origin corner shifts by the offset; without it the raster would sit
        // 1000 units away from offset-shifted sibling geometry.
        assert_eq!(m.corners[0], [0.0, 0.0, 0.0]);
        // Opposite corner = origin + U*w + V*h, also offset-shifted.
        assert_eq!(m.corners[2], [4.0, 2.0, 0.0]);
        // Zero offset leaves the quad at its absolute WCS position.
        let m0 = ImageModel::from_raster_image(&img, [0.0; 3]).unwrap();
        assert_eq!(m0.corners[0], [1000.0, 2000.0, 0.0]);
    }

    #[test]
    fn missing_file_decodes_to_none() {
        let model = model_for("/nonexistent/ocs/does-not-exist.png");
        assert!(model.decoded().is_none());
        // Result is cached: the cell is now initialized (to None).
        assert!(model.decoded.get().is_some());
    }
}
