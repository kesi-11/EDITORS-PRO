/// Multi-layer composition compute shader
///
/// Bind group 0:
///   [0] uniform Params — layer_count, pad1, pad2, pad3
///   [1] background texture_2d<f32>
///   [2] output texture_storage_2d<rgba8unorm, write>
///   [3] uniform OverlayParams — opacity, position_x, position_y, pad
///
/// For the MVP, composites a single overlay layer onto the background
/// using source-over (Porter-Duff) alpha blending.
///
/// The overlay is the same size as the background texture (loaded as
/// input_tex in a separate dispatch). In practice, the overlay texture
/// is supplied by the caller through a second texture binding; for the
/// MVP we treat the background as the base and the overlay as an
/// additional texture at binding 4, or alternatively we pack overlay
/// info into the uniform buffers.

struct Params {
    layer_count: f32,
    pad1: f32,
    pad2: f32,
    pad3: f32,
};

struct OverlayParams {
    opacity: f32,
    position_x: f32,
    position_y: f32,
    pad: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var background_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<uniform> overlay_params: OverlayParams;
@group(0) @binding(4) var overlay_tex: texture_2d<f32>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let dims = textureDimensions(background_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }

    let bg = textureLoad(background_tex, id.xy, 0);

    // Determine the overlay pixel coordinate accounting for position offset
    let overlay_x = i32(id.x) - i32(overlay_params.position_x);
    let overlay_y = i32(id.y) - i32(overlay_params.position_y);
    let overlay_dims = textureDimensions(overlay_tex);

    // Check if this output pixel is covered by the overlay
    var result = bg;

    if (overlay_x >= 0 && overlay_x < i32(overlay_dims.x) &&
        overlay_y >= 0 && overlay_y < i32(overlay_dims.y))
    {
        let overlay = textureLoad(overlay_tex, vec2i(overlay_x, overlay_y), 0);

        // Source-over alpha compositing (Porter-Duff)
        let src_alpha = overlay.a * overlay_params.opacity;
        let dst_alpha = bg.a;

        let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

        if (out_alpha > 0.0) {
            let out_r = (overlay.r * src_alpha + bg.r * dst_alpha * (1.0 - src_alpha)) / out_alpha;
            let out_g = (overlay.g * src_alpha + bg.g * dst_alpha * (1.0 - src_alpha)) / out_alpha;
            let out_b = (overlay.b * src_alpha + bg.b * dst_alpha * (1.0 - src_alpha)) / out_alpha;
            result = vec4f(out_r, out_g, out_b, out_alpha);
        }
        // If out_alpha == 0, result stays as bg (which is transparent black)
    }

    textureStore(output_tex, id.xy, result);
}
