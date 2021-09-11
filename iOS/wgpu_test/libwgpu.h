//
//  libwgpu_on_ios.h
//  wgpu_test
//
//  Created by LiJinlei on 2021/9/10.
//

#ifndef libwgpu_on_ios_h
#define libwgpu_on_ios_h

#include <stdint.h>

struct wgpu_canvas;

struct ios_obj {
    void *view;
    // CAMetalLayer
    void *metal_layer;
    int maximum_frames;
    const char *temporary_directory;
    void (*callback_to_swift)(int32_t arg);
};

struct wgpu_canvas *create_wgpu_canvas(struct ios_obj object);
void enter_frame(struct wgpu_canvas *data);

#endif /* libwgpu_on_ios_h */
