//! Safe Rust API for Syphon (OpenGL and Metal server/client, server directory).
//!
//! OpenGL: CGL context and GL usage must follow Syphon's and macOS's rules.
//! Metal: pass `MTLDevice`/`MTLTexture`/`MTLCommandBuffer` pointers (e.g. from the `metal` crate).

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::NonNull;

#[cfg(target_os = "macos")]
use crate::ffi;

/// CGL context (from OpenGL/OpenGL.h). On macOS this is the real type from the FFI; elsewhere a placeholder.
#[cfg(target_os = "macos")]
pub type CGLContextObj = crate::ffi::CGLContextObj;
#[cfg(not(target_os = "macos"))]
pub type CGLContextObj = *mut std::ffi::c_void;

/// OpenGL texture target for rectangle textures (Syphon uses this).
pub const GL_TEXTURE_RECTANGLE: u32 = 0x84F5;

/// Create a headless CGL context for offscreen OpenGL (e.g. tests). Caller must destroy with `cgl_destroy_context`.
#[cfg(target_os = "macos")]
pub fn cgl_create_headless_context() -> Option<CGLContextObj> {
    let ctx = unsafe { ffi::syphon_cgl_create_headless_context() };
    if ctx.is_null() {
        None
    } else {
        Some(ctx)
    }
}
#[cfg(not(target_os = "macos"))]
pub fn cgl_create_headless_context() -> Option<CGLContextObj> {
    None
}

/// Destroy a CGL context created with `cgl_create_headless_context`.
#[cfg(target_os = "macos")]
pub fn cgl_destroy_context(ctx: CGLContextObj) {
    if !ctx.is_null() {
        unsafe { ffi::syphon_cgl_destroy_context(ctx) };
    }
}
#[cfg(not(target_os = "macos"))]
pub fn cgl_destroy_context(_ctx: CGLContextObj) {}

/// Make the given CGL context current on this thread.
#[cfg(target_os = "macos")]
pub fn cgl_make_current(ctx: CGLContextObj) {
    unsafe { ffi::syphon_cgl_make_current(ctx) };
}
#[cfg(not(target_os = "macos"))]
pub fn cgl_make_current(_ctx: CGLContextObj) {}

/// Create a GL_TEXTURE_RECTANGLE RGBA8 texture and upload `rgba` (width*height*4 bytes). CGL context must be current. Returns 0 on failure.
#[cfg(target_os = "macos")]
pub fn gl_create_texture_rectangle_rgba8(width: usize, height: usize, rgba: &[u8]) -> u32 {
    let expected = width * height * 4;
    if rgba.len() < expected {
        return 0;
    }
    unsafe {
        ffi::syphon_gl_create_texture_rectangle_rgba8(width, height, rgba.as_ptr())
    }
}
#[cfg(not(target_os = "macos"))]
pub fn gl_create_texture_rectangle_rgba8(_width: usize, _height: usize, _rgba: &[u8]) -> u32 {
    0
}

/// Read back a GL_TEXTURE_RECTANGLE texture into `out_rgba` (width*height*4 bytes). CGL context must be current.
#[cfg(target_os = "macos")]
pub fn gl_read_texture_rectangle_rgba8(tex_id: u32, width: usize, height: usize, out_rgba: &mut [u8]) {
    let expected = width * height * 4;
    if out_rgba.len() < expected {
        return;
    }
    unsafe {
        ffi::syphon_gl_read_texture_rectangle_rgba8(tex_id, width, height, out_rgba.as_mut_ptr());
    }
}
#[cfg(not(target_os = "macos"))]
pub fn gl_read_texture_rectangle_rgba8(_tex_id: u32, _width: usize, _height: usize, _out_rgba: &mut [u8]) {
}

/// Delete a GL texture created with `gl_create_texture_rectangle_rgba8` or returned by Syphon.
#[cfg(target_os = "macos")]
pub fn gl_delete_texture(tex_id: u32) {
    if tex_id != 0 {
        unsafe { ffi::syphon_gl_delete_texture(tex_id) };
    }
}
#[cfg(not(target_os = "macos"))]
pub fn gl_delete_texture(_tex_id: u32) {}

/// Server directory: shared singleton listing available Syphon servers.
pub struct ServerDirectory {
    #[cfg(target_os = "macos")]
    ptr: NonNull<std::ffi::c_void>,
}

/// A description of a Syphon server (from the directory or from a server's `server_description`).
/// If you retain it for longer than a directory snapshot, use `retain`/`release` or clone the strings.
pub struct ServerDescription {
    #[cfg(target_os = "macos")]
    ptr: NonNull<std::ffi::c_void>,
    /// If true, we own a retain and must release on drop.
    #[cfg(target_os = "macos")]
    owned: bool,
}

