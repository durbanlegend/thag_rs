{
let (r, g, b) = (252, 243, 210);
// f64::from(b).mul_add(0.114, f64::from(r).mul_add(0.299, f64::from(g) * 0.587))
println!("{}", f64::from(b).mul_add(
        0.114,
        f64::from(r).mul_add(0.299, f64::from(g) * 0.587),
    ) / 256.0);
let luma_255 = 0.2126 * (r as f32) + 0.7152 * (g as f32) + 0.0722 * (b as f32);
let luma_0_to_1 = luma_255 / 255.0;
println!("luma_0_to_1={luma_0_to_1}");
}
