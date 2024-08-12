#import bevy_pbr::{
    mesh_view_bindings::{view, globals},
    forward_io::VertexOutput,
    utils::coords_to_viewport_uv,
}
// #import bevy_render::view::View

fn rand(st: vec2<f32>) -> f32 {
	return fract(sin(dot(st.xy, vec2<f32>(12.9898, 78.233))) * 43758.547);
}

fn noise(st: vec2<f32>) -> f32 {
	let i: vec2<f32> = floor(st);
	let f: vec2<f32> = fract(st);
	let a: f32 = rand(i);
	let b: f32 = rand(i + vec2<f32>(1., 0.));
	let c: f32 = rand(i + vec2<f32>(0., 1.));
	let d: f32 = rand(i + vec2<f32>(1., 1.));
	let avg: f32 = (a + b + c + d) / 4.;
	return smoothstep(0., 1., avg);
}

// @group(0) @binding(0) var<uniform> view: View;
@group(2) @binding(101) var<uniform> shader: BackgroundShader;
struct BackgroundShader{
    pos : vec2f,
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {

    // Pixel coordinates (centre of pixel, origin at bottom left)
    let frag_coord = mesh.position.xy;

    // Normalised pixel coordinates (from 0 to 1)
    let uv = coords_to_viewport_uv(mesh.position.xy, view.viewport);



	let space_color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
	var color: vec4<f32> = vec4<f32>(space_color);

	// 1st layer
	let layer_1_offset: vec2<f32> = frag_coord + (shader.pos * 84.0);
	var strength: f32 = smoothstep(0.98, 1., noise(layer_1_offset));
	var star_color: vec4<f32> = vec4<f32>(1., 1., 0.9, 1.0) * strength * 0.7;

	color = color + star_color;

	// 2nd later
	let scale: f32 = 1.25;
	let layer_2_offset: vec2<f32> = frag_coord + (shader.pos * (84.0 / scale));
	strength = smoothstep(0.99, 1., noise(layer_2_offset));
	star_color = vec4<f32>(1., 0.8, 0.9, 1.0) * strength * 0.5;

	color = color + star_color;

	return color;
}


// @fragment
// fn fragment(
//     mesh: VertexOutput,
// ) -> @location(0) vec4<f32> {
//     let viewport_uv = coords_to_viewport_uv(mesh.position.xy, view.viewport);
//     let color = (viewport_uv, viewport_uv);
//     return color;
// }
