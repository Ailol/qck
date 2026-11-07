(function (global) {
  'use strict';
  const levels = Object.freeze([
    {id: 1, name: 'fabric', description: 'One fabric, including the internet.'},
    {id: 2, name: 'spatial', description: 'Placement and relationships in space.'},
    {id: 3, name: 'time', description: 'The time vector enters the view.'},
    {id: 4, name: 'current', description: 'Your mind within the proximity of self change.'}
  ]);
  const defaults = Object.freeze({version: 1, background: '#101014', wash: '#795981', ink: '#c8c5b8', mode: 'radial', angle: 135, glow: 24, grain: 8, level: 2, time: 0, current: '', playing: false});
  const presets = Object.freeze({
    Fabric: {background: '#101014', wash: '#656a58', ink: '#c8c5b8'},
    Rekon: {background: '#08131c', wash: '#3f8baa', ink: '#b7d9e8'},
    Heist: {background: '#170e1d', wash: '#8c568f', ink: '#dab9c8'},
    Agent: {background: '#15161a', wash: '#777c93', ink: '#d8d7dc'},
    Spy: {background: '#081a1d', wash: '#287f85', ink: '#a6d8cf'},
    Fun: {background: '#1c110c', wash: '#bf8541', ink: '#f1d3a6'},
    Charm: {background: '#1c101b', wash: '#ac637e', ink: '#e8bdce'}
  });
  const numeric = {angle: [0, 360], glow: [0, 100], grain: [0, 24], level: [1, 4], time: [0, 100]};
  function lerp(start, end, fraction) {
    if (![start, end, fraction].every(value => typeof value === 'number' && Number.isFinite(value)) || fraction < 0 || fraction > 1) throw new TypeError('Interpolation needs finite endpoints and a fraction from 0 to 1.');
    return start * (1 - fraction) + end * fraction;
  }
  function normalize(patch, base = defaults) {
    if (!patch || typeof patch !== 'object' || Array.isArray(patch)) throw new TypeError('Settings must be an object.');
    const state = {...base};
    for (const key of Object.keys(defaults)) {
      if (!Object.hasOwn(patch, key)) continue;
      const value = patch[key];
      if (['background', 'wash', 'ink'].includes(key)) {
        if (typeof value !== 'string' || value.length > 160 || !CSS.supports('color', value)) throw new TypeError('Choose a valid CSS colour for ' + key + '.');
      } else if (numeric[key]) {
        const [min, max] = numeric[key];
        if (typeof value !== 'number' || !Number.isFinite(value) || value < min || value > max || (key === 'level' && !Number.isInteger(value))) throw new TypeError('Invalid ' + key + ' value.');
      } else if (key === 'mode' && !['solid', 'radial', 'linear', 'conic'].includes(value)) throw new TypeError('Choose a supported background mode.');
      else if (key === 'current' && (typeof value !== 'string' || value.length > 2000)) throw new TypeError('Current context must be text up to 2000 characters.');
      else if (key === 'playing' && typeof value !== 'boolean') throw new TypeError('Playing must be true or false.');
      else if (key === 'version' && value !== 1) throw new TypeError('Unsupported settings version.');
      state[key] = value;
    }
    return state;
  }
  function apply(target, input) {
    const state = normalize(input);
    const tint = 'color-mix(in oklab, ' + state.wash + ' ' + state.glow + '%, transparent)';
    const background = {
      solid: 'none',
      radial: 'radial-gradient(ellipse at 24% 42%, ' + tint + ', transparent 72%)',
      linear: 'linear-gradient(' + state.angle + 'deg, ' + tint + ', transparent)',
      conic: 'conic-gradient(from ' + state.angle + 'deg at 45% 45%, transparent, ' + tint + ', transparent)'
    }[state.mode];
    const css = {'--qck-background': state.background, '--qck-wash': state.wash, '--qck-ink': state.ink, '--qck-grain': state.grain / 100, '--qck-glow': state.glow / 100, '--qck-time': state.time / 100};
    for (const [key, value] of Object.entries(css)) target.style.setProperty(key, value);
    target.style.backgroundColor = state.background;
    target.style.backgroundImage = background;
    target.dataset.qckLevel = String(state.level);
    target.dispatchEvent(new CustomEvent('qck:interp', {detail: {...state}, bubbles: true}));
    return state;
  }
  function mount(container, options = {}) {
    const target = options.target;
    if (!(container instanceof HTMLElement) || !(target instanceof HTMLElement)) throw new TypeError('Provide a panel container and a scene target.');
    const storageKey = options.storageKey === undefined ? 'qck.interp.v1' : options.storageKey;
    let state = normalize(options.state || {}), frame = 0, last = 0, dead = false, storageNotice = '';
    if (storageKey) {
      try {
        const saved = localStorage.getItem(storageKey);
        if (saved) state = normalize({...JSON.parse(saved), playing: false}, state);
      } catch { storageNotice = 'Saved settings could not be restored. Starting with the current palette.'; }
    }
    const form = document.createElement('form');
    form.className = 'qck-interp';
    form.setAttribute('aria-label', 'Colour and level controls');
    const colour = (key, label) => '<label class="qck-colour"><span>' + label + '</span><input type="color" data-picker="' + key + '" aria-label="' + label + ' picker"><input type="text" data-field="' + key + '" aria-label="' + label + ' CSS colour" spellcheck="false"></label>';
    const range = (key, label, max) => '<label class="qck-range"><span>' + label + '</span><output data-output="' + key + '"></output><input type="range" min="0" max="' + max + '" step="1" data-field="' + key + '" aria-label="' + label + '"></label>';
    form.innerHTML = '<div class="qck-heading"><span class="qck-mark" aria-hidden="true">✧</span><div><strong>qck.inte`rp</strong><small>Colour, space &amp; current</small></div></div><fieldset><legend>Palette</legend><div class="qck-presets">' + Object.entries(presets).map(([name, p]) => '<button type="button" data-preset="' + name + '"><i style="background:' + p.wash + '"></i>' + name + '</button>').join('') + '</div></fieldset><fieldset><legend>Background &amp; field</legend>' + colour('background', 'Fabric') + colour('wash', 'Light') + colour('ink', 'Line') + '<label class="qck-select"><span>Background</span><select data-field="mode" aria-label="Background mode"><option value="solid">Solid</option><option value="radial">Aura</option><option value="linear">Linear</option><option value="conic">Conic</option></select></label>' + range('angle', 'Direction', 360) + range('glow', 'Light', 100) + range('grain', 'Grain', 24) + '</fieldset><fieldset><legend>Levels</legend><select class="qck-level" data-field="level" aria-label="Level">' + levels.map(l => '<option value="' + l.id + '">' + l.id + 'd · ' + l.name + '</option>').join('') + '</select><p class="qck-description"></p></fieldset><fieldset class="qck-tempo"><legend>Time vector</legend><div class="qck-time"><button type="button" data-action="play" aria-label="Play time vector">▶</button><input type="range" min="0" max="100" step="0.1" data-field="time" aria-label="Time vector"><output data-output="time"></output></div></fieldset><label class="qck-current">Current<textarea data-field="current" maxlength="2000" rows="2" placeholder="Set your current context" aria-label="Current context"></textarea></label><div class="qck-actions"><button type="button" data-action="export">Export</button><label class="qck-import">Import<input type="file" accept=".json,application/json" hidden aria-label="Import settings"></label><button type="button" data-action="reset">Reset</button></div><p class="qck-status" role="status" aria-live="polite"></p>';
    container.append(form);
    const status = message => {form.querySelector('.qck-status').textContent = message;};
    const save = () => {
      if (!storageKey) return;
      try {localStorage.setItem(storageKey, JSON.stringify({...state, playing: false}));}
      catch {status('Applied. Use Export to keep a copy; browser storage is unavailable.');}
    };
    function render() {
      form.querySelectorAll('[data-field]').forEach(el => {if (document.activeElement !== el) el.value = state[el.dataset.field];});
      form.querySelectorAll('[data-picker]').forEach(el => {const value = state[el.dataset.picker]; if (/^#[0-9a-f]{6}$/i.test(value)) el.value = value; el.title = value;});
      form.querySelectorAll('[data-output]').forEach(el => {const key = el.dataset.output; el.value = key === 'angle' ? state[key] + '°' : Math.round(state[key]) + '%';});
      form.querySelector('.qck-description').textContent = levels[state.level - 1].description;
      form.querySelector('.qck-tempo').disabled = state.level < 3;
      form.querySelector('[data-field="current"]').disabled = state.level < 4;
      const play = form.querySelector('[data-action="play"]');
      play.textContent = state.playing && state.level >= 3 ? 'Ⅱ' : '▶';
      play.setAttribute('aria-label', state.playing && state.level >= 3 ? 'Pause time vector' : 'Play time vector');
      form.querySelectorAll('[data-preset]').forEach(el => el.setAttribute('aria-pressed', String(Object.entries(presets[el.dataset.preset]).every(([k,v]) => state[k] === v))));
    }
    function tick(now) {
      if (dead || !state.playing || state.level < 3) {frame = 0; last = 0; return;}
      if (last) state.time = (state.time + (now - last) / 100) % 100;
      last = now;
      apply(target, state);
      const slider = form.querySelector('[data-field="time"]');
      if (document.activeElement !== slider) slider.value = state.time;
      form.querySelector('[data-output="time"]').value = Math.round(state.time) + '%';
      frame = requestAnimationFrame(tick);
    }
    function set(patch, replace = false) {
      if (dead) throw new Error('This interface has been removed.');
      const next = normalize(patch, replace ? defaults : state);
      if (next.level < 3) next.playing = false;
      state = next;
      apply(target, state); render(); save();
      if (state.playing && !frame) {last = 0; frame = requestAnimationFrame(tick);}
      if (!state.playing && frame) {cancelAnimationFrame(frame); frame = 0; last = 0;}
      return {...state};
    }
    const exportSettings = () => JSON.stringify({...state, playing: false}, null, 2) + '\n';
    const importSettings = json => set(typeof json === 'string' ? JSON.parse(json) : json, true);
    form.addEventListener('submit', event => event.preventDefault());
    form.addEventListener('input', event => {
      const el = event.target, key = el.dataset.field || el.dataset.picker;
      if (!key) return;
      try {
        set({[key]: numeric[key] ? Number(el.value) : el.value});
        el.removeAttribute('aria-invalid'); status('');
      } catch (error) {el.setAttribute('aria-invalid', 'true'); status(error.message);}
    });
    form.addEventListener('click', event => {
      const button = event.target.closest('button');
      if (!button) return;
      if (button.dataset.preset) set(presets[button.dataset.preset]);
      if (button.dataset.action === 'play') set({playing: !state.playing});
      if (button.dataset.action === 'reset') {set(defaults, true); status('Original palette restored.');}
      if (button.dataset.action === 'export') {
        const url = URL.createObjectURL(new Blob([exportSettings()], {type: 'application/json'}));
        const link = document.createElement('a'); link.href = url; link.download = 'qck.interp.json'; link.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000); status('Settings exported.');
      }
    });
    form.querySelector('[type="file"]').addEventListener('change', async event => {
      const file = event.target.files[0]; if (!file) return;
      try {if (file.size > 20000) throw new Error('Choose a settings file smaller than 20 KB.'); importSettings(await file.text()); status('Settings restored.');}
      catch (error) {status('Import not applied. ' + error.message);}
      event.target.value = '';
    });
    apply(target, state); render(); status(storageNotice);
    if (state.playing && state.level >= 3) frame = requestAnimationFrame(tick);
    return Object.freeze({set, get: () => ({...state}), export: exportSettings, import: importSettings, destroy() {dead = true; cancelAnimationFrame(frame); form.remove();}});
  }
  global.qck = global.qck || {};
  global.qck.interp = Object.freeze({mount, apply, normalize, lerp, defaults, presets, levels});
})(globalThis);