/// OpenGL Syphon server: publishes frames to clients.
pub struct OpenGLServer {
    #[cfg(target_os = "macos")]
    ptr: NonNull<std::ffi::c_void>,
}

/// OpenGL Syphon client: receives frames from a server.
pub struct OpenGLClient {
    #[cfg(target_os = "macos")]
    ptr: NonNull<std::ffi::c_void>,
    /// Keeps the callback alive and gives a stable pointer to the C side.
    #[cfg(target_os = "macos")]
    _callback_storage: Option<Box<CallbackHolder>>,
}

/// A single frame image from a client. Release promptly after drawing.
pub struct OpenGLImage {
    #[cfg(target_os = "macos")]
    ptr: NonNull<std::ffi::c_void>,
}

/// Opaque pointer to MTLDevice (e.g. from `metal::Device`). Use when creating Metal server/client.
pub type MTLDevicePtr = *mut std::ffi::c_void;

/// Opaque pointer to MTLTexture. Use when publishing a frame or after receiving one from the client.
pub type MTLTexturePtr = *mut std::ffi::c_void;

/// Opaque pointer to MTLCommandBuffer. Use when publishing a frame on the Metal server.
pub type MTLCommandBufferPtr = *mut std::ffi::c_void;

/// Metal Syphon server: publishes frames from Metal textures.
pub struct MetalServer {
    #[cfg(target_os = "macos")]
    ptr: NonNull<std::ffi::c_void>,
}

/// Metal Syphon client: receives frames as MTLTextures.
pub struct MetalClient {
    #[cfg(target_os = "macos")]
    ptr: NonNull<std::ffi::c_void>,
    #[cfg(target_os = "macos")]
    _callback_storage: Option<Box<CallbackHolder>>,
}

/// A Metal texture from Syphon (server or client). Release when done drawing.
pub struct MetalTexture {
    #[cfg(target_os = "macos")]
    ptr: NonNull<std::ffi::c_void>,
}

#[cfg(target_os = "macos")]
fn opt_cstr_to_string(s: *mut c_char) -> Option<String> {
    if s.is_null() {
        return None;
    }
    let cstr = unsafe { CStr::from_ptr(s) };
    let out = cstr.to_string_lossy().into_owned();
    unsafe { libc::free(s as *mut _) };
    Some(out)
}

impl ServerDirectory {
    /// Returns the shared server directory, or `None` on failure.
    pub fn shared() -> Option<Self> {
        #[cfg(target_os = "macos")]
        {
            let ptr = unsafe { ffi::syphon_server_directory_shared() };
            NonNull::new(ptr).map(|ptr| Self { ptr })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    /// Number of servers currently in the directory.
    pub fn servers_count(&self) -> usize {
        #[cfg(target_os = "macos")]
        {
            unsafe { ffi::syphon_server_directory_servers_count(self.ptr.as_ptr()) }
        }
        #[cfg(not(target_os = "macos"))]
        0
    }

    /// Server description at index (not retained; valid only while directory is not refreshed).
    pub fn server_at_index(&self, index: usize) -> Option<ServerDescription> {
        #[cfg(target_os = "macos")]
        {
            let ptr = unsafe { ffi::syphon_server_directory_server_at_index(self.ptr.as_ptr(), index) };
            NonNull::new(ptr).map(|ptr| ServerDescription { ptr, owned: false })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    /// All current server descriptions. Descriptions are not retained; valid only until the next directory update.
    pub fn servers(&self) -> Vec<ServerDescription> {
        let n = self.servers_count();
        (0..n)
            .filter_map(|i| self.server_at_index(i))
            .collect()
    }
}

impl ServerDescription {
    /// Copy the server UUID (unique id), if present.
    pub fn uuid(&self) -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            let s = unsafe { ffi::syphon_server_description_copy_uuid(self.ptr.as_ptr()) };
            opt_cstr_to_string(s)
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    /// Copy the server name (human-readable), if present.
    pub fn name(&self) -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            let s = unsafe { ffi::syphon_server_description_copy_name(self.ptr.as_ptr()) };
            opt_cstr_to_string(s)
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    /// Copy the application name hosting the server, if present.
    pub fn app_name(&self) -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            let s = unsafe { ffi::syphon_server_description_copy_app_name(self.ptr.as_ptr()) };
            opt_cstr_to_string(s)
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    /// Retain the description so it remains valid after the directory updates. Call `release` or drop a retained clone when done.
    pub fn retain(&self) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_server_description_retain(self.ptr.as_ptr());
        }
    }

    /// Release a description that was retained.
    pub fn release(&self) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_server_description_release(self.ptr.as_ptr());
        }
    }
}

