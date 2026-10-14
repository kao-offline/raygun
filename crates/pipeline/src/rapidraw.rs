//! RapidRAW's full AgX and simplified-ACES transforms, adapted to our linear Rec.2020 pipeline.
//! Source: CyberTimon/RapidRAW at 74597d066685283588502edbecd747bfe80404cf,
//! src-tauri/src/image_processing.rs and shaders/shader.wgsl (AGPL-3.0).
//! Copyright Timon Käch and RapidRAW contributors; modified by Raygun contributors, 2026.
use lightcraft_color::{Mat3, REC2020, RgbSpace, SRGB, Xy};
use std::sync::OnceLock;

pub const CURVE_N: usize = 4096;

pub struct Transform {
    pub mode: u8,
    pub to_srgb: [[f32; 3]; 3],
    pub from_srgb: [[f32; 3]; 3],
    pub agx_in: [[f32; 3]; 3],
    pub agx_out: [[f32; 3]; 3],
    curve: &'static [f32; CURVE_N],
}

fn primary(p: Xy, scale: f64, rotation: f64) -> Xy {
    let (x, y) = ((p.x - REC2020.white.x) * scale, (p.y - REC2020.white.y) * scale);
    let (sin, cos) = rotation.sin_cos();
    Xy::new(REC2020.white.x + x * cos - y * sin, REC2020.white.y + x * sin + y * cos)
}

pub fn curve() -> &'static [f32; CURVE_N] {
    static LUT: OnceLock<[f32; CURVE_N]> = OnceLock::new();
    LUT.get_or_init(|| {
        std::array::from_fn(|i| {
            let x = i as f32 / (CURVE_N - 1) as f32;
            let scale = if x < 0.6060606 { -1.0359 } else { 1.3475 };
            let v = (2.3843 * (x - 0.6060606) / scale).max(0.0);
            (scale * v / (1.0 + v.powf(1.5)).powf(1.0 / 1.5) + 0.43446).clamp(0.0, 1.0).powf(2.4)
        })
    })
}

impl Transform {
    pub fn for_mode(mode: u8) -> Option<&'static Self> {
        fn build(mode: u8) -> Transform {
            let inset = RgbSpace {
                name: "RapidRAW AgX inset",
                white: REC2020.white,
                r: primary(REC2020.r, 1.0 - 0.2946245, 0.03540329),
                g: primary(REC2020.g, 1.0 - 0.25861925, -0.02108586),
                b: primary(REC2020.b, 1.0 - 0.14641371, -0.06305724),
            };
            let outset = RgbSpace {
                name: "RapidRAW AgX outset",
                white: REC2020.white,
                r: primary(REC2020.r, 1.0 - 0.2907764, 0.0),
                g: primary(REC2020.g, 1.0 - 0.2631554, 0.0),
                b: primary(REC2020.b, 1.0 - 0.04581072, 0.0),
            };
            let pipe_to_base = SRGB.to_space(&REC2020);
            let input = REC2020.from_xyz().mul(&inset.to_xyz()).mul(&pipe_to_base);
            let output =
                pipe_to_base.inverse().unwrap_or(Mat3::IDENTITY).mul(&REC2020.from_xyz().mul(&outset.to_xyz()).inverse().unwrap_or(Mat3::IDENTITY));
            Transform {
                mode,
                to_srgb: REC2020.to_space(&SRGB).to_f32(),
                from_srgb: pipe_to_base.to_f32(),
                agx_in: input.to_f32(),
                agx_out: output.to_f32(),
                curve: curve(),
            }
        }
        static AGX: OnceLock<Transform> = OnceLock::new();
        static ACES: OnceLock<Transform> = OnceLock::new();
        match mode {
            3 => Some(AGX.get_or_init(|| build(3))),
            4 => Some(ACES.get_or_init(|| build(4))),
            _ => None,
        }
    }

    pub fn apply(&self, c: [f32; 3]) -> [f32; 3] {
        let mut rgb = multiply(self.to_srgb, c.map(|v| if v.is_finite() { v.clamp(-65504.0, 65504.0) } else { 0.0 }));
        if self.mode == 3 {
            let min = rgb.into_iter().fold(0.0f32, f32::min);
            rgb = multiply(self.agx_in, rgb.map(|v| v - min));
            rgb = rgb.map(|v| {
                let x = (((v / 0.18).max(1e-6).log2() + 15.2) / 20.2).clamp(0.0, 1.0);
                let i = (x * (CURVE_N - 1) as f32) as usize;
                self.curve.get(i).copied().unwrap_or(1.0)
            });
            rgb = multiply(self.agx_out, rgb);
        } else {
            rgb = rgb.map(|v| {
                let x = v.max(0.0);
                (x * (2.51 * x + 0.03) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0)
            });
        }
        multiply(self.from_srgb, rgb.map(|v| v.clamp(0.0, 1.0)))
    }
}

