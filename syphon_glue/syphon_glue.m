/*
 * syphon_glue.m - Objective-C implementation of the Syphon C glue API.
 */
#ifdef __APPLE__

#define GL_SILENCE_DEPRECATION 1
/* GL_TEXTURE_RECTANGLE not in macOS gl.h; Syphon uses rectangle textures. */
#ifndef GL_TEXTURE_RECTANGLE
#define GL_TEXTURE_RECTANGLE 0x84F5
#endif

#import <Foundation/Foundation.h>
#import <CoreFoundation/CoreFoundation.h>
#import <OpenGL/OpenGL.h>
#import <OpenGL/gl.h>
#import <Metal/Metal.h>
#import <Syphon/Syphon.h>

static NSString *nullable_cstring_to_nsstring(const char *cstr) {
    if (!cstr) return nil;
    return [NSString stringWithUTF8String:cstr];
}

/* Server directory */
void *syphon_server_directory_shared(void) {
    return (__bridge void *)[SyphonServerDirectory sharedDirectory];
}

size_t syphon_server_directory_servers_count(void *dir) {
    SyphonServerDirectory *d = (__bridge SyphonServerDirectory *)dir;
    return (size_t)[d.servers count];
}

void *syphon_server_directory_server_at_index(void *dir, size_t index) {
    SyphonServerDirectory *d = (__bridge SyphonServerDirectory *)dir;
    NSArray *servers = d.servers;
    if (index >= [servers count]) return NULL;
    return (__bridge void *)[servers objectAtIndex:index];
}

static char *copy_nsstring_to_cstring(NSString *s) {
    if (!s) return NULL;
    const char *utf8 = [s UTF8String];
    if (!utf8) return NULL;
    return strdup(utf8);
}

char *syphon_server_description_copy_uuid(void *desc) {
    NSDictionary *d = (__bridge NSDictionary *)desc;
    NSString *v = d[SyphonServerDescriptionUUIDKey];
    return copy_nsstring_to_cstring(v);
}

char *syphon_server_description_copy_name(void *desc) {
    NSDictionary *d = (__bridge NSDictionary *)desc;
    NSString *v = d[SyphonServerDescriptionNameKey];
    return copy_nsstring_to_cstring(v);
}

char *syphon_server_description_copy_app_name(void *desc) {
    NSDictionary *d = (__bridge NSDictionary *)desc;
    NSString *v = d[SyphonServerDescriptionAppNameKey];
    return copy_nsstring_to_cstring(v);
}

void syphon_server_description_retain(void *desc) {
    (void)CFBridgingRetain((__bridge id)desc);
}

void syphon_server_description_release(void *desc) {
    if (desc) CFRelease((CFTypeRef)desc);
}

/* OpenGL server */
void *syphon_opengl_server_create(const char *name, CGLContextObj context, void *options) {
    NSString *nsName = nullable_cstring_to_nsstring(name);
    SyphonOpenGLServer *server = [[SyphonOpenGLServer alloc] initWithName:nsName
                                                                  context:context
                                                                  options:(__bridge NSDictionary *)options];
    return (__bridge_retained void *)server;
}

void syphon_opengl_server_release(void *server) {
    (void)(__bridge_transfer SyphonOpenGLServer *)server;
}

bool syphon_opengl_server_has_clients(void *server) {
    SyphonOpenGLServer *s = (__bridge SyphonOpenGLServer *)server;
    return s.hasClients ? true : false;
}

void *syphon_opengl_server_server_description(void *server) {
    SyphonOpenGLServer *s = (__bridge SyphonOpenGLServer *)server;
    NSDictionary *desc = s.serverDescription;
    return (__bridge_retained void *)desc;
}

void syphon_opengl_server_publish_frame(void *server, GLuint tex_id, GLenum target,
    double x, double y, double w, double h, double tex_w, double tex_h, bool flipped) {
    SyphonOpenGLServer *s = (__bridge SyphonOpenGLServer *)server;
    NSRect region = NSMakeRect(x, y, w, h);
    NSSize size = NSMakeSize(tex_w, tex_h);
    [s publishFrameTexture:tex_id textureTarget:target imageRegion:region
        textureDimensions:size flipped:flipped ? YES : NO];
}

bool syphon_opengl_server_bind_to_draw_frame(void *server, double w, double h) {
    SyphonOpenGLServer *s = (__bridge SyphonOpenGLServer *)server;
    NSSize size = NSMakeSize(w, h);
    return [s bindToDrawFrameOfSize:size] ? true : false;
}

void syphon_opengl_server_unbind_and_publish(void *server) {
    SyphonOpenGLServer *s = (__bridge SyphonOpenGLServer *)server;
    [s unbindAndPublish];
}

void syphon_opengl_server_stop(void *server) {
    SyphonOpenGLServer *s = (__bridge SyphonOpenGLServer *)server;
    [s stop];
}

