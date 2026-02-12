//! Roundtrip example: generate a test pattern, send via Syphon (Metal and OpenGL) server,
//! receive via Syphon client, then verify sent and received pixels match.
//!
//! Run on macOS: cargo run --example roundtrip

#![cfg(target_os = "macos")]

use metal::foreign_types::{ForeignType, ForeignTypeRef};
use metal::{Device, MTLPixelFormat, MTLRegion, MTLTextureType, Texture, TextureDescriptor};
use rusty_syphon::{
    cgl_create_headless_context, cgl_destroy_context, cgl_make_current,
    gl_create_texture_rectangle_rgba8, gl_delete_texture, gl_read_texture_rectangle_rgba8,
    OpenGLClient, OpenGLImage, OpenGLServer, MetalClient, MetalServer, MetalTexture,
    GL_TEXTURE_RECTANGLE,
};
use std::time::Duration;

const W: usize = 64;
const H: usize = 64;
const BYTES_PER_PIXEL: usize = 4;
const TOTAL_BYTES: usize = W * H * BYTES_PER_PIXEL;

/// Deterministic test pattern: each pixel (x, y) = RGBA(x%256, y%256, (x+y)%256, 255).
fn make_test_pattern() -> Vec<u8> {
    let mut data = vec![0u8; TOTAL_BYTES];
    for y in 0..H {
        for x in 0..W {
            let i = (y * W + x) * BYTES_PER_PIXEL;
            data[i] = (x % 256) as u8;     // R
            data[i + 1] = (y % 256) as u8; // G
            data[i + 2] = ((x + y) % 256) as u8; // B
            data[i + 3] = 255;             // A
        }
    }
    data
}

fn test_metal_roundtrip() {
    let device = Device::system_default().expect("no Metal device");
    let queue = device.new_command_queue();

    // 1. Create texture and fill with test pattern (Metal uses BGRA in our descriptor)
    let desc = TextureDescriptor::new();
    desc.set_texture_type(MTLTextureType::D2);
    desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    desc.set_width(W as u64);
    desc.set_height(H as u64);
    desc.set_usage(metal::MTLTextureUsage::Unknown);

    let texture = device.new_texture(&desc);
    let pattern = make_test_pattern();
    // Metal BGRA8Unorm: upload as BGRA to match
    let mut bgra = vec![0u8; TOTAL_BYTES];
    for i in (0..TOTAL_BYTES).step_by(4) {
        bgra[i] = pattern[i + 2];     // B
        bgra[i + 1] = pattern[i + 1]; // G
        bgra[i + 2] = pattern[i];     // R
        bgra[i + 3] = pattern[i + 3]; // A
    }
    let region = MTLRegion::new_2d(0, 0, W as u64, H as u64);
    texture.replace_region(region, 0, bgra.as_ptr() as *const _, (W * BYTES_PER_PIXEL) as u64);

    // 2. Create Syphon Metal server and publish the frame
    let server = MetalServer::new(
        Some("rusty-syphon-roundtrip-metal"),
        device.as_ptr() as *mut _,
        None,
    )
    .expect("create Metal server");

    let cmd_buf = queue.new_command_buffer();
    server.publish_frame(
        texture.as_ptr() as *mut _,
        cmd_buf.as_ptr() as *mut _,
        0.0,
        0.0,
        W as f64,
        H as f64,
        false,
    );
    cmd_buf.commit();
    cmd_buf.wait_until_completed();

    // 3. Get server description and create client (same process roundtrip)
    let server_desc = server.server_description().expect("server description");
    std::thread::sleep(Duration::from_millis(50));

    let client = MetalClient::new(&server_desc, device.as_ptr() as *mut _, None, None)
        .expect("create Metal client");

    // 4. Poll until we have a new frame, then read it back
    for _ in 0..100 {
        if client.has_new_frame() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(client.has_new_frame(), "Metal client never got a new frame");

    let received: MetalTexture = client.new_frame_image().expect("new frame image");
    let received_tex = unsafe { Texture::from_ptr(received.as_ptr() as *mut metal::MTLTexture) };
    let mut readback = vec![0u8; TOTAL_BYTES];
    let region = MTLRegion::new_2d(0, 0, W as u64, H as u64);
    received_tex.get_bytes(
        readback.as_mut_ptr() as *mut _,
        (W * BYTES_PER_PIXEL) as u64,
        region,
        0,
    );
    std::mem::forget(received_tex);
    drop(received);

    // 5. Compare: readback is BGRA, pattern is RGBA — convert readback to RGBA for comparison
    let mut received_rgba = vec![0u8; TOTAL_BYTES];
    for i in (0..TOTAL_BYTES).step_by(4) {
        received_rgba[i] = readback[i + 2];     // R
        received_rgba[i + 1] = readback[i + 1]; // G
        received_rgba[i + 2] = readback[i];     // B
        received_rgba[i + 3] = readback[i + 3]; // A
    }

    assert_eq!(
        pattern, received_rgba,
        "Metal roundtrip: sent and received pixels differ"
    );
    println!("OK Metal: sent and received images match ({} bytes)", TOTAL_BYTES);
}

fn test_opengl_roundtrip() {
    let ctx = cgl_create_headless_context().expect("create headless CGL context");
    cgl_make_current(ctx);

    let pattern = make_test_pattern();
    let tex_id = gl_create_texture_rectangle_rgba8(W, H, &pattern);
    assert!(tex_id != 0, "create OpenGL texture failed");

    let server = OpenGLServer::new(Some("rusty-syphon-roundtrip-opengl"), ctx, None)
        .expect("create OpenGL server");
    server.publish_frame(
        tex_id,
        GL_TEXTURE_RECTANGLE,
        0.0,
        0.0,
        W as f64,
        H as f64,
        W as f64,
        H as f64,
        false,
    );

    let server_desc = server.server_description().expect("server description");
    std::thread::sleep(Duration::from_millis(50));

    let client = OpenGLClient::new(&server_desc, ctx, None, None).expect("create OpenGL client");

    for _ in 0..100 {
        if client.has_new_frame() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(client.has_new_frame(), "OpenGL client never got a new frame");

    let image: OpenGLImage = client.new_frame_image().expect("new frame image");
    let recv_tex = image.texture_name();
    let (tw, th) = image.texture_size();
    assert!((tw - W as f64).abs() < 1.0 && (th - H as f64).abs() < 1.0, "unexpected texture size");
    let mut readback = vec![0u8; TOTAL_BYTES];
    gl_read_texture_rectangle_rgba8(recv_tex, W, H, &mut readback);
    drop(image);

    if pattern != readback {
        let mut first_diff = None;
        for (i, (a, b)) in pattern.iter().zip(readback.iter()).enumerate() {
            if a != b {
                first_diff = Some((i, *a, *b));
                break;
            }
        }
        panic!(
            "OpenGL roundtrip: first diff at byte {:?} (sent {}, received {})",
            first_diff.map(|(i, _, _)| i),
            first_diff.map(|(_, a, _)| a).unwrap_or(0),
            first_diff.map(|(_, _, b)| b).unwrap_or(0)
        );
    }
    println!("OK OpenGL: sent and received images match ({} bytes)", TOTAL_BYTES);

    gl_delete_texture(tex_id);
    cgl_destroy_context(ctx);
}

fn main() {
    test_metal_roundtrip();
    test_opengl_roundtrip();
    println!("All roundtrip tests passed.");
}