impl Clone for ServerDescription {
    fn clone(&self) -> Self {
        self.retain();
        Self {
            #[cfg(target_os = "macos")]
            ptr: self.ptr,
            #[cfg(target_os = "macos")]
            owned: true,
        }
    }
}

impl Drop for ServerDescription {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        if self.owned {
            unsafe { ffi::syphon_server_description_release(self.ptr.as_ptr()) };
        }
    }
}

impl OpenGLServer {
    /// Create a new OpenGL server. `name` can be None (empty). `options` can be None.
    /// Returns None if creation failed.
    pub fn new(
        name: Option<&str>,
        context: CGLContextObj,
        _options: Option<&std::collections::HashMap<String, String>>,
    ) -> Option<Self> {
        #[cfg(target_os = "macos")]
        {
            let name_ptr = name
                .map(|s| std::ffi::CString::new(s).ok())
                .flatten()
                .as_ref()
                .map(|c| c.as_ptr())
                .unwrap_or(std::ptr::null());
            let ptr = unsafe {
                ffi::syphon_opengl_server_create(name_ptr, context, std::ptr::null_mut())
            };
            NonNull::new(ptr).map(|ptr| Self { ptr })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    /// True if any clients are attached.
    pub fn has_clients(&self) -> bool {
        #[cfg(target_os = "macos")]
        unsafe { ffi::syphon_opengl_server_has_clients(self.ptr.as_ptr()) }
        #[cfg(not(target_os = "macos"))]
        false
    }

    /// Server description (retained; caller owns and should release via ServerDescription).
    pub fn server_description(&self) -> Option<ServerDescription> {
        #[cfg(target_os = "macos")]
        {
            let ptr = unsafe { ffi::syphon_opengl_server_server_description(self.ptr.as_ptr()) };
            NonNull::new(ptr).map(|ptr| ServerDescription { ptr, owned: true })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    /// Publish a frame from a texture. Region (x,y,w,h) and texture size (tex_w, tex_h), flipped.
    pub fn publish_frame(
        &self,
        tex_id: u32,
        target: u32,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        tex_w: f64,
        tex_h: f64,
        flipped: bool,
    ) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_opengl_server_publish_frame(
                self.ptr.as_ptr(),
                tex_id,
                target,
                x,
                y,
                w,
                h,
                tex_w,
                tex_h,
                flipped,
            );
        }
    }

    /// Bind the server's FBO to draw a frame of the given size. Pair with `unbind_and_publish`.
    pub fn bind_to_draw_frame(&self, w: f64, h: f64) -> bool {
        #[cfg(target_os = "macos")]
        unsafe { ffi::syphon_opengl_server_bind_to_draw_frame(self.ptr.as_ptr(), w, h) }
        #[cfg(not(target_os = "macos"))]
        false
    }

    /// Unbind and publish the just-drawn frame.
    pub fn unbind_and_publish(&self) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_opengl_server_unbind_and_publish(self.ptr.as_ptr());
        }
    }

    /// Stop the server.
    pub fn stop(&self) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_opengl_server_stop(self.ptr.as_ptr());
        }
    }
}

impl Drop for OpenGLServer {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.stop();
            unsafe {
                ffi::syphon_opengl_server_release(self.ptr.as_ptr());
            }
        }
    }
}

/// Callback for new frames: invoked when a new frame is available (may be on another thread).
pub type NewFrameCallback = Box<dyn Fn() + Send>;

/// Holds the closure so we can pass a single pointer to C and invoke it from the callback.
#[cfg(target_os = "macos")]
struct CallbackHolder(Box<dyn Fn() + Send>);

