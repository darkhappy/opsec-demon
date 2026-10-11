// Procedural concert backdrop: no textures or external asset downloads.
struct Controls { time: f32, aspect: f32, burst: f32, page: f32 }
@group(0) @binding(0) var<uniform> c: Controls;
struct Vertex { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vertex(@builtin(vertex_index) i: u32) -> Vertex {
    var p = array<vec2<f32>, 3>(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
    var v: Vertex; v.position = vec4(p[i], 0., 1.); v.uv = p[i] * 0.5 + 0.5; return v;
}
fn glow(d: f32, width: f32) -> f32 { return width / (abs(d) + width); }
@fragment fn fragment(v: Vertex) -> @location(0) vec4<f32> {
    var p = (v.uv - 0.5) * vec2(c.aspect, 1.);
    let r = length(p); let a = atan2(p.y, p.x);
    let cyan = vec3(0.03, 0.72, 0.88); let pink = vec3(0.85, 0.06, 0.48);
    var color = vec3(0.008, 0.013, 0.032);
    let ring = 0.24 + 0.016 * sin(a * 12. + c.time * 1.6);
    color += mix(cyan, pink, 0.5 + 0.5 * sin(a + c.time * 0.2)) * glow(r - ring, 0.0015) * 0.48;
    color += cyan * glow(r - 0.3, 0.0006) * 0.14;
    let spokes = pow(max(0., cos(a * 8. - c.time * 0.4)), 64.) * exp(-abs(r - 0.26) * 14.);
    color += pink * spokes * 0.2;
    // Perspective floor and travelling scan wave.
    let ground = max(0.03, p.y + 0.51);
    let grid_x = abs(fract(p.x / ground * 1.6 + 0.5) - 0.5);
    let grid_y = abs(fract(0.45 / ground + c.time * 0.12) - 0.5);
    color += cyan * (glow(grid_x, 0.008) + glow(grid_y, 0.008)) * 0.035;
    color += pink * exp(-pow((p.y - 0.25 * sin(c.time * 0.4)) * 20., 2.)) * 0.035;
    // Orbiting stars and angular laser beams.
    for (var i = 0u; i < 28u; i += 1u) {
        let f = f32(i); let angle = f * 2.39996 + c.time * (0.025 + f * 0.0006);
        let star = vec2(cos(angle), sin(angle)) * (0.18 + fract(f * 0.618) * 0.75);
        color += mix(cyan, pink, fract(f * 0.381)) * exp(-length(p - star) * 700.) * 0.7;
    }
    let beam = pow(max(0., cos(a * 3. + c.time * 0.17)), 180.);
    color += cyan * beam * 0.06;
    // Shared simulation celebration, triggered by the actual model events.
    color += mix(pink, vec3(1., 0.48, 0.04), 0.5) * glow(r - (1. - c.burst) * 0.7, 0.004) * c.burst;
    color *= 0.85 + 0.15 * sin(c.page + a);
    return vec4(color, 1.);
}
