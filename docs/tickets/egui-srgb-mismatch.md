# egui sRGB framebuffer mismatch

## Summary

egui is rendering against an sRGB-encoded framebuffer (`Rgba8UnormSrgb`) but expects a linear-RGB target. Result: egui's gamma correction is applied on top of the GPU's automatic sRGB encoding, producing colors that are slightly washed out / too bright. Currently masked by the debug panel using only neutral greys, but will become visible as egui usage expands.

## Symptom

On startup:
```
WARN egui_wgpu::renderer] Detected a linear (sRGBA aware) framebuffer Rgba8UnormSrgb. egui prefers Rgba8Unorm or Bgra8Unorm
```

Visual: any non-grey egui colors (button highlights, custom theme accents, sliders, status colors) will appear lighter and less saturated than authored.

## Root cause

Two systems disagree about who's responsible for sRGB conversion:

1. **The wgpu surface** is configured as `Rgba8UnormSrgb`. The GPU automatically applies sRGB encoding when shader output is written to this surface. This is the right format for the 3D scene because the lighting math runs in linear space and we want sRGB on screen.
2. **egui** assumes its target is `Rgba8Unorm` (linear). It pre-applies sRGB encoding in its own shader, expecting the GPU to write the bytes through unchanged.

When egui's pre-encoded output hits the sRGB surface, the GPU encodes again — double-encoding lifts midtones and dulls contrast.

## Options

### A. Tell egui the surface is sRGB
egui's `Renderer` accepts a flag indicating the target's gamma behavior. Setting it to "already sRGB" makes egui skip its own conversion, leaving one encoding step instead of two. Simplest fix; one config change in `EguiState::new`.

### B. Switch the surface to `Rgba8Unorm` (linear)
Then the 3D shaders need to perform sRGB conversion themselves before writing the final color. More invasive — touches every fragment shader that writes to the surface — but matches what's typically considered the "correct" pipeline (linear throughout, convert at the very end).

### C. Render egui to an offscreen `Rgba8Unorm` texture, then composite onto the sRGB surface
Adds a render pass. Most flexible but most code. Unnecessary for now.

## Recommendation

Go with **Option A**. The 3D pipeline already handles sRGB correctly; only egui needs adjusting. One-line change. Validate by checking that egui's default theme colors look correct against a reference screenshot.

## Acceptance

- [ ] Warning no longer appears in console
- [ ] egui debug panel text and background still legible
- [ ] A button or slider rendered with a non-grey color (e.g. blue) matches the same color rendered in a known-correct egui demo

## References

- egui_wgpu config: see `egui-wgpu` crate docs for the gamma/sRGB flag
- Surface format chosen in: `src/engine/state/engine_state.rs` (look for `swapchain_format`)
- egui state init: `src/engine/ui/egui_state.rs`

## Priority

Low. Cosmetic only. Pin until egui usage grows beyond grey debug labels.
