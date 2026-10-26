//! RapidRAW `.rrdata` sidecar import (Phase 1).
//!
//! Reference: RapidRAW stores a flat JSON object of UI-scale adjustments
//! (`exposure` in stops -5..5, most others -100..100, curves in 0..255,
//! HSL/grading/calibration nested objects). Source: RapidRAW
//! `src/utils/adjustments.ts` (`INITIAL_ADJUSTMENTS`) and
//! `src-tauri/src/image_processing.rs` (`get_global_adjustments_from_json`).
//! No RapidRAW code is copied; only the observed key names and slider ranges
//! are mapped onto Raygun/LightCraft develop controls.
//!
//! This is a v1 approximation: core tone/color/detail/curve/HSL/grading map
//! directly where slider ranges match. Relative temperature, calibration
//! scale differences, masks, AI patches, perspective, and LUT files are
//! reported as unmapped so a later phase can promote them without silently
//! misrendering.

use lightcraft_develop::{Crop, DevelopSettings, controls};
use lightcraft_geom::{CropGeometry, Orientation, Point, Rect};
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct RrImport {
    pub applied: Vec<String>,
    pub unmapped: Vec<String>,
}

/// Maximum pasted sidecar size; protects both desktop and browser imports.
pub const MAX_RRDATA_BYTES: usize = 1024 * 1024;

/// Accept a flat adjustment object or a sidecar containing `adjustments`.
pub fn parse_rrdata(text: &str) -> Result<Value, String> {
    if text.len() > MAX_RRDATA_BYTES {
        return Err("RapidRAW adjustments exceed the 1 MiB limit".into());
    }
    let value: Value = serde_json::from_str(text).map_err(|e| format!("Invalid RapidRAW JSON: {e}"))?;
    let adjustments = value.get("adjustments").unwrap_or(&value);
    if !adjustments.is_object() {
        return Err("RapidRAW adjustments must be a JSON object".into());
    }
    Ok(adjustments.clone())
}

fn num(v: &Value, key: &str) -> Option<f64> {
    v.get(key)?.as_f64()
}

fn set_mapped(s: &mut DevelopSettings, out: &mut RrImport, control: &str, v: f64) {
    if controls::find(control).is_none() {
        out.unmapped.push(control.to_string());
        return;
    }
    controls::set(s, control, v);
    out.applied.push(control.to_string());
}

fn curve_points(arr: Option<&Vec<Value>>) -> Option<Vec<Point>> {
    let arr = arr?;
    if arr.len() > 64 {
        return None;
    }
    let mut pts = Vec::new();
    for p in arr {
        let x = p.get("x")?.as_f64()?;
        let y = p.get("y")?.as_f64()?;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        pts.push(Point::new(x / 255.0, y / 255.0));
    }
    if pts.len() < 2 {
        return None;
    }
    pts.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    Some(pts)
}

/// Apply adjustments without image metadata. Pixel crops and nonzero straighten
/// require [`apply_rrdata_for_image`]; they are reported as unmapped here.
pub fn apply_rrdata(settings: &mut DevelopSettings, json: &Value) -> RrImport {
    apply_rrdata_for_image(settings, json, None)
}

