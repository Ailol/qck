# qck.inte`rp

Code and filenames use `qck.interp`. The display name keeps Ailo's ``inte`rp`` spelling. It recalls CS 1.6's `ex_interp` and the idea of rendering between timed states.

Valve's [client/server timing paper](https://www.gamedevs.org/uploads/latency-compensation-in-client-server-protocols.pdf), by Yahn W. Bernier, describes interpolating positions and angles between timestamped updates and the trade-off between smoothness and display delay. Valve's [Half-Life SDK user command](https://github.com/ValveSoftware/halflife/blob/master/common/usercmd.h) also carries client interpolation time. These are references for the naming and timing idea, not instructions to change game networking settings.

Here, the time control supplies a fraction along a known connection. `qck.interp.lerp(start, end, fraction)` returns the in-between value for a finite fraction from 0 to 1; it does not extrapolate beyond the endpoints. The background gradients similarly blend chosen colours. The implementation is a local visual interface rather than a network-packet buffer.

Small browser controls for colour, backgrounds, light and grain. A working SVG companion demonstrates the connection fabric and the project's four view levels. No packages, build step, external fonts or network services are required.

Open [qck.draw.html](qck.draw.html) for the integrated canvas editor, or [qck.interp.html](qck.interp.html) for the SVG companion. Keep the adjacent JS and CSS files together. GitHub's file page displays source rather than executing the demo.

## Controls

- **Fabric, Light, Line:** choose colours with the picker or enter CSS colours directly, including `oklch(...)` and `color(display-p3 ...)` in browsers that support them. The picker edits sRGB; the text field retains the entered CSS colour.
- **Background:** solid, aura, linear or conic. Direction changes linear and conic gradients; light controls the wash intensity.
- **Grain:** a fine visual texture in the companion. The reusable module emits the grain value for an existing renderer to consume.
- **Levels:** fabric, spatial, time and current, using the [preserved project vocabulary](spatial-levels.md).
- **Time vector:** scrub a parameter or play it along the demo's links. Playback starts only when requested and pauses below level 3.
- **Current:** enter context at level 4. In the demo, click a node to choose a current anchor.
- **Export / Import:** save or restore a versioned palette and interface state. Invalid imports leave the active state intact. Export stores playback paused.

Settings are remembered in this browser when local storage is available. Export keeps a portable copy. Node positions and the demo's selected anchor are scene data and are not included in the settings file.

## Palette

All presets are editable starting points, rather than limits on the colour space.

| Preset | Fabric | Light | Line |
|---|---|---|---|
| Fabric | `#101014` | `#656a58` | `#c8c5b8` |
| Rekon | `#08131c` | `#3f8baa` | `#b7d9e8` |
| Heist | `#170e1d` | `#8c568f` | `#dab9c8` |
| Agent | `#15161a` | `#777c93` | `#d8d7dc` |
| Spy | `#081a1d` | `#287f85` | `#a6d8cf` |
| Fun | `#1c110c` | `#bf8541` | `#f1d3a6` |
| Charm | `#1c101b` | `#ac637e` | `#e8bdce` |

## Add to an existing editor

```html
<link rel="stylesheet" href="qck.interp.css">
<div id="scene"><!-- existing editor --></div>
<aside id="palette"></aside>
<script src="qck.interp.js"></script>
<script>
const scene = document.querySelector('#scene');
const panel = qck.interp.mount(document.querySelector('#palette'), {
  target: scene,
  storageKey: 'my-editor.palette.v1'
});
</script>
```

`qck.draw.html` integrates the module with the canvas source read from the user's running `qck.draw` at `localhost:3433` on 7 November 2026. Browser-extension markup was excluded. The existing node, connect, draw, select and erase tools remain, with the palette drawer added at the lower right. Editing a palette text field does not trigger the editor's Delete/Backspace node shortcut. The added canvas view supports fabric anchors, spatial nodes, time-vector dots and current-context text. This repository copy does not replace the separate running server or recover any unsaved scene data from that server.

The module sets the target's background colour and gradient, `data-qck-level`, and these inherited CSS properties:

| Property | Value |
|---|---|
| `--qck-background` | Fabric CSS colour |
| `--qck-wash` | Light CSS colour |
| `--qck-ink` | Line CSS colour |
| `--qck-glow` | Light intensity from 0 to 1 |
| `--qck-grain` | Grain amount from 0 to 0.24 |
| `--qck-time` | Time parameter from 0 to 1 |

Use those properties in SVG/HTML styles. A canvas painter should consume the state and redraw using its own colour handling; a CSS background cannot replace a background painted inside an opaque canvas.

```js
scene.addEventListener('qck:interp', ({detail: state}) => {
  // Update the editor's painter using state.ink, state.background,
  // state.level, state.time and state.current.
});
```

The event bubbles and carries a copy of the current settings. Register the listener before mounting if the initial event is needed.

## API

```js
panel.set({background: 'oklch(18% 0.05 290)', level: 3});
panel.get();                         // copy of the current settings
const json = panel.export();         // JSON string
panel.import(json);                  // validate, apply and remember atomically
panel.destroy();                     // remove controls and stop playback

qck.interp.apply(scene, settings); // apply without mounting controls
qck.interp.normalize(settings);   // validate and fill defaults
qck.interp.lerp(10, 30, 0.25);     // 15: a quarter of the way between endpoints
```

`destroy()` leaves the target's last visual settings in place. Set `storageKey: false` to disable persistence. A custom `state` is the initial state; a valid saved state for the selected storage key takes precedence. Unknown input keys are ignored. Version 1 requires finite in-range numeric values, valid browser-supported CSS colours, a supported background mode, and a text context of at most 2,000 characters.

The demo supports pointer dragging and keyboard arrows for nodes, Shift+arrows for larger steps, and Enter/Space to select an anchor in level 4. All colour and view controls have labels. Animation is off by default; the remaining interface transitions respect reduced-motion preferences.

## Verification

Run `node test/qck.interp.test.cjs` for dependency-free state and renderer contract checks. The suite covers the exact level wording, state validation, CSS/event application, text-editing protection for node deletion, fabric anchors, time-vector placement, current-context text and script syntax. Its CSS support boundary uses colour cases separately checked in Chromium; it is not a visual browser test. The rendered layout, native import/export controls and reload persistence still require a browser check of these new files.
