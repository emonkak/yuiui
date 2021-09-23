[[block]]
struct Uniforms {
    projection: mat4x4<f32>;
    transform: mat4x4<f32>;
    scale_factor: f32;
};

[[group(0), binding(0)]] var<uniform> uniforms: Uniforms;

struct VertexInput {
    [[builtin(vertex_index)]] vertex_index: u32;
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] size: vec2<f32>;
    [[location(2)]] color: vec4<f32>;
    [[location(3)]] border_color: vec4<f32>;
    [[location(4)]] border_radius: f32;
    [[location(5)]] border_width: f32;
};

struct VertexOutput {
    [[builtin(position)]] coord: vec4<f32>;
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] size: vec2<f32>;
    [[location(2)]] color: vec4<f32>;
    [[location(3)]] border_color: vec4<f32>;
    [[location(4)]] border_radius: f32;
    [[location(5)]] border_width: f32;
};

var RECTANGLE_POSITIONS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0)
);

[[stage(vertex)]]
fn vs_main(input: VertexInput) -> VertexOutput {
    let position: vec2<f32> = input.position * uniforms.scale_factor;
    let size: vec2<f32> = input.size * uniforms.scale_factor;

    let border_radius: f32 = min(
        input.border_radius,
        min(input.size.x, input.size.y) / 2.0
    );

    let transform: mat4x4<f32> = uniforms.projection
        * uniforms.transform
        * mat4x4<f32>(
            vec4<f32>(size.x + 1.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, size.y + 1.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 1.0, 0.0),
            vec4<f32>(position - vec2<f32>(0.5), 0.0, 1.0)
        );

    let rectangle_position: vec2<f32> = RECTANGLE_POSITIONS[input.vertex_index];

    var output: VertexOutput;
    output.position = position;
    output.size = size;
    output.color = input.color;
    output.border_color = input.border_color;
    output.border_radius = border_radius * uniforms.scale_factor;
    output.border_width = input.border_width * uniforms.scale_factor;
    output.coord = transform * vec4<f32>(rectangle_position, 0.0, 1.0);

    return output;
}

fn round_corner_distance(
    coord: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: f32
) -> f32 {
    let inner_size: vec2<f32> = size - vec2<f32>(radius, radius) * 2.0;
    let top_left: vec2<f32> = position + vec2<f32>(radius, radius);
    let bottom_right: vec2<f32> = top_left + inner_size;

    let top_left_distance: vec2<f32> = top_left - coord;
    let bottom_right_distance: vec2<f32> = coord - bottom_right;

    let distance: vec2<f32> = vec2<f32>(
        max(max(top_left_distance.x, bottom_right_distance.x), 0.0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0.0)
    );

    return length(distance);
}


[[stage(fragment)]]
fn fs_main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    let coord = input.coord * uniforms.transform;

    var mixed_color: vec4<f32> = input.color;

    if (input.border_width > 0.0) {
        let internal_border: f32 = max(
            input.border_radius - input.border_width,
            0.0
        );

        let internal_distance: f32 = round_corner_distance(
            coord.xy,
            input.position + vec2<f32>(input.border_width, input.border_width),
            input.size - vec2<f32>(input.border_width * 2.0, input.border_width * 2.0),
            internal_border
        );

        let border_mix: f32 = smoothStep(
            max(internal_border - 0.5, 0.0),
            internal_border + 0.5,
            internal_distance
        );

        mixed_color = mix(input.color, input.border_color, vec4<f32>(border_mix, border_mix, border_mix, border_mix));
    }

    let round_corner_distance: f32 = round_corner_distance(
        coord.xy,
        input.position,
        input.size,
        input.border_radius
    );

    let radius_alpha: f32 = 1.0 - smoothStep(
        max(input.border_radius - 0.5, 0.0),
        input.border_radius + 0.5,
        round_corner_distance
    );

    return vec4<f32>(mixed_color.rgb, mixed_color.a * radius_alpha);
}
