// WGSL adaptation of AMD FidelityFX FSR 1 (EASU / RCAS), scalar FP32 path.
// Source: GPUOpen-Effects/FidelityFX-FSR/ffx-fsr/ffx_fsr1.h (v1.0.2).
// Copyright (c) 2021 Advanced Micro Devices, Inc. All rights reserved.
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

struct Controls { output: vec2<f32>, sharpness: f32, fsr: f32 }
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var<uniform> c: Controls;
struct Vertex { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vertex(@builtin(vertex_index) i: u32) -> Vertex {
    let p = array<vec2<f32>,3>(vec2(-1.,-1.),vec2(3.,-1.),vec2(-1.,3.));
    var v: Vertex; v.position = vec4(p[i],0.,1.); v.uv = p[i] * vec2(0.5,-0.5) + 0.5; return v;
}
fn load(p: vec2<i32>) -> vec3<f32> {
    return textureLoad(image,clamp(p,vec2(0),vec2<i32>(textureDimensions(image))-1),0).rgb;
}
fn luma(v: vec3<f32>) -> f32 { return v.g * 1. + (v.r + v.b) * 0.5; }
// Direction and edge length for one of the bilinearly weighted four centers.
fn edge(w: f32, a: f32, b: f32, e: f32, d: f32, h: f32) -> vec3<f32> {
    let dx = d-b; let dy = h-a;
    let lx = clamp(abs(dx) / max(max(abs(d-e),abs(e-b)),1e-8),0.,1.);
    let ly = clamp(abs(dy) / max(max(abs(h-e),abs(e-a)),1e-8),0.,1.);
    return vec3(dx*w,dy*w,(lx*lx+ly*ly)*w);
}
@fragment fn easu(v: Vertex) -> @location(0) vec4<f32> {
    if c.fsr < 0.5 { return textureSampleLevel(image,linear_sampler,v.uv,0.); }
    let position = v.position.xy * vec2<f32>(textureDimensions(image)) / c.output - 0.5;
    let fp = vec2<i32>(floor(position)); let pp = fract(position);
    let offsets = array<vec2<i32>,12>(vec2(0,-1),vec2(1,-1),vec2(-1,0),vec2(0,0),vec2(1,0),vec2(2,0),vec2(-1,1),vec2(0,1),vec2(1,1),vec2(2,1),vec2(0,2),vec2(1,2));
    var rgb: array<vec3<f32>,12>; var l: array<f32,12>;
    for (var i=0u;i<12u;i+=1u) { rgb[i]=load(fp+offsets[i]); l[i]=luma(rgb[i]); }
    var gradient = edge((1.-pp.x)*(1.-pp.y),l[0],l[2],l[3],l[4],l[7])
                 + edge(pp.x*(1.-pp.y),l[1],l[3],l[4],l[5],l[8])
                 + edge((1.-pp.x)*pp.y,l[3],l[6],l[7],l[8],l[10])
                 + edge(pp.x*pp.y,l[4],l[7],l[8],l[9],l[11]);
    var dir = vec2(1.,0.);
    if dot(gradient.xy,gradient.xy) >= 1./32768. { dir = normalize(gradient.xy); }
    let strength = pow(gradient.z*0.5,2.);
    let stretch = dot(dir,dir)/max(abs(dir.x),abs(dir.y));
    let len = vec2(1.+(stretch-1.)*strength,1.-0.5*strength);
    let lob = 0.5+(0.21-0.5)*strength; let clip = 1./lob;
    var color = vec3(0.); var weight = 0.;
    for (var i=0u;i<12u;i+=1u) {
        let off=vec2<f32>(offsets[i])-pp;
        let rotated=vec2(dot(off,dir),dot(off,vec2(-dir.y,dir.x)))*len;
        let distance=min(dot(rotated,rotated),clip);
        let base=0.4*distance-1.; let window=lob*distance-1.;
        let w=(1.5625*base*base-0.5625)*window*window;
        color+=rgb[i]*w; weight+=w;
    }
    let lo=min(min(rgb[3],rgb[4]),min(rgb[7],rgb[8]));
    let hi=max(max(rgb[3],rgb[4]),max(rgb[7],rgb[8]));
    return vec4(clamp(color/max(weight,1e-8),lo,hi),1.);
}
@fragment fn rcas(v: Vertex) -> @location(0) vec4<f32> {
    let p=vec2<i32>(v.position.xy);
    let e=load(p);
    if c.fsr < 0.5 || c.sharpness <= 0. { return vec4(e,1.); }
    let b=load(p+vec2(0,-1)); let d=load(p+vec2(-1,0));
    let f=load(p+vec2(1,0)); let h=load(p+vec2(0,1));
    let mn=min(min(b,d),min(f,h)); let mx=max(max(b,d),max(f,h));
    let hit_min=min(mn,e)/max(4.*mx,vec3(1e-8));
    let hit_max=(1.-max(mx,e))/min(4.*mn-4.,vec3(-1e-8));
    let limiter=max(-hit_min,hit_max);
    let lobe=max(-0.1875,min(max(max(limiter.r,limiter.g),limiter.b),0.))*c.sharpness;
    return vec4(clamp((lobe*(b+d+f+h)+e)/(4.*lobe+1.),vec3(0.),vec3(1.)),1.);
}