/// Apply adjustments using the EXIF-oriented source dimensions, before user turns.
pub fn apply_rrdata_for_image(settings: &mut DevelopSettings, json: &Value, dimensions: Option<(u32, u32)>) -> RrImport {
    let mut out = RrImport::default();
    // (rr_key, control_id)
    const DIRECT: &[(&str, &str)] = &[
        ("exposure", "light.exposure"),
        ("contrast", "light.contrast"),
        ("highlights", "light.highlights"),
        ("shadows", "light.shadows"),
        ("whites", "light.whites"),
        ("blacks", "light.blacks"),
        ("vibrance", "color.vibrance"),
        ("saturation", "color.saturation"),
        ("clarity", "effects.clarity"),
        ("dehaze", "effects.dehaze"),
        ("tint", "wb.tint"),
        ("lumaNoiseReduction", "detail.nrLuminance"),
        ("colorNoiseReduction", "detail.nrColor"),
        ("grainAmount", "grain.amount"),
        ("grainSize", "grain.size"),
        ("grainRoughness", "grain.roughness"),
        ("vignetteAmount", "vignette.amount"),
    ];
    for (rr, ctl) in DIRECT {
        if let Some(v) = num(json, rr)
            && v.is_finite()
        {
            // wb.tint in RapidRAW relative mode shares the -100..100 range.
            set_mapped(settings, &mut out, ctl, v);
        }
    }
    // structure -> texture (closest existing control; flagged by name).
    if let Some(v) = num(json, "structure")
        && v.is_finite()
    {
        set_mapped(settings, &mut out, "effects.texture", v);
    }
    // sharpness: RapidRAW UI 0..80 -> Raygun 0..150.
    if let Some(v) = num(json, "sharpness")
        && v.is_finite()
        && v >= 0.0
    {
        set_mapped(settings, &mut out, "detail.sharpenAmount", v * 1.5);
    }
    // temperature: only Kelvin values map directly; relative -100..100 does not.
    if let Some(v) = num(json, "temperature")
        && v.is_finite()
    {
        if (2000.0..=50000.0).contains(&v) {
            set_mapped(settings, &mut out, "wb.temp", v);
        } else {
            out.unmapped.push("temperature(relative)".to_string());
        }
    }
    // curves: {luma,red,green,blue} arrays of {x,y} in 0..255.
    if let Some(curves) = json.get("curves").and_then(Value::as_object) {
        let map = [("luma", "master"), ("red", "red"), ("green", "green"), ("blue", "blue")];
        for (rr, lc) in map {
            if let Some(arr) = curves.get(rr).and_then(Value::as_array)
                && let Some(pts) = curve_points(Some(arr))
            {
                match lc {
                    "master" => settings.curve.master = pts,
                    "red" => settings.curve.red = pts,
                    "green" => settings.curve.green = pts,
                    "blue" => settings.curve.blue = pts,
                    _ => {}
                }
                out.applied.push(format!("curve.{lc}"));
            }
        }
    }
    // HSL: {reds,oranges,yellows,greens,aquas,blues,purples,magentas} x {hue,saturation,luminance}.
    if let Some(hsl) = json.get("hsl").and_then(Value::as_object) {
        const BANDS: &[(&str, &str)] = &[
            ("reds", "red"),
            ("oranges", "orange"),
            ("yellows", "yellow"),
            ("greens", "green"),
            ("aquas", "aqua"),
            ("blues", "blue"),
            ("purples", "purple"),
            ("magentas", "magenta"),
        ];
        for (rr_band, lc_band) in BANDS {
            if let Some(band) = hsl.get(*rr_band).and_then(Value::as_object) {
                for (rr_k, suffix) in [("hue", "hue"), ("saturation", "sat"), ("luminance", "lum")] {
                    if let Some(v) = band.get(rr_k).and_then(Value::as_f64)
                        && v.is_finite()
                    {
                        set_mapped(settings, &mut out, &format!("mixer.{lc_band}.{suffix}"), v);
                    }
                }
            }
        }
    }
    // Color grading: {shadows,midtones,highlights,global} x {hue,sat/lum} + blending/balance.
    if let Some(cg) = json.get("colorGrading").and_then(Value::as_object) {
        const SECTIONS: &[(&str, &str)] = &[("shadows", "shadows"), ("midtones", "midtones"), ("highlights", "highlights"), ("global", "global")];
        for (rr_s, lc_s) in SECTIONS {
            if let Some(sec) = cg.get(*rr_s).and_then(Value::as_object) {
                for (rr_k, lc_k) in [("hue", "hue"), ("saturation", "sat"), ("luminance", "lum")] {
                    if let Some(v) = sec.get(rr_k).and_then(Value::as_f64)
                        && v.is_finite()
                    {
                        set_mapped(settings, &mut out, &format!("grading.{lc_s}.{lc_k}"), v);
                    }
                }
            }
        }
        if let Some(v) = cg.get("blending").and_then(Value::as_f64)
            && v.is_finite()
        {
            set_mapped(settings, &mut out, "grading.blending", v);
        }
        if let Some(v) = cg.get("balance").and_then(Value::as_f64)
            && v.is_finite()
        {
            set_mapped(settings, &mut out, "grading.balance", v);
        }
    }
    // toneMapper: basic/agx/aces (also "filmic" as base alias).
    if let Some(t) = json.get("toneMapper").and_then(Value::as_str) {
        let mode = match t.to_ascii_lowercase().as_str() {
            "agx" => Some(3.0),
            "aces" | "simplified-aces" | "simplified_aces" => Some(4.0),
            "basic" | "base" | "filmic" | "reinhard" => Some(0.0),
            _ => None,
        };
        if let Some(m) = mode {
            set_mapped(settings, &mut out, "light.toneMapper", m);
        } else {
            out.unmapped.push("toneMapper".to_string());
        }
    }
    const GEOMETRY: &[&str] = &["crop", "rotation", "orientationSteps", "flipHorizontal", "flipVertical"];
    if GEOMETRY.iter().any(|key| json.get(key).is_some()) {
        let keys = GEOMETRY.iter().filter(|key| json.get(key).is_some()).map(|key| key.to_string());
        match import_geometry(settings, json, dimensions) {
            Some((orientation, crop)) => {
                settings.orientation = orientation;
                settings.crop = crop;
                out.applied.extend(keys);
            }
            None => out.unmapped.extend(keys),
        }
    }
    // Unsupported fields remain explicit rather than silently approximated.
    for key in ["colorCalibration", "brightness", "masks", "aiPatches", "aspectRatio", "lensBlurEnabled", "lutPath", "lutIntensity"] {
        if json.get(key).is_some() {
            out.unmapped.push(key.to_string());
        }
    }
    out
}

