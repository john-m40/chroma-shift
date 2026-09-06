# chroma-shift

sRGB hex codes tell you almost nothing about how a colour will look next to
another one, or how far apart two colours actually are perceptually. "#3366cc"
and "#3366cd" are one bit apart in hex and invisible to the eye; "#3366cc" and
"#336633" are one channel apart and obviously not. RGB distance doesn't match
perception. CIE Lab space was built so that Euclidean distance in it roughly
does.

This is a small library plus CLI for moving between the colour spaces you
need to reason about that: sRGB (what you type as hex), linear-light RGB (what
displays actually mix), CIE XYZ (the device-independent space in between), and
CIE Lab (the perceptually-uniform one).

No dependencies. It's a few reference formulas from the CIE and IEC specs,
written out directly, plus a thin CLI.

## Library usage

```rust
use chroma_shift::Rgb;

let rgb = Rgb::from_hex("#3366cc").unwrap();
let lab = rgb.to_xyz().to_lab();
println!("L*={:.1} a*={:.1} b*={:.1}", lab.l, lab.a, lab.b);

let hsl = rgb.to_hsl();
println!("hsl({:.0} {:.0}% {:.0}%)", hsl.h, hsl.s * 100.0, hsl.l * 100.0);
```

`Rgb::to_hsl` / `Rgb::to_hsv` and their `to_rgb` inverses match the CSS
`hsl()`/`hsv()` model. They're not perceptually uniform - hue, saturation and
lightness/value don't correspond to equal steps in how different colours
look - so they're an on-ramp for people arriving with CSS colours, not a
replacement for Lab when you actually need to measure colour difference.

## CLI usage

```
$ chroma-shift '#3366cc'
hex: #3366cc
xyz: 0.1701 0.1456 0.5903
lab: 45.03 18.71 -57.85

$ chroma-shift lab 45.03 18.71 -57.85
hex: #3366cc

$ chroma-shift diff '#000000' '#808080'
delta-E76:   53.584
delta-E2000: 39.934
```

`diff` reports both dE76 and dE2000. dE76 is plain Euclidean distance in
Lab. dE2000 weights that distance by lightness, chroma and hue instead of
treating them as interchangeable, so it tracks perceived difference better
- above, going from black to mid grey is a pure lightness change, and dE2000
comes out noticeably lower than dE76 because its lightness term is scaled
down the further the pair sits from L*=50. "#3366cc" vs "#3366cd" (one bit
apart in hex) makes the same point at the other end: both metrics put it
near zero, while "#3366cc" vs "#336633" (one channel changed by the same
numeric amount) puts both metrics much higher - RGB distance doesn't track
perception, and Lab distance, in either flavour, roughly does.

## Building

Standard `cargo build` / `cargo run -- '#3366cc'` / `cargo test`. No external
crates, so there's nothing to fetch.

## Status

First cut. Handles sRGB/XYZ/Lab round trips, both CIE76 and CIEDE2000
delta-E, and HSL/HSV conversions. Still missing: a gamut clipping warning
when Lab -> RGB falls outside 0..1, 3-digit and 8-digit (alpha) hex input,
and a palette command for perceptually even colour ramps.
