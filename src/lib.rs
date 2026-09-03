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

    /// Perceptual distance (CIE76 dE). Cheap: just Euclidean distance in Lab.
    /// Good enough for small differences, but it doesn't correct for the
    /// eye's uneven sensitivity across hue and chroma the way dE2000 does.
    pub fn delta_e76(self, other: Lab) -> f64 {
        ((self.l - other.l).powi(2) + (self.a - other.a).powi(2) + (self.b - other.b).powi(2)).sqrt()
    }

    /// CIEDE2000 delta-E (Sharma, Wu & Dalal, 2005). More faithful to actual
    /// perceived difference than dE76, particularly for low-chroma and blue
    /// colours where dE76 is known to overstate the gap. The formula has a
    /// handful of empirical correction terms and singularities around
    /// achromatic colours (C ~= 0) that dE76 doesn't need to worry about,
    /// which is the whole reason it's kept as a separate method rather than
    /// replacing dE76.
    pub fn delta_e2000(self, other: Lab) -> f64 {
        let (l1, a1, b1) = (self.l, self.a, self.b);
        let (l2, a2, b2) = (other.l, other.a, other.b);

        let c1 = (a1 * a1 + b1 * b1).sqrt();
        let c2 = (a2 * a2 + b2 * b2).sqrt();
        let c_bar7 = ((c1 + c2) / 2.0).powi(7);
        let g = 0.5 * (1.0 - (c_bar7 / (c_bar7 + 25f64.powi(7))).sqrt());

        let a1p = a1 * (1.0 + g);
        let a2p = a2 * (1.0 + g);
        let c1p = (a1p * a1p + b1 * b1).sqrt();
        let c2p = (a2p * a2p + b2 * b2).sqrt();

        // Hue angle in degrees, 0 for the achromatic case rather than atan2's NaN-free
        // but arbitrary answer at the origin.
        let hue = |a: f64, b: f64| -> f64 {
            if a == 0.0 && b == 0.0 {
                0.0
            } else {
                let deg = b.atan2(a).to_degrees();
                if deg < 0.0 { deg + 360.0 } else { deg }
            }
        };
        let h1p = hue(a1p, b1);
        let h2p = hue(a2p, b2);

        let delta_lp = l2 - l1;
        let delta_cp = c2p - c1p;

        let delta_hp_deg = if c1p * c2p == 0.0 {
            0.0
        } else {
            let dh = h2p - h1p;
            if dh > 180.0 {
                dh - 360.0
            } else if dh < -180.0 {
                dh + 360.0
            } else {
                dh
            }
        };
        let delta_hp = 2.0 * (c1p * c2p).sqrt() * (delta_hp_deg.to_radians() / 2.0).sin();

        let l_bar_p = (l1 + l2) / 2.0;
        let c_bar_p = (c1p + c2p) / 2.0;
        let h_bar_p = if c1p * c2p == 0.0 {
            h1p + h2p
        } else if (h1p - h2p).abs() > 180.0 {
            if h1p + h2p < 360.0 {
                (h1p + h2p + 360.0) / 2.0
            } else {
                (h1p + h2p - 360.0) / 2.0
            }
        } else {
            (h1p + h2p) / 2.0
        };

        let t = 1.0 - 0.17 * (h_bar_p - 30.0).to_radians().cos()
            + 0.24 * (2.0 * h_bar_p).to_radians().cos()
            + 0.32 * (3.0 * h_bar_p + 6.0).to_radians().cos()
            - 0.20 * (4.0 * h_bar_p - 63.0).to_radians().cos();

        let delta_theta = 30.0 * (-(((h_bar_p - 275.0) / 25.0).powi(2))).exp();
        let c_bar_p7 = c_bar_p.powi(7);
        let rc = 2.0 * (c_bar_p7 / (c_bar_p7 + 25f64.powi(7))).sqrt();
        let sl = 1.0 + (0.015 * (l_bar_p - 50.0).powi(2)) / (20.0 + (l_bar_p - 50.0).powi(2)).sqrt();
        let sc = 1.0 + 0.045 * c_bar_p;
        let sh = 1.0 + 0.015 * c_bar_p * t;
        let rt = -(2.0 * delta_theta.to_radians()).sin() * rc;

        let term_l = delta_lp / sl;
        let term_c = delta_cp / sc;
        let term_h = delta_hp / sh;

        (term_l * term_l + term_c * term_c + term_h * term_h + rt * term_c * term_h).sqrt()
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

    #[test]
    fn delta_e2000_matches_reference_pairs() {
        // Values from Sharma, Wu & Dalal (2005), Table 1, the standard
        // reference dataset for checking a CIEDE2000 implementation. This
        // subset covers ordinary pairs plus the near-achromatic and
        // hue-wraparound cases that are easy to get subtly wrong.
        let cases: &[(Lab, Lab, f64)] = &[
            (
                Lab { l: 50.0, a: 2.6772, b: -79.7751 },
                Lab { l: 50.0, a: 0.0, b: -82.7485 },
                2.0425,
            ),
            (
                Lab { l: 50.0, a: -1.3802, b: -84.2814 },
                Lab { l: 50.0, a: 0.0, b: -82.7485 },
                1.0000,
            ),
            (
                Lab { l: 50.0, a: 2.4900, b: -0.0010 },
                Lab { l: 50.0, a: -2.4900, b: 0.0009 },
                7.1792,
            ),
            (
                Lab { l: 50.0, a: 2.5000, b: 0.0 },
                Lab { l: 73.0, a: 25.0, b: -18.0 },
                27.1492,
            ),
            (
                Lab { l: 50.0, a: 2.5000, b: 0.0 },
                Lab { l: 50.0, a: 3.2972, b: 0.0 },
                1.0000,
            ),
            (
                Lab { l: 60.2574, a: -34.0099, b: 36.2677 },
                Lab { l: 60.4626, a: -34.1751, b: 39.4387 },
                1.2644,
            ),
            (
                Lab { l: 2.0776, a: 0.0795, b: -1.1350 },
                Lab { l: 0.9033, a: -0.0636, b: -0.5514 },
                0.6377,
            ),
        ];
        for (lab1, lab2, expected) in cases {
            let got = lab1.delta_e2000(*lab2);
            assert!(
                (got - expected).abs() < 0.005,
                "delta_e2000({:?}, {:?}) = {}, expected {}",
                lab1,
                lab2,
                got,
                expected
            );
        }
    }
}
