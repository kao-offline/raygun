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
//! scale differences, masks, AI patches, geometry/crop, and LUT files are
//! reported as unmapped so a later phase can promote them without silently
//! misrendering.

use lightcraft_develop::{DevelopSettings, controls};
use lightcraft_geom::Point;
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct RrImport {
    pub applied: Vec<String>,
    pub unmapped: Vec<String>,
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

/// Apply a RapidRAW adjustments object onto `settings` in place.
pub fn apply_rrdata(settings: &mut DevelopSettings, json: &Value) -> RrImport {
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
        if let Some(v) = num(json, rr) {
            if v.is_finite() {
                // wb.tint in RapidRAW relative mode shares the -100..100 range.
                set_mapped(settings, &mut out, ctl, v);
            }
        }
    }
    // structure -> texture (closest existing control; flagged by name).
    if let Some(v) = num(json, "structure") {
        if v.is_finite() {
            set_mapped(settings, &mut out, "effects.texture", v);
        }
    }
    // sharpness: RapidRAW UI 0..80 -> Raygun 0..150.
    if let Some(v) = num(json, "sharpness") {
        if v.is_finite() && v >= 0.0 {
            set_mapped(settings, &mut out, "detail.sharpenAmount", v * 1.5);
        }
    }
    // temperature: only Kelvin values map directly; relative -100..100 does not.
    if let Some(v) = num(json, "temperature") {
        if v.is_finite() {
            if (2000.0..=50000.0).contains(&v) {
                set_mapped(settings, &mut out, "wb.temp", v);
            } else {
                out.unmapped.push("temperature(relative)".to_string());
            }
        }
    }
    // curves: {luma,red,green,blue} arrays of {x,y} in 0..255.
    if let Some(curves) = json.get("curves").and_then(Value::as_object) {
        let map = [
            ("luma", "master"),
            ("red", "red"),
            ("green", "green"),
            ("blue", "blue"),
        ];
        for (rr, lc) in map {
            if let Some(arr) = curves.get(rr).and_then(Value::as_array) {
                if let Some(pts) = curve_points(Some(arr)) {
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
                    if let Some(v) = band.get(rr_k).and_then(Value::as_f64) {
                        if v.is_finite() {
                            set_mapped(settings, &mut out, &format!("mixer.{lc_band}.{suffix}"), v);
                        }
                    }
                }
            }
        }
    }
    // Color grading: {shadows,midtones,highlights,global} x {hue,sat/lum} + blending/balance.
    if let Some(cg) = json.get("colorGrading").and_then(Value::as_object) {
        const SECTIONS: &[(&str, &str)] = &[
            ("shadows", "shadows"),
            ("midtones", "midtones"),
            ("highlights", "highlights"),
            ("global", "global"),
        ];
        for (rr_s, lc_s) in SECTIONS {
            if let Some(sec) = cg.get(*rr_s).and_then(Value::as_object) {
                for (rr_k, lc_k) in [("hue", "hue"), ("saturation", "sat"), ("luminance", "lum")] {
                    if let Some(v) = sec.get(rr_k).and_then(Value::as_f64) {
                        if v.is_finite() {
                            set_mapped(settings, &mut out, &format!("grading.{lc_s}.{lc_k}"), v);
                        }
                    }
                }
            }
        }
        if let Some(v) = cg.get("blending").and_then(Value::as_f64) {
            if v.is_finite() {
                set_mapped(settings, &mut out, "grading.blending", v);
            }
        }
        if let Some(v) = cg.get("balance").and_then(Value::as_f64) {
            if v.is_finite() {
                set_mapped(settings, &mut out, "grading.balance", v);
            }
        }
    }
    // Known-but-unmapped in v1 (no silent misrender): calibration scale
    // mismatch, masks/AI, geometry/crop, LUT files, brightness.
    for key in [
        "colorCalibration",
        "brightness",
        "masks",
        "aiPatches",
        "crop",
        "lensBlurEnabled",
        "lutPath",
        "lutIntensity",
        "toneMapper",
    ] {
        if json.get(key).is_some() {
            out.unmapped.push(key.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_core_tone_and_reports_unmapped() {
        let mut s = DevelopSettings::default();
        let j = json!({
            "exposure": 1.5, "contrast": 10.0, "highlights": -45.0,
            "shadows": 38.0, "whites": 5.0, "blacks": -5.0,
            "vibrance": 22.0, "clarity": 28.0, "dehaze": 18.0,
            "colorCalibration": {"redHue": 10.0},
            "lutPath": "film.cube", "toneMapper": "agx", "crop": {}
        });
        let r = apply_rrdata(&mut s, &j);
        assert!((s.light.exposure - 1.5).abs() < 1e-9);
        assert!((s.light.highlights + 45.0).abs() < 1e-9);
        assert!((s.light.shadows - 38.0).abs() < 1e-9);
        assert!(r.applied.contains(&"light.exposure".to_string()));
        for k in ["colorCalibration", "lutPath", "toneMapper", "crop"] {
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
