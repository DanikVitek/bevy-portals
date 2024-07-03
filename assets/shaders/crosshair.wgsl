// This shader draws a crosshair with a given input color
#import bevy_ui::ui_vertex_output::UiVertexOutput

struct CrosshairMaterial {
    @location(0) color: vec4<f32>
}

@group(1) @binding(0)
var<uniform> input: CrosshairMaterial;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // the UVs are now adjusted around the middle of the rect.
    let uv = in.uv - 0.5;

    let alpha = min(step(abs(uv.x), 0.05) + step(abs(uv.y), 0.05), 1.0);

    return vec4<f32>(input.color.rgb, alpha);
    // return vec4<f32>(abs(uv), 0.0, 1.0);
}