//! Conversions between sRGB, CIE XYZ and CIE L*a*b*, all D65-referenced.
//!
//! The formulas here are the standard ones (IEC 61966-2-1 for sRGB,
//! CIE 15:2004 for Lab), reimplemented from the reference equations
//! rather than pulled from a crate, since the whole point of this
//! project is to own the math end to end.

pub const D65_WHITE: [f64; 3] = [0.95047, 1.0, 1.08883];

/// sRGB colour, components in 0.0..=1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xyz {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl Rgb {
    pub fn from_hex(s: &str) -> Result<Rgb, String> {
        let s = s.trim().trim_start_matches('#');
        if s.len() != 6 {
            return Err(format!("expected 6 hex digits, got {:?}", s));
        }
        let byte = |i: usize| -> Result<f64, String> {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map(|v| v as f64 / 255.0)
                .map_err(|e| format!("bad hex digit at {}: {}", i, e))
        };
        Ok(Rgb {
            r: byte(0)?,
            g: byte(2)?,
            b: byte(4)?,
        })
    }

    pub fn to_hex(self) -> String {
        let clamp = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", clamp(self.r), clamp(self.g), clamp(self.b))
    }

    /// Removes the sRGB transfer function, giving linear-light RGB.
    fn to_linear(self) -> [f64; 3] {
        [self.r, self.g, self.b].map(|c| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
    }

    fn from_linear(lin: [f64; 3]) -> Rgb {
        let enc = lin.map(|c| {
            let c = c.clamp(0.0, 1.0);
            if c <= 0.0031308 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        });
        Rgb { r: enc[0], g: enc[1], b: enc[2] }
    }

    pub fn to_xyz(self) -> Xyz {
        let [r, g, b] = self.to_linear();
        // sRGB -> XYZ matrix, D65 white point.
        Xyz {
            x: r * 0.4124564 + g * 0.3575761 + b * 0.1804375,
            y: r * 0.2126729 + g * 0.7151522 + b * 0.0721750,
            z: r * 0.0193339 + g * 0.1191920 + b * 0.9503041,
        }
    }
}

impl Xyz {
    pub fn to_rgb(self) -> Rgb {
        let r = self.x * 3.2404542 + self.y * -1.5371385 + self.z * -0.4985314;
        let g = self.x * -0.9692660 + self.y * 1.8760108 + self.z * 0.0415560;
        let b = self.x * 0.0556434 + self.y * -0.2040259 + self.z * 1.0572252;
        Rgb::from_linear([r, g, b])
    }

    pub fn to_lab(self) -> Lab {
        let f = |t: f64| -> f64 {
            const DELTA: f64 = 6.0 / 29.0;
            if t > DELTA.powi(3) {
                t.cbrt()
            } else {
                t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
            }
        };
        let fx = f(self.x / D65_WHITE[0]);
        let fy = f(self.y / D65_WHITE[1]);
        let fz = f(self.z / D65_WHITE[2]);
        Lab {
            l: 116.0 * fy - 16.0,
            a: 500.0 * (fx - fy),
            b: 200.0 * (fy - fz),
        }
    }
}

impl Lab {
    pub fn to_xyz(self) -> Xyz {
        let fy = (self.l + 16.0) / 116.0;
        let fx = fy + self.a / 500.0;
        let fz = fy - self.b / 200.0;
        let finv = |t: f64| -> f64 {
            const DELTA: f64 = 6.0 / 29.0;
            if t > DELTA {
                t.powi(3)
            } else {
                3.0 * DELTA * DELTA * (t - 4.0 / 29.0)
            }
        };
        Xyz {
            x: finv(fx) * D65_WHITE[0],
            y: finv(fy) * D65_WHITE[1],
            z: finv(fz) * D65_WHITE[2],
        }
    }

    /// Perceptual distance (CIE76 dE). Cheap and fine for a first pass;
    /// dE2000 is the more accurate metric but a lot more code.
    pub fn delta_e76(self, other: Lab) -> f64 {
        ((self.l - other.l).powi(2) + (self.a - other.a).powi(2) + (self.b - other.b).powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_round_trips() {
        let white = Rgb::from_hex("#ffffff").unwrap();
        let lab = white.to_xyz().to_lab();
        assert!((lab.l - 100.0).abs() < 0.01);
        assert!(lab.a.abs() < 0.01);
        assert!(lab.b.abs() < 0.01);

        let back = lab.to_xyz().to_rgb();
        assert_eq!(back.to_hex(), "#ffffff");
    }

    #[test]
    fn black_is_zero_lightness() {
        let lab = Rgb::from_hex("#000000").unwrap().to_xyz().to_lab();
        assert!(lab.l.abs() < 0.01);
    }

    #[test]
    fn hex_round_trip_is_stable() {
        let orig = Rgb::from_hex("#3366cc").unwrap();
        let back = orig.to_xyz().to_rgb();
        assert_eq!(orig.to_hex(), back.to_hex());
    }

    #[test]
    fn rejects_bad_hex() {
        assert!(Rgb::from_hex("#abc").is_err());
        assert!(Rgb::from_hex("nothex1").is_err());
    }
}
