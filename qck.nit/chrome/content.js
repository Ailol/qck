(() => {
  if (globalThis.__qckNitInstalled) return;
  globalThis.__qckNitInstalled = true;

  const ids = new WeakMap();
  let nextId = 1;
  let enabled = false;
  let locked = false;
  let lastPoint = { x: innerWidth / 2, y: innerHeight / 2 };
  let host = null;
  let shadow = null;
  let layer = null;
  let hud = null;

  function idOf(el) {
    let id = ids.get(el);
    if (!id) {
      id = nextId++;
      ids.set(el, id);
    }
    return id;
  }

  function rectOf(el) {
    const r = el.getBoundingClientRect();
    return { x: r.x, y: r.y, w: r.width, h: r.height };
  }

  function visible(el) {
    if (!(el instanceof Element)) return false;
    const r = el.getBoundingClientRect();
    if (r.width <= 0 || r.height <= 0) return false;
    const s = getComputedStyle(el);
    return s.display !== "none" && s.visibility !== "hidden" && Number(s.opacity || 1) !== 0;
  }

  function parentElement(el) {
    const root = el.getRootNode();
    if (el.parentElement) return el.parentElement;
    if (root instanceof ShadowRoot) return root.host;
    return null;
  }

  function chainOf(el) {
    const chain = [];
    let node = el;
    while (node instanceof Element) {
      if (visible(node)) chain.push(node);
      node = parentElement(node);
    }
    return chain.reverse();
  }

  function selectorHint(el) {
    if (!el) return "";
    if (el.id) return `${el.tagName.toLowerCase()}#${el.id}`;
    const cls = [...el.classList].slice(0, 2).join(".");
    return cls ? `${el.tagName.toLowerCase()}.${cls}` : el.tagName.toLowerCase();
  }

  function ensureUi() {
    if (host) return;

    host = document.createElement("div");
    host.dataset.qckNit = "overlay";
    host.style.cssText = "position:fixed;inset:0;z-index:2147483647;pointer-events:none;contain:strict;";
    shadow = host.attachShadow({ mode: "closed" });
    shadow.innerHTML = `
      <style>
        :host { all: initial; }
        #layer { position:fixed; inset:0; pointer-events:none; overflow:hidden; }
        .box { position:fixed; box-sizing:border-box; border:1px solid rgba(0,220,255,.78); background:rgba(0,220,255,.035); }
        .box::after { content:attr(data-label); position:absolute; left:0; top:-18px; max-width:320px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; padding:1px 5px; border-radius:4px; background:rgba(8,12,18,.88); color:#dffbff; font:11px/15px ui-monospace,SFMono-Regular,Consolas,monospace; }
        .leaf { border-width:2px; background:rgba(255,210,60,.07); border-color:rgba(255,210,60,.95); }
        .leaf::after { background:rgba(55,42,0,.92); color:#fff4b0; }
        #hud { position:fixed; right:12px; top:12px; min-width:260px; max-width:420px; padding:9px 11px; border:1px solid rgba(255,255,255,.16); border-radius:8px; background:rgba(7,10,14,.92); color:#eef7ff; box-shadow:0 8px 34px rgba(0,0,0,.28); font:12px/1.45 ui-monospace,SFMono-Regular,Consolas,monospace; }
        #hud b { color:#7ee9ff; font-weight:600; }
        #hud .muted { opacity:.68; }
      </style>
      <div id="layer"></div>
      <div id="hud"></div>`;

    layer = shadow.querySelector("#layer");
    hud = shadow.querySelector("#hud");
    document.documentElement.appendChild(host);
  }

  function clearUi() {
    if (layer) layer.replaceChildren();
  }

  function draw(point = lastPoint) {
    if (!enabled || locked) return;
    lastPoint = point;
    ensureUi();

    const stack = document.elementsFromPoint(point.x, point.y)
      .filter((el) => el !== host && visible(el));
    const leaf = stack[0];

    clearUi();
    if (!leaf) {
      hud.textContent = "qck.nit — no visible box";
      return;
    }

    const chain = chainOf(leaf);
    const capped = chain.slice(-10);

    capped.forEach((el, index) => {
      const r = rectOf(el);
      const depth = chain.indexOf(el);
      const box = document.createElement("div");
      box.className = `box${el === leaf ? " leaf" : ""}`;
      box.style.left = `${r.x}px`;
      box.style.top = `${r.y}px`;
      box.style.width = `${r.w}px`;
      box.style.height = `${r.h}px`;
      box.style.transform = `translate(${index * 1.5}px, ${index * 1.5}px)`;
      box.dataset.label = `${idOf(el)} · d${depth} · ${selectorHint(el)}`;
      layer.appendChild(box);
    });

    const r = rectOf(leaf);
    const hitIds = stack.slice(0, 64).map(idOf);
    let bitmap = 0n;
    stack.slice(0, 64).forEach((_, i) => { bitmap |= 1n << BigInt(i); });

    hud.innerHTML = `
      <b>qck.nit</b> ${locked ? "locked" : "live"}<br>
      id=${idOf(leaf)} depth=${chain.length - 1} ${selectorHint(leaf)}<br>
      box=[${r.x.toFixed(1)}, ${r.y.toFixed(1)}, ${r.w.toFixed(1)}, ${r.h.toFixed(1)}]<br>
      hit=${stack.length} bitmap=0x${bitmap.toString(16)}<br>
      <span class="muted">Alt+click lock · extension icon toggle</span>`;

    console.debug("qck.nit", {
      leaf: { id: idOf(leaf), depth: chain.length - 1, box: r, selector: selectorHint(leaf) },
      hitIds,
      bitmap: `0x${bitmap.toString(16)}`
    });
  }

  function onMove(event) {
    if (!enabled || locked) return;
    requestAnimationFrame(() => draw({ x: event.clientX, y: event.clientY }));
  }

  function onAltClick(event) {
    if (!enabled || !event.altKey) return;
    event.preventDefault();
    event.stopPropagation();
    lastPoint = { x: event.clientX, y: event.clientY };
    locked = !locked;

    if (locked) {
      const wasLocked = locked;
      locked = false;
      draw(lastPoint);
      locked = wasLocked;
      if (hud) hud.innerHTML = hud.innerHTML.replace("live", "locked");
    } else {
      draw(lastPoint);
    }
  }

  function toggle() {
    enabled = !enabled;
    locked = false;

    if (!enabled) {
      host?.remove();
      host = shadow = layer = hud = null;
      return;
    }

    ensureUi();
    draw(lastPoint);
  }

  addEventListener("mousemove", onMove, { passive: true, capture: true });
  addEventListener("click", onAltClick, true);
  addEventListener("scroll", () => { if (enabled && !locked) draw(lastPoint); }, { passive: true, capture: true });
  addEventListener("resize", () => { if (enabled && !locked) draw(lastPoint); }, { passive: true });

  chrome.runtime.onMessage.addListener((message) => {
    if (message?.type === "qck.nit.toggle") toggle();
  });
})();