impl OpenGLClient {
    /// Create a client for the given server description and context. `callback` can be None (no handler).
    /// The callback may be invoked on a different thread. When provided, it is kept for the client's lifetime.
    pub fn new(
        description: &ServerDescription,
        context: CGLContextObj,
        _options: Option<&std::collections::HashMap<String, String>>,
        callback: Option<NewFrameCallback>,
    ) -> Option<Self> {
        #[cfg(target_os = "macos")]
        {
            unsafe extern "C" fn raw_callback(userdata: *mut std::ffi::c_void) {
                if userdata.is_null() {
                    return;
                }
                let h = &*(userdata as *const CallbackHolder);
                (h.0)();
            }
            let callback_storage: Option<Box<CallbackHolder>> =
                callback.map(|c| Box::new(CallbackHolder(c)));
            let userdata = callback_storage
                .as_ref()
                .map(|b| (&**b) as *const CallbackHolder as *mut std::ffi::c_void)
                .unwrap_or(std::ptr::null_mut());
            type Cb = unsafe extern "C" fn(*mut std::ffi::c_void);
            let cb_opt: Option<Cb> = callback_storage.as_ref().map(|_| raw_callback as Cb);
            let ptr = unsafe {
                ffi::syphon_opengl_client_create(
                    description.ptr.as_ptr(),
                    context,
                    std::ptr::null_mut(),
                    cb_opt,
                    userdata,
                )
            };
            NonNull::new(ptr).map(|ptr| Self {
                ptr,
                _callback_storage: callback_storage,
            })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    pub fn is_valid(&self) -> bool {
        #[cfg(target_os = "macos")]
        unsafe { ffi::syphon_opengl_client_is_valid(self.ptr.as_ptr()) }
        #[cfg(not(target_os = "macos"))]
        false
    }

    pub fn has_new_frame(&self) -> bool {
        #[cfg(target_os = "macos")]
        unsafe { ffi::syphon_opengl_client_has_new_frame(self.ptr.as_ptr()) }
        #[cfg(not(target_os = "macos"))]
        false
    }

    /// Get the current frame image. Caller must drop the image when done drawing.
    pub fn new_frame_image(&self) -> Option<OpenGLImage> {
        #[cfg(target_os = "macos")]
        {
            let ptr = unsafe { ffi::syphon_opengl_client_new_frame_image(self.ptr.as_ptr()) };
            NonNull::new(ptr).map(|ptr| OpenGLImage { ptr })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    pub fn stop(&self) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_opengl_client_stop(self.ptr.as_ptr());
        }
    }
}

impl Drop for OpenGLClient {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.stop();
            unsafe {
                ffi::syphon_opengl_client_release(self.ptr.as_ptr());
            }
        }
    }
}

impl OpenGLImage {
    pub fn texture_name(&self) -> u32 {
        #[cfg(target_os = "macos")]
        unsafe { ffi::syphon_opengl_image_texture_name(self.ptr.as_ptr()) }
        #[cfg(not(target_os = "macos"))]
        0
    }

    pub fn texture_size(&self) -> (f64, f64) {
        #[cfg(target_os = "macos")]
        {
            let mut w = 0.0;
            let mut h = 0.0;
            unsafe {
                ffi::syphon_opengl_image_texture_size(self.ptr.as_ptr(), &mut w, &mut h);
            }
            (w, h)
        }
        #[cfg(not(target_os = "macos"))]
        (0.0, 0.0)
    }
}

impl Drop for OpenGLImage {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_opengl_image_release(self.ptr.as_ptr());
        }
    }
}

// ---------------------------------------------------------------------------
// Metal server
// ---------------------------------------------------------------------------

