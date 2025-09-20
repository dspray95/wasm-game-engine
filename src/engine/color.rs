use std::f64;
use log::info;

#[inline]
pub fn srgb_to_linear_approx_2_2(srgb_component: u32) -> f64 {
    info!("srgb_to_linear_approx_2_2: Processing sRGB component {}", srgb_component);

    let normalised_srgb = (srgb_component as f64) / 255.0;
    info!("srgb_to_linear_approx_2_2: Normalized sRGB value: {:.15}", normalised_srgb);

    // Simple power 2.2 curve (often used as an approximation)
    let linear_result = normalised_srgb.powf(2.2);
    info!(
        "srgb_to_linear_approx_2_2: Using powf(2.2) approximation. Result: {:.15}",
        linear_result
    );
    linear_result
}
