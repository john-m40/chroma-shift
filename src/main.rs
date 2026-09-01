use chroma_shift::{Lab, Rgb};
use std::env;
use std::process::ExitCode;

fn usage() -> String {
    "usage:\n  chroma-shift <#rrggbb>              show xyz + lab for a hex colour\n  chroma-shift lab <L> <a> <b>        show the closest hex colour for a lab triple\n  chroma-shift diff <#rrggbb> <#rrggbb>  CIE76 delta-E between two colours".to_string()
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
            Ok(format!("hex: {}", rgb.to_hex()))
        }
        [cmd, hex1, hex2] if cmd == "diff" => {
            let lab1 = Rgb::from_hex(hex1)?.to_xyz().to_lab();
            let lab2 = Rgb::from_hex(hex2)?.to_xyz().to_lab();
            Ok(format!("delta-E76: {:.3}", lab1.delta_e76(lab2)))
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
