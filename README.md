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
```

## CLI usage

```
$ chroma-shift '#3366cc'
hex: #3366cc
xyz: 0.1938 0.1729 0.5163
lab: 48.44 8.98 -46.68

$ chroma-shift lab 48.44 8.98 -46.68
hex: #3366cc

$ chroma-shift diff '#3366cc' '#3366cd'
delta-E76: 0.045

$ chroma-shift diff '#3366cc' '#336633'
delta-E76: 41.238
```

`diff` is what makes the perceptual point concrete: the first pair is a
one-bit hex change and the delta-E is near zero, the second pair changes one
channel by the same numeric amount and the delta-E is enormous.

## Building

Standard `cargo build` / `cargo run -- '#3366cc'` / `cargo test`. No external
crates, so there's nothing to fetch.

## Status

First cut. Handles sRGB/XYZ/Lab round trips and a CIE76 delta-E. Still
missing: dE2000 (more accurate but a lot more code), a gamut clipping
warning when Lab -> RGB falls outside 0..1, and HSL/HSV as an on-ramp for
people coming from CSS.
