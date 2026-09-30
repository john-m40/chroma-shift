use chroma_shift::{palette, Lab, Rgb};
use std::env;
use std::process::ExitCode;

fn usage() -> String {
    "usage:\n  chroma-shift <hex>                  show xyz + lab for a hex colour (#rgb, #rrggbb or #rrggbbaa)\n  chroma-shift lab <L> <a> <b>        show the closest hex colour for a lab triple\n  chroma-shift diff <#rrggbb> <#rrggbb>  dE76 and dE2000 between two colours\n  chroma-shift palette <hex> <hex> <n>  n colours evenly spaced in Lab between two colours (2..=256)".to_string()
}

fn run(args: &[String]) -> Result<String, String> {
    match args {
        [hex] => {
            let rgb = Rgb::from_hex(hex)?;
            let xyz = rgb.to_xyz();
            let lab = xyz.to_lab();
            Ok(format!(
                "hex: {}\nxyz: {:.4} {:.4} {:.4}\nlab: {:.2} {:.2} {:.2}",
                rgb.to_hex(),
                xyz.x,
                xyz.y,
                xyz.z,
                lab.l,
                lab.a,
                lab.b
            ))
        }
        [cmd, l, a, b] if cmd == "lab" => {
            let parse = |s: &str| s.parse::<f64>().map_err(|e| format!("bad number {:?}: {}", s, e));
            let lab = Lab { l: parse(l)?, a: parse(a)?, b: parse(b)? };
            let rgb = lab.to_xyz().to_rgb();
            let mut out = format!("hex: {}", rgb.to_hex());
            if rgb.is_out_of_gamut() {
                out.push_str("\nwarning: outside sRGB gamut, clipped to nearest displayable colour");
            }
            Ok(out)
        }
        [cmd, hex1, hex2] if cmd == "diff" => {
            let lab1 = Rgb::from_hex(hex1)?.to_xyz().to_lab();
            let lab2 = Rgb::from_hex(hex2)?.to_xyz().to_lab();
            Ok(format!(
                "delta-E76:   {:.3}\ndelta-E2000: {:.3}",
                lab1.delta_e76(lab2),
                lab1.delta_e2000(lab2)
            ))
        }
        [cmd, hex1, hex2, n] if cmd == "palette" => {
            let steps: usize = n.parse().map_err(|e| format!("bad step count {:?}: {}", n, e))?;
            if !(2..=256).contains(&steps) {
                return Err("step count must be between 2 and 256".to_string());
            }
            let from = Rgb::from_hex(hex1)?.to_xyz().to_lab();
            let to = Rgb::from_hex(hex2)?.to_xyz().to_lab();
            let mut clipped = false;
            let lines: Vec<String> = palette(from, to, steps)
                .into_iter()
                .map(|lab| {
                    let rgb = lab.to_xyz().to_rgb();
                    clipped |= rgb.is_out_of_gamut();
                    rgb.to_hex()
                })
                .collect();
            let mut out = lines.join("\n");
            if clipped {
                out.push_str("\nwarning: some steps fall outside sRGB gamut and were clipped");
            }
            Ok(out)
        }
        _ => Err(usage()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::FAILURE
        }
    }
}