/* OpenGL client */
typedef void (*new_frame_callback_t)(void *userdata);

void *syphon_opengl_client_create(void *server_description, CGLContextObj context,
    void *options, new_frame_callback_t new_frame_callback, void *userdata) {
    NSDictionary *desc = (__bridge NSDictionary *)server_description;
    void (^handler)(SyphonOpenGLClient *);
    if (new_frame_callback) {
        new_frame_callback_t cb = new_frame_callback;
        void *ud = userdata;
        handler = ^(SyphonOpenGLClient *client) {
            (void)client;
            cb(ud);
        };
    } else {
        handler = nil;
    }
    SyphonOpenGLClient *client = [[SyphonOpenGLClient alloc] initWithServerDescription:desc
                                                                             context:context
                                                                             options:(__bridge NSDictionary *)options
                                                                       newFrameHandler:handler];
    return (__bridge_retained void *)client;
}

void syphon_opengl_client_release(void *client) {
    (void)(__bridge_transfer SyphonOpenGLClient *)client;
}

bool syphon_opengl_client_is_valid(void *client) {
    SyphonOpenGLClient *c = (__bridge SyphonOpenGLClient *)client;
    return c.isValid ? true : false;
}

bool syphon_opengl_client_has_new_frame(void *client) {
    SyphonOpenGLClient *c = (__bridge SyphonOpenGLClient *)client;
    return c.hasNewFrame ? true : false;
}

void *syphon_opengl_client_new_frame_image(void *client) {
    SyphonOpenGLClient *c = (__bridge SyphonOpenGLClient *)client;
    SyphonOpenGLImage *img = [c newFrameImage];
    return (__bridge_retained void *)img;
}

void syphon_opengl_client_stop(void *client) {
    SyphonOpenGLClient *c = (__bridge SyphonOpenGLClient *)client;
    [c stop];
}

/* OpenGL image */
void syphon_opengl_image_release(void *image) {
    (void)(__bridge_transfer SyphonOpenGLImage *)image;
}

GLuint syphon_opengl_image_texture_name(void *image) {
    SyphonOpenGLImage *img = (__bridge SyphonOpenGLImage *)image;
    return img.textureName;
}

void syphon_opengl_image_texture_size(void *image, double *out_w, double *out_h) {
    SyphonOpenGLImage *img = (__bridge SyphonOpenGLImage *)image;
    NSSize size = img.textureSize;
    if (out_w) *out_w = size.width;
    if (out_h) *out_h = size.height;
}

/* Metal server */
void *syphon_metal_server_create(const char *name, void *device, void *options) {
    NSString *nsName = nullable_cstring_to_nsstring(name);
    id<MTLDevice> mtlDevice = (__bridge id<MTLDevice>)device;
    SyphonMetalServer *server = [[SyphonMetalServer alloc] initWithName:nsName
                                                                  device:mtlDevice
                                                                 options:(__bridge NSDictionary *)options];
    return (__bridge_retained void *)server;
}

void syphon_metal_server_release(void *server) {
    (void)(__bridge_transfer SyphonMetalServer *)server;
}

bool syphon_metal_server_has_clients(void *server) {
    SyphonMetalServer *s = (__bridge SyphonMetalServer *)server;
    return s.hasClients ? true : false;
}

void *syphon_metal_server_server_description(void *server) {
    SyphonMetalServer *s = (__bridge SyphonMetalServer *)server;
    NSDictionary *desc = s.serverDescription;
    return (__bridge_retained void *)desc;
}

void syphon_metal_server_publish_frame(void *server, void *texture, void *command_buffer,
    double x, double y, double w, double h, bool flipped) {
    SyphonMetalServer *s = (__bridge SyphonMetalServer *)server;
    id<MTLTexture> mtlTexture = (__bridge id<MTLTexture>)texture;
    id<MTLCommandBuffer> mtlCmdBuf = (__bridge id<MTLCommandBuffer>)command_buffer;
    NSRect region = NSMakeRect(x, y, w, h);
    [s publishFrameTexture:mtlTexture onCommandBuffer:mtlCmdBuf imageRegion:region flipped:flipped ? YES : NO];
}

void *syphon_metal_server_new_frame_image(void *server) {
    SyphonMetalServer *s = (__bridge SyphonMetalServer *)server;
    id<MTLTexture> tex = [s newFrameImage];
    return (__bridge_retained void *)tex;
}

void syphon_metal_server_stop(void *server) {
    SyphonMetalServer *s = (__bridge SyphonMetalServer *)server;
    [s stop];
}