fn mirror_rect(mut rect: Rect, horizontal: bool, vertical: bool) -> Rect {
    if horizontal {
        (rect.x0, rect.x1) = (1.0 - rect.x1, 1.0 - rect.x0);
    }
    if vertical {
        (rect.y0, rect.y1) = (1.0 - rect.y1, 1.0 - rect.y0);
    }
    rect
}

fn import_geometry(s: &DevelopSettings, json: &Value, dimensions: Option<(u32, u32)>) -> Option<(Orientation, Crop)> {
    let orientation = match json.get("orientationSteps") {
        Some(value) => {
            let turns = value.as_u64()?;
            if turns > 3 {
                return None;
            }
            Orientation::from_parts(false, turns as u8)
        }
        None => s.orientation,
    };
    let horizontal = match json.get("flipHorizontal") {
        Some(value) => value.as_bool()?,
        None => s.crop.flip_h,
    };
    let vertical = match json.get("flipVertical") {
        Some(value) => value.as_bool()?,
        None => s.crop.flip_v,
    };
    let previous_rotation = if s.crop.flip_h != s.crop.flip_v { -s.crop.geometry.angle } else { s.crop.geometry.angle };
    let rotation = match json.get("rotation") {
        Some(value) => value.as_f64()?,
        None => previous_rotation,
    };
    if !rotation.is_finite() || !(-45.0..=45.0).contains(&rotation) {
        return None;
    }
    let dimensions = dimensions
        .filter(|(w, h)| *w > 0 && *h > 0)
        .map(|(w, h)| if orientation.swaps_axes() { (f64::from(h), f64::from(w)) } else { (f64::from(w), f64::from(h)) });
    let rect = match json.get("crop") {
        Some(value) if value.is_null() => Rect::UNIT,
        Some(value) => {
            let (w, h, round) = match value.get("unit").and_then(Value::as_str) {
                Some("%") => (100.0, 100.0, false),
                None | Some("px") => {
                    let (w, h) = dimensions?;
                    (w, h, true)
                }
                _ => return None,
            };
            if value.get("unit").is_some_and(|unit| !unit.is_string()) {
                return None;
            }
            let read = |key: &str| {
                let n = num(value, key)?;
                n.is_finite().then_some(if round { n.round() } else { n })
            };
            let (x, y, width, height) = (read("x")?, read("y")?, read("width")?, read("height")?);
            if x < 0.0 || y < 0.0 || x >= w || y >= h || width <= 0.0 || height <= 0.0 {
                return None;
            }
            Rect::new(x / w, y / h, (x + width.min(w - x)) / w, (y + height.min(h - y)) / h)
        }
        None => mirror_rect(s.crop.geometry.rect, s.crop.flip_h, s.crop.flip_v),
    };
    if ![rect.x0, rect.y0, rect.x1, rect.y1].iter().all(|v| v.is_finite()) || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return None;
    }
    // RapidRAW flips before straighten/crop; Raygun flips the cropped output.
    // Conjugating the rotation and reflecting its crop preserves the same frame.
    let geometry = CropGeometry { rect: mirror_rect(rect, horizontal, vertical), angle: if horizontal != vertical { -rotation } else { rotation } };
    if rotation != 0.0 {
        let (w, h) = dimensions?;
        // RapidRAW fills outside-source corners transparently; our sampler clamps.
        // Refuse that case instead of importing different border pixels.
        if !geometry.is_within_image(w, h) {
            return None;
        }
    }
    let mut crop = Crop { geometry, flip_h: horizontal, flip_v: vertical, ..s.crop };
    if json.get("crop").is_some() {
        crop.aspect = None;
    }
    Some((orientation, crop))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_flat_and_wrapped_sidecars_with_limits() {
        let flat = json!({"exposure": 1});
        assert_eq!(parse_rrdata(&flat.to_string()).unwrap(), flat);
        assert_eq!(parse_rrdata(&json!({"adjustments": flat}).to_string()).unwrap(), flat);
        for text in ["[]", "null", "{", r#"{"adjustments":null}"#] {
            assert!(parse_rrdata(text).is_err());
        }
        assert!(parse_rrdata(&" ".repeat(MAX_RRDATA_BYTES + 1)).is_err());
        assert!(parse_rrdata(&format!("{}0{}", "[".repeat(200), "]".repeat(200))).is_err());
    }

    #[test]
    fn geometry_matches_pre_crop_flip_order_and_is_idempotent() {
        use lightcraft_pipeline::geometry::Frame;
        for turns in 0..4 {
            let (w, h) = if turns % 2 == 0 { (600.0, 400.0) } else { (400.0, 600.0) };
            for horizontal in [false, true] {
                for vertical in [false, true] {
                    for rotation in [-12.0_f64, 0.0, 17.0] {
                        let mut s = DevelopSettings::default();
                        let input = json!({"orientationSteps": turns, "flipHorizontal": horizontal, "flipVertical": vertical,
                            "rotation": rotation, "crop": {"x": 150, "y": 120, "width": 100, "height": 130}});
                        let r = apply_rrdata_for_image(&mut s, &input, Some((600, 400)));
                        assert!(r.unmapped.is_empty(), "{input}: {r:?}");
                        let frame = Frame::new(600, 400, &s, true);
                        let mapping = frame.out_to_oriented(100, 130);
                        for (x, y) in [(0.0, 0.0), (33.5, 82.0), (100.0, 130.0)] {
                            // Independent reference: inverse fine rotation, then undo pre-crop flips.
                            let (sin, cos) = (-rotation.to_radians()).sin_cos();
                            let (dx, dy) = (150.0 + x - w / 2.0, 120.0 + y - h / 2.0);
                            let mut expected = Point::new(w / 2.0 + cos * dx - sin * dy, h / 2.0 + sin * dx + cos * dy);
                            if horizontal {
                                expected.x = w - expected.x;
                            }
                            if vertical {
                                expected.y = h - expected.y;
                            }
                            let actual = mapping.apply(Point::new(x, y));
                            assert!(
                                (actual.x - expected.x).abs() < 1e-9 && (actual.y - expected.y).abs() < 1e-9,
                                "{input}: {actual:?} != {expected:?}"
                            );
                        }
                        let before = s.clone();
                        apply_rrdata_for_image(&mut s, &input, Some((600, 400)));
                        assert_eq!(s, before);
                    }
                }
            }
        }
    }

    #[test]
    fn crop_units_rounding_clipping_and_null_reset() {
        let mut s = DevelopSettings::default();
        apply_rrdata_for_image(&mut s, &json!({"crop": {"x": 10.6, "y": 20.1, "width": 500, "height": 40.9}}), Some((100, 100)));
        assert_eq!(s.crop.geometry.rect, Rect::new(0.11, 0.20, 1.0, 0.61));
        let r = apply_rrdata(&mut s, &json!({"crop": {"unit": "%", "x": 10, "y": 20, "width": 40, "height": 50}}));
        assert!(r.unmapped.is_empty());
        assert_eq!(s.crop.geometry.rect, Rect::new(0.1, 0.2, 0.5, 0.7));
        apply_rrdata(&mut s, &json!({"crop": null}));
        assert_eq!(s.crop.geometry.rect, Rect::UNIT);
    }

    #[test]
    fn invalid_geometry_is_atomic_while_tone_still_imports() {
        for invalid in [
            json!({"crop": {"unit": "%", "x": 50, "y": 20, "width": 1e-300, "height": 20}}),
            json!({"crop": {"x": -1, "y": 0, "width": 10, "height": 10}}),
            json!({"crop": {"unit": "other", "x": 1, "y": 0, "width": 10, "height": 10}}),
            json!({"orientationSteps": 4}),
            json!({"flipVertical": 1}),
            json!({"rotation": 46}),
            json!({"rotation": 12, "crop": null}),
        ] {
            let mut s = DevelopSettings::default();
            let before = s.crop;
            let mut input = invalid.clone();
            input["exposure"] = json!(1.5);
            input["flipHorizontal"] = json!(true);
            let r = apply_rrdata_for_image(&mut s, &input, Some((600, 400)));
            assert_eq!(s.crop, before, "{input}");
            assert_eq!(s.orientation, DevelopSettings::default().orientation);
            assert_eq!(s.light.exposure, 1.5);
            assert!(r.unmapped.contains(&"flipHorizontal".into()));
        }
    }

    #[test]
    fn maps_core_tone_and_reports_unmapped() {
        let mut s = DevelopSettings::default();
        let j = json!({
            "exposure": 1.5, "contrast": 10.0, "highlights": -45.0,
            "shadows": 38.0, "whites": 5.0, "blacks": -5.0,
            "vibrance": 22.0, "clarity": 28.0, "dehaze": 18.0,
            "toneMapper": "agx",
            "colorCalibration": {"redHue": 10.0},
            "lutPath": "film.cube", "crop": {}
        });
        let r = apply_rrdata(&mut s, &j);
        assert!((s.light.exposure - 1.5).abs() < 1e-9);
        assert!((s.light.highlights + 45.0).abs() < 1e-9);
        assert!((s.light.shadows - 38.0).abs() < 1e-9);
        assert!((s.light.tone_mapper - 3.0).abs() < 1e-9);
        assert!(r.applied.contains(&"light.exposure".to_string()));
        assert!(r.applied.contains(&"light.toneMapper".to_string()));
        for k in ["colorCalibration", "lutPath", "crop"] {
            assert!(r.unmapped.contains(&k.to_string()), "missing {k}");
        }
    }

    #[test]
    fn maps_curves_hsl_grading() {
        let mut s = DevelopSettings::default();
        let j = json!({
            "curves": {"luma": [{"x": 0.0, "y": 0.0}, {"x": 255.0, "y": 255.0}]},
            "hsl": {"reds": {"hue": 10.0, "saturation": -5.0, "luminance": 3.0}},
            "colorGrading": {"shadows": {"hue": 28.0, "saturation": 30.0, "luminance": 0.0}, "blending": 50.0, "balance": 0.0}
        });
        let r = apply_rrdata(&mut s, &j);
        assert_eq!(s.curve.master.len(), 2);
        assert!(r.applied.contains(&"mixer.red.hue".to_string()));
        assert!(r.applied.contains(&"grading.shadows.hue".to_string()));
    }

    #[test]
    fn rejects_bad_numbers_without_panic() {
        let mut s = DevelopSettings::default();
        let j = json!({"exposure": "hot", "curves": {"luma": [{"x": 0.0}]}});
        let r = apply_rrdata(&mut s, &j);
        assert_eq!(s.light.exposure, 0.0);
        assert!(r.applied.is_empty());
    }
}
