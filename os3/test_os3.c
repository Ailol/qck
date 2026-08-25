#include "os3.h"

#include <assert.h>
#include <stdio.h>

int main(void) {
    os3_world_t world;
    assert(os3_world_create(&world) == BM_OK);

    const os3_object_t objects[] = {
        {1, 0, 0, 0, 800, 600, 0},
        {2, 1, 20, 20, 500, 300, 1},
        {3, 1, 40, 40, 220, 120, 2},
    };

    os3_set(&world, OS3_RUNNING, 0, 1);
    os3_set(&world, OS3_VISIBLE, 0, 1);

    os3_set(&world, OS3_RUNNING, 1, 1);
    os3_set(&world, OS3_VISIBLE, 1, 1);

    os3_set(&world, OS3_RUNNING, 2, 1);
    os3_set(&world, OS3_VISIBLE, 2, 1);
    os3_set(&world, OS3_FOCUSED, 2, 1);
    os3_set(&world, OS3_CHANGED, 2, 1);

    assert(os3_has(&world, OS3_CHANGED, 2));

    nit_box_t boxes[3];
    size_t n = os3_project(&world, objects, 3, boxes, 3);
    assert(n == 3);
    assert(nit_depth(boxes, n, 3) == 1);
    assert(nit_hit(boxes, n, 50, 50) == 3);
    assert((boxes[2].flags & NIT_SELECTED) != 0);

    os3_set(&world, OS3_VISIBLE, 2, 0);
    n = os3_project(&world, objects, 3, boxes, 3);
    assert(n == 2);

    puts("os3: 6/6 passed; host executes, BitMemory carries state, nit projects it.");

    os3_world_close(&world);
    return 0;
}
