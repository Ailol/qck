# os3

`os3/` is the smallest executable proof of the OS3 idea inside qck.

The host OS still executes the program. BitMemory carries machine/project state. `qck.nit` projects that state into navigable geometry.

```text
host process
    ↕
BitMemory Ω
    ↓
bitmap planes
    ↓
os3_project()
    ↓
qck.nit boxes / depth
```

No instruction VM is introduced.

## v0 planes

```text
running
visible
changed
focused
allowed
```

The planes share one existing `bm_mem_t` and one object index. `os3_project()` emits only objects that are both `running & visible`; focused state becomes `NIT_SELECTED`.

## Build the proof

From the repository root:

```sh
cc -std=c11 -D_POSIX_C_SOURCE=200112L -Wall -Wextra \
  os3/os3.c os3/test_os3.c \
  infra/distributed/bitmemory/bitmemory.c \
  qck.nit/nit.c \
  -o os3/test_os3

./os3/test_os3
```

Expected:

```text
os3: 6/6 passed; host executes, BitMemory carries state, nit projects it.
```

## Boundary

```text
BitMemory = remembered state
host OS   = execution
nit'      = projection
os3       = machine-shaped view over the same state
```

This proof deliberately does not add a loader, kernel, scheduler, service bus, or localhost API.