/* Metal client */
void *syphon_metal_client_create(void *server_description, void *device,
    void *options, new_frame_callback_t new_frame_callback, void *userdata) {
    NSDictionary *desc = (__bridge NSDictionary *)server_description;
    id<MTLDevice> mtlDevice = (__bridge id<MTLDevice>)device;
    void (^handler)(SyphonMetalClient *);
    if (new_frame_callback) {
        new_frame_callback_t cb = new_frame_callback;
        void *ud = userdata;
        handler = ^(SyphonMetalClient *client) {
            (void)client;
            cb(ud);
        };
    } else {
        handler = nil;
    }
    SyphonMetalClient *client = [[SyphonMetalClient alloc] initWithServerDescription:desc
                                                                             device:mtlDevice
                                                                            options:(__bridge NSDictionary *)options
                                                                    newFrameHandler:handler];
    return (__bridge_retained void *)client;
}

void syphon_metal_client_release(void *client) {
    (void)(__bridge_transfer SyphonMetalClient *)client;
}

bool syphon_metal_client_is_valid(void *client) {
    SyphonMetalClient *c = (__bridge SyphonMetalClient *)client;
    return c.isValid ? true : false;
}

bool syphon_metal_client_has_new_frame(void *client) {
    SyphonMetalClient *c = (__bridge SyphonMetalClient *)client;
    return c.hasNewFrame ? true : false;
}

void *syphon_metal_client_new_frame_image(void *client) {
    SyphonMetalClient *c = (__bridge SyphonMetalClient *)client;
    id<MTLTexture> tex = [c newFrameImage];
    return (__bridge_retained void *)tex;
}

void syphon_metal_client_stop(void *client) {
    SyphonMetalClient *c = (__bridge SyphonMetalClient *)client;
    [c stop];
}

void syphon_metal_texture_release(void *texture) {
    (void)(__bridge_transfer id)texture;
}

/* CGL headless context (for tests) */
CGLContextObj syphon_cgl_create_headless_context(void) {
    CGLPixelFormatAttribute attrs[] = {
        kCGLPFAOpenGLProfile, (CGLPixelFormatAttribute)kCGLOGLPVersion_3_2_Core,
        kCGLPFAAccelerated,
        (CGLPixelFormatAttribute)0
    };
    CGLPixelFormatObj pix = NULL;
    GLint npix = 0;
    if (CGLChoosePixelFormat(attrs, &pix, &npix) != kCGLNoError || !pix || npix == 0) {
        return NULL;
    }
    CGLContextObj ctx = NULL;
    if (CGLCreateContext(pix, NULL, &ctx) != kCGLNoError) {
        CGLDestroyPixelFormat(pix);
        return NULL;
    }
    CGLDestroyPixelFormat(pix);
    return ctx;
}

void syphon_cgl_destroy_context(CGLContextObj ctx) {
    if (ctx) {
        CGLSetCurrentContext(NULL);
        CGLDestroyContext(ctx);
    }
}

void syphon_cgl_make_current(CGLContextObj ctx) {
    CGLSetCurrentContext(ctx);
}

/* OpenGL texture helpers; CGL context must be current. GL_TEXTURE_RECTANGLE, RGBA8. */
GLuint syphon_gl_create_texture_rectangle_rgba8(size_t width, size_t height, const unsigned char *rgba) {
    GLuint tex = 0;
    glGenTextures(1, &tex);
    if (tex == 0) return 0;
    glBindTexture(GL_TEXTURE_RECTANGLE, tex);
    glTexImage2D(GL_TEXTURE_RECTANGLE, 0, GL_RGBA8, (GLsizei)width, (GLsizei)height, 0,
                 GL_RGBA, GL_UNSIGNED_BYTE, rgba ? rgba : NULL);
    glBindTexture(GL_TEXTURE_RECTANGLE, 0);
    return tex;
}

void syphon_gl_read_texture_rectangle_rgba8(GLuint tex_id, size_t width, size_t height, unsigned char *out_rgba) {
    if (!out_rgba || tex_id == 0) return;
    GLuint fbo = 0;
    glGenFramebuffers(1, &fbo);
    glBindFramebuffer(GL_FRAMEBUFFER, fbo);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_RECTANGLE, tex_id, 0);
    /* Syphon client textures are top-down in GL (top = y=0); read rows in order so out_rgba is top row first. */
    const size_t row_bytes = width * 4;
    for (size_t row = 0; row < height; row++) {
        glReadPixels(0, (GLint)row, (GLsizei)width, 1, GL_RGBA, GL_UNSIGNED_BYTE,
                     out_rgba + row * row_bytes);
    }
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_RECTANGLE, 0, 0);
    glBindFramebuffer(GL_FRAMEBUFFER, 0);
    glDeleteFramebuffers(1, &fbo);
}

void syphon_gl_delete_texture(GLuint tex_id) {
    if (tex_id != 0) {
        glDeleteTextures(1, &tex_id);
    }
}

#endif /* __APPLE__ */
