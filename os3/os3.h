#ifndef QCK_OS3_H
#define QCK_OS3_H

#include <stddef.h>
#include <stdint.h>

#include "../infra/distributed/bitmemory/bitmemory.h"
#include "../qck.nit/nit.h"

#define OS3_MAX_OBJECTS 256u

typedef enum {
    OS3_RUNNING = 0,
    OS3_VISIBLE,
    OS3_CHANGED,
    OS3_FOCUSED,
    OS3_ALLOWED,
    OS3_PLANE_COUNT
} os3_plane_t;

typedef struct {
    uint64_t id;
    uint64_t parent;
    float x;
    float y;
    float w;
    float h;
    float z;
} os3_object_t;

typedef struct {
    bm_mem_t bits;
} os3_world_t;

bm_status_t os3_world_create(os3_world_t *world);
void        os3_world_close(os3_world_t *world);
void        os3_set(os3_world_t *world, os3_plane_t plane,
                    uint32_t object, int value);
int         os3_has(const os3_world_t *world, os3_plane_t plane,
                    uint32_t object);

size_t os3_project(const os3_world_t *world,
                   const os3_object_t *objects,
                   size_t count,
                   nit_box_t *out,
                   size_t capacity);

#endif
