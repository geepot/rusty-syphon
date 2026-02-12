/*
 * syphon_glue.h - C API for Syphon (macOS only).
 * All Syphon/ObjC types are opaque pointers (void*).
 */
#ifndef SYPHON_GLUE_H
#define SYPHON_GLUE_H

#include <stddef.h>
#include <stdbool.h>

#ifdef __APPLE__
#include <OpenGL/OpenGL.h>
#endif

#ifdef __cplusplus
extern "C" {
#endif

/* Server directory */
void *syphon_server_directory_shared(void);
size_t syphon_server_directory_servers_count(void *dir);
void *syphon_server_directory_server_at_index(void *dir, size_t index);

/* Server description (NSDictionary*); do not release unless you retained */
char *syphon_server_description_copy_uuid(void *desc);
char *syphon_server_description_copy_name(void *desc);
char *syphon_server_description_copy_app_name(void *desc);
void syphon_server_description_retain(void *desc);
void syphon_server_description_release(void *desc);

/* OpenGL server */
void *syphon_opengl_server_create(const char *name, CGLContextObj context, void *options);
void syphon_opengl_server_release(void *server);
bool syphon_opengl_server_has_clients(void *server);
void *syphon_opengl_server_server_description(void *server);
void syphon_opengl_server_publish_frame(void *server, GLuint tex_id, GLenum target,
    double x, double y, double w, double h, double tex_w, double tex_h, bool flipped);
bool syphon_opengl_server_bind_to_draw_frame(void *server, double w, double h);
void syphon_opengl_server_unbind_and_publish(void *server);
void syphon_opengl_server_stop(void *server);

/* OpenGL client. new_frame_callback may be NULL. */
void *syphon_opengl_client_create(void *server_description, CGLContextObj context,
    void *options, void (*new_frame_callback)(void *userdata), void *userdata);
void syphon_opengl_client_release(void *client);
bool syphon_opengl_client_is_valid(void *client);
bool syphon_opengl_client_has_new_frame(void *client);
void *syphon_opengl_client_new_frame_image(void *client);
void syphon_opengl_client_stop(void *client);

/* OpenGL image (caller must release with syphon_opengl_image_release) */
void syphon_opengl_image_release(void *image);
GLuint syphon_opengl_image_texture_name(void *image);
void syphon_opengl_image_texture_size(void *image, double *out_w, double *out_h);

/* Metal server (device/texture/command_buffer are MTLDevice*, MTLTexture*, MTLCommandBuffer*) */
void *syphon_metal_server_create(const char *name, void *device, void *options);
void syphon_metal_server_release(void *server);
bool syphon_metal_server_has_clients(void *server);
void *syphon_metal_server_server_description(void *server);
void syphon_metal_server_publish_frame(void *server, void *texture, void *command_buffer,
    double x, double y, double w, double h, bool flipped);
void *syphon_metal_server_new_frame_image(void *server);
void syphon_metal_server_stop(void *server);

/* Metal client. new_frame_callback may be NULL. */
void *syphon_metal_client_create(void *server_description, void *device,
    void *options, void (*new_frame_callback)(void *userdata), void *userdata);
void syphon_metal_client_release(void *client);
bool syphon_metal_client_is_valid(void *client);
bool syphon_metal_client_has_new_frame(void *client);
void *syphon_metal_client_new_frame_image(void *client);
void syphon_metal_client_stop(void *client);

/* Metal texture (caller must release with syphon_metal_texture_release) */
void syphon_metal_texture_release(void *texture);

/* CGL headless context for tests (caller must destroy with syphon_cgl_destroy_context) */
CGLContextObj syphon_cgl_create_headless_context(void);
void syphon_cgl_destroy_context(CGLContextObj ctx);
void syphon_cgl_make_current(CGLContextObj ctx);

/* OpenGL texture helpers; CGL context must be current. GL_TEXTURE_RECTANGLE, RGBA8. */
GLuint syphon_gl_create_texture_rectangle_rgba8(size_t width, size_t height, const unsigned char *rgba);
void syphon_gl_read_texture_rectangle_rgba8(GLuint tex_id, size_t width, size_t height, unsigned char *out_rgba);
void syphon_gl_delete_texture(GLuint tex_id);

#ifdef __cplusplus
}
#endif

#endif /* SYPHON_GLUE_H */