#[inline]
pub fn multiply(m: [[f32; 3]; 3], c: [f32; 3]) -> [f32; 3] {
    m.map(|r| r[0] * c[0] + r[1] * c[1] + r[2] * c[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn agx_matches_independent_upstream_matrix_and_pixel_reference() {
        // NumPy float32 translation of the pinned upstream primaries/matrix and LUT
        // routines, evaluated independently of Raygun's colour-space implementation.
        let transform = Transform::for_mode(3).unwrap();
        for (input, expected) in [
            ([0.18; 3], [0.5105575; 3]),
            ([4.0, 0.2, 0.01], [1.0, 0.7759754, 0.5664165]),
            ([0.001, 0.4, 1.5], [0.4044285, 0.760057, 1.0]),
            ([-0.1, 0.2, 0.3], [0.2942104, 0.6449533, 0.7134122]),
            ([0.02, 2.0, 0.01], [0.747897, 1.0, 0.5464779]),
        ] {
            let working = SRGB.to_space(&REC2020).apply_f32(input);
            let actual = REC2020.to_space(&SRGB).apply_f32(transform.apply(working));
            for (a, e) in actual.into_iter().zip(expected) {
                assert!((a - e).abs() < 1e-4, "{input:?}: {actual:?} != {expected:?}");
            }
        }
    }

    #[test]
    fn aces_matches_rapidraw_reference_on_neutral_and_coloured_samples() {
        let transform = Transform::for_mode(4).unwrap();
        for rgb in [[0.18; 3], [4.0, 0.2, 0.01], [0.001, 0.4, 1.5], [-0.1, 0.2, 0.3]] {
            let working = SRGB.to_space(&REC2020).apply_f32(rgb);
            let output = REC2020.to_space(&SRGB).apply_f32(transform.apply(working));
            for (actual, x) in output.into_iter().zip(rgb) {
                let x = x.max(0.0);
                let expected = ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0);
                assert!((actual - expected).abs() < 1e-5, "{rgb:?}: {actual} != {expected}");
            }
        }
    }
    #[test]
    fn agx_matches_reference_sigmoid_and_stays_finite() {
        for (i, actual) in curve().iter().enumerate() {
            let x = i as f32 / 4095.0;
            let scale = if x < 0.6060606 { -1.0359 } else { 1.3475 };
            let t = 2.3843 * (x - 0.6060606) / scale;
            let expected = (scale * t / (1.0 + t.powf(1.5)).powf(1.0 / 1.5) + 0.43446).clamp(0.0, 1.0).powf(2.4);
            assert!((actual - expected).abs() < 1e-6);
        }
        for mode in [3, 4] {
            let transform = Transform::for_mode(mode).unwrap();
            for c in [[0.0; 3], [1e4, 0.2, 0.01], [-1.0, 0.1, 2.0], [f32::NAN, f32::INFINITY, f32::NEG_INFINITY]] {
                assert!(transform.apply(c).iter().all(|v| v.is_finite() && *v >= -1e-6 && *v <= 1.00001));
            }
        }
        assert!(Transform::for_mode(1).is_none());
    }
}
