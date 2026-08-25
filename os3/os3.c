#include "os3.h"

static uint64_t os3_bit(os3_plane_t plane, uint32_t object) {
    return (uint64_t)plane * OS3_MAX_OBJECTS + object;
}

bm_status_t os3_world_create(os3_world_t *world) {
    return bm_create(&world->bits, 1);
}

void os3_world_close(os3_world_t *world) {
    bm_close(&world->bits);
}

void os3_set(os3_world_t *world, os3_plane_t plane,
             uint32_t object, int value) {
    if (!world || plane >= OS3_PLANE_COUNT || object >= OS3_MAX_OBJECTS)
        return;

    if (value)
        bm_set(&world->bits, os3_bit(plane, object));
    else
        bm_clear(&world->bits, os3_bit(plane, object));
}

int os3_has(const os3_world_t *world, os3_plane_t plane,
            uint32_t object) {
    if (!world || plane >= OS3_PLANE_COUNT || object >= OS3_MAX_OBJECTS)
        return 0;

    return bm_get(&world->bits, os3_bit(plane, object));
}

size_t os3_project(const os3_world_t *world,
                   const os3_object_t *objects,
                   size_t count,
                   nit_box_t *out,
                   size_t capacity) {
    if (!world || !objects || !out)
        return 0;

    size_t n = 0;

    for (size_t i = 0;
         i < count && n < capacity && i < OS3_MAX_OBJECTS;
         ++i) {
        if (!os3_has(world, OS3_RUNNING, (uint32_t)i) ||
            !os3_has(world, OS3_VISIBLE, (uint32_t)i))
            continue;

        uint64_t flags = NIT_VISIBLE | NIT_ENABLED;
        if (os3_has(world, OS3_FOCUSED, (uint32_t)i))
            flags |= NIT_SELECTED;

        out[n++] = (nit_box_t){
            .id = objects[i].id,
            .x = objects[i].x,
            .y = objects[i].y,
            .w = objects[i].w,
            .h = objects[i].h,
            .z = objects[i].z,
            .parent = objects[i].parent,
            .flags = flags,
        };
    }

    return n;
}