impl MetalServer {
    /// Create a Metal server. `name` can be None. `options` can be None.
    /// `device` must be a valid MTLDevice pointer (e.g. from the `metal` crate).
    pub fn new(
        name: Option<&str>,
        device: MTLDevicePtr,
        _options: Option<&std::collections::HashMap<String, String>>,
    ) -> Option<Self> {
        #[cfg(target_os = "macos")]
        {
            if device.is_null() {
                return None;
            }
            let name_ptr = name
                .map(|s| std::ffi::CString::new(s).ok())
                .flatten()
                .as_ref()
                .map(|c| c.as_ptr())
                .unwrap_or(std::ptr::null());
            let ptr =
                unsafe { ffi::syphon_metal_server_create(name_ptr, device as *mut _, std::ptr::null_mut()) };
            NonNull::new(ptr).map(|ptr| Self { ptr })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    pub fn has_clients(&self) -> bool {
        #[cfg(target_os = "macos")]
        unsafe { ffi::syphon_metal_server_has_clients(self.ptr.as_ptr()) }
        #[cfg(not(target_os = "macos"))]
        false
    }

    /// Server description (retained; caller owns).
    pub fn server_description(&self) -> Option<ServerDescription> {
        #[cfg(target_os = "macos")]
        {
            let ptr = unsafe { ffi::syphon_metal_server_server_description(self.ptr.as_ptr()) };
            NonNull::new(ptr).map(|ptr| ServerDescription { ptr, owned: true })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    /// Publish a frame from a Metal texture. Region (x, y, w, h). You must commit `command_buffer`.
    pub fn publish_frame(
        &self,
        texture: MTLTexturePtr,
        command_buffer: MTLCommandBufferPtr,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        flipped: bool,
    ) {
        #[cfg(target_os = "macos")]
        if !texture.is_null() && !command_buffer.is_null() {
            unsafe {
                ffi::syphon_metal_server_publish_frame(
                    self.ptr.as_ptr(),
                    texture as *mut _,
                    command_buffer as *mut _,
                    x,
                    y,
                    w,
                    h,
                    flipped,
                );
            }
        }
    }

    /// Current frame as MTLTexture (caller must release via MetalTexture or syphon_metal_texture_release).
    pub fn new_frame_image(&self) -> Option<MetalTexture> {
        #[cfg(target_os = "macos")]
        {
            let ptr = unsafe { ffi::syphon_metal_server_new_frame_image(self.ptr.as_ptr()) };
            NonNull::new(ptr).map(|ptr| MetalTexture { ptr })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    pub fn stop(&self) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_metal_server_stop(self.ptr.as_ptr());
        }
    }
}

impl Drop for MetalServer {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.stop();
            unsafe {
                ffi::syphon_metal_server_release(self.ptr.as_ptr());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Metal client
// ---------------------------------------------------------------------------

impl MetalClient {
    /// Create a Metal client. `device` must be a valid MTLDevice pointer. `callback` can be None.
    pub fn new(
        description: &ServerDescription,
        device: MTLDevicePtr,
        _options: Option<&std::collections::HashMap<String, String>>,
        callback: Option<NewFrameCallback>,
    ) -> Option<Self> {
        #[cfg(target_os = "macos")]
        {
            if device.is_null() {
                return None;
            }
            unsafe extern "C" fn raw_callback(userdata: *mut std::ffi::c_void) {
                if userdata.is_null() {
                    return;
                }
                let h = &*(userdata as *const CallbackHolder);
                (h.0)();
            }
            let callback_storage: Option<Box<CallbackHolder>> =
                callback.map(|c| Box::new(CallbackHolder(c)));
            let userdata = callback_storage
                .as_ref()
                .map(|b| (&**b) as *const CallbackHolder as *mut std::ffi::c_void)
                .unwrap_or(std::ptr::null_mut());
            type Cb = unsafe extern "C" fn(*mut std::ffi::c_void);
            let cb_opt: Option<Cb> = callback_storage.as_ref().map(|_| raw_callback as Cb);
            let ptr = unsafe {
                ffi::syphon_metal_client_create(
                    description.ptr.as_ptr(),
                    device as *mut _,
                    std::ptr::null_mut(),
                    cb_opt,
                    userdata,
                )
            };
            NonNull::new(ptr).map(|ptr| Self {
                ptr,
                _callback_storage: callback_storage,
            })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    pub fn is_valid(&self) -> bool {
        #[cfg(target_os = "macos")]
        unsafe { ffi::syphon_metal_client_is_valid(self.ptr.as_ptr()) }
        #[cfg(not(target_os = "macos"))]
        false
    }

    pub fn has_new_frame(&self) -> bool {
        #[cfg(target_os = "macos")]
        unsafe { ffi::syphon_metal_client_has_new_frame(self.ptr.as_ptr()) }
        #[cfg(not(target_os = "macos"))]
        false
    }

    /// Get the current frame as MTLTexture. Caller must drop the returned value when done.
    pub fn new_frame_image(&self) -> Option<MetalTexture> {
        #[cfg(target_os = "macos")]
        {
            let ptr = unsafe { ffi::syphon_metal_client_new_frame_image(self.ptr.as_ptr()) };
            NonNull::new(ptr).map(|ptr| MetalTexture { ptr })
        }
        #[cfg(not(target_os = "macos"))]
        None
    }

    pub fn stop(&self) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_metal_client_stop(self.ptr.as_ptr());
        }
    }
}

impl Drop for MetalClient {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.stop();
            unsafe {
                ffi::syphon_metal_client_release(self.ptr.as_ptr());
            }
        }
    }
}

impl MetalTexture {
    /// Raw MTLTexture pointer for use with the `metal` crate or other Metal code.
    pub fn as_ptr(&self) -> MTLTexturePtr {
        #[cfg(target_os = "macos")]
        {
            self.ptr.as_ptr() as MTLTexturePtr
        }
        #[cfg(not(target_os = "macos"))]
        std::ptr::null_mut()
    }
}

impl Drop for MetalTexture {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        unsafe {
            ffi::syphon_metal_texture_release(self.ptr.as_ptr());
        }
    }
}
