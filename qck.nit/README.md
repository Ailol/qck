# qck.nit

> resolve the web into depth.

`qck.nit` starts with the smallest useful physics: bounding boxes + relations.
It does not need to know React, Angular, Figma, or what a "button" means before it can orient the page.

```text
Chromium
   ↓ x,y,w,h,z,parent
qck.nit
   ↓
box IDs + hit bitmap + depth
   ↓
std10 / BitMemory later
```

## v0

Each visible thing can be represented as:

```c
typedef struct {
    uint64_t id;
    float x, y, w, h, z;
    uint64_t parent;
    uint64_t flags;
} nit_box_t;
```

The bounding box says **where** the thing is. Parent/z relations give the first structural **depth**. More pollen can attach to the same stable ID later: DOM, source, events, network, design, runtime state.

The first bitmap bridge is `nit_hit_bitmap()`: bit `i` means `boxes[i]` contains the queried point. `nit_hit()` resolves the top-most visible/enabled object.

## Chrome test extension

A zero-flag Chrome extension lives in `qck.nit/chrome/`.

1. Open `chrome://extensions`.
2. Enable **Developer mode**.
3. Choose **Load unpacked**.
4. Select the `qck.nit/chrome` folder.
5. Open any normal webpage and click the **qck.nit** extension icon.
6. Move the pointer around the page to see the current element plus its visible parent depth.
7. **Alt+click** to lock/unlock the current depth view.
8. Open DevTools Console to see the selected box, stable runtime ID, hit IDs and bitmap.

No Chrome launch flags or remote-debugging mode are required.

The Chrome adapter currently runs the same geometry idea in JavaScript so it is immediately testable. The C core remains the native contract; WASM/native BitMemory pollen can replace the geometry path later without changing the depth model.

## test

GCC/Clang:

```sh
cc -std=c11 -O2 -Wall -Wextra -pedantic nit.c test_nit.c -o test_nit
./test_nit
```

MSVC:

```bat
cl /O2 /W4 nit.c test_nit.c
test_nit.exe
```

Expected:

```text
qck.nit: 12/12 passed
```

## next

- stable DOM/source IDs attached to the same box IDs
- BitMemory-backed property/relation planes
- C core compiled to WASM for the browser test path
- optional native BitMemory runner
- optional SIMD/ASM hit-mask kernel after profiling
- depth-space visual surface in `qck.web`

Keep the public model C-simple. Assembly is an implementation accelerator, not the meaning of `qck.nit`.
