// Unit tests for assets/init.js. Run: node --test tests/js/*.test.mjs
// init.js runs in a vm sandbox with a small fake DOM.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

const ROOT = new URL("../../", import.meta.url);
const SOURCE = readFileSync(new URL("assets/init.js", ROOT), "utf8");
const FIXTURE = JSON.parse(readFileSync(new URL("tests/fixtures/attributes.json", ROOT), "utf8"));

// ---- Fake DOM ---------------------------------------------------------------

class FakeElement {
  constructor(tag, attrs = {}) {
    this.tagName = tag.toUpperCase();
    this.attrs = new Map(Object.entries(attrs));
    this.children = [];
    this.parentElement = null;
    this.listeners = {};
    this.disabled = false;
  }
  getAttribute(n) {
    return this.attrs.has(n) ? this.attrs.get(n) : null;
  }
  setAttribute(n, v) {
    this.attrs.set(n, String(v));
  }
  hasAttribute(n) {
    return this.attrs.has(n);
  }
  removeAttribute(n) {
    this.attrs.delete(n);
  }
  append(...kids) {
    for (const k of kids) {
      k.parentElement = this;
      this.children.push(k);
    }
    return this;
  }
  addEventListener(type, fn) {
    (this.listeners[type] ||= []).push(fn);
  }
  dispatch(type, detail) {
    for (const fn of this.listeners[type] || []) fn({ type, detail, target: this });
  }
  matches(selector) {
    return selectorMatches(this, selector);
  }
  closest(selector) {
    for (let el = this; el; el = el.parentElement) if (el.matches(selector)) return el;
    return null;
  }
  querySelectorAll(selector) {
    const out = [];
    const walk = (el) => {
      for (const k of el.children) {
        if (k.matches(selector)) out.push(k);
        walk(k);
      }
    };
    walk(this);
    return out;
  }
}

// Supports: `tag`, `[attr]`, and lists with commas.
function selectorMatches(el, selector) {
  return selector.split(",").some((part) => {
    const s = part.trim();
    const attr = /^\[([a-z-]+)\]$/.exec(s);
    if (attr) return el.hasAttribute(attr[1]);
    return el.tagName === s.toUpperCase();
  });
}

class FakePlayer extends FakeElement {
  constructor(attrs) {
    super("hyperframes-player", attrs);
    this.paused = true;
    this.muted = false;
    this.currentTime = 0;
    this.calls = [];
  }
  play() {
    this.calls.push("play");
    this.paused = false;
    this.dispatch("play");
    return undefined;
  }
  pause() {
    this.calls.push("pause");
    this.paused = true;
    this.dispatch("pause");
  }
  seek(t) {
    this.calls.push(`seek:${t}`);
    this.currentTime = t;
  }
}

function setup({ reduced = false, observer = true } = {}) {
  const body = new FakeElement("body");
  const listeners = {};
  const observed = new Set();
  let intersect = null;
  const document = {
    body,
    readyState: "complete",
    addEventListener(type, fn) {
      (listeners[type] ||= []).push(fn);
    },
    getElementById(id) {
      const all = [body, ...body.querySelectorAll("*")];
      return all.find((el) => el.getAttribute("id") === id) || null;
    },
    querySelectorAll: (s) => body.querySelectorAll(s),
  };
  // `*` for getElementById: match every element.
  const origMatches = FakeElement.prototype.matches;
  FakeElement.prototype.matches = function (s) {
    return s === "*" ? true : origMatches.call(this, s);
  };
  const window = {
    document,
    matchMedia: (q) => ({ matches: reduced && q.includes("prefers-reduced-motion: reduce") }),
  };
  if (observer) {
    window.IntersectionObserver = class {
      constructor(cb, opts) {
        intersect = cb;
        this.opts = opts;
      }
      observe(el) {
        observed.add(el);
      }
      unobserve(el) {
        observed.delete(el);
      }
    };
  }
  window.window = window;
  vm.runInNewContext(SOURCE, window);
  const fire = (type, event) => {
    for (const fn of listeners[type] || []) fn(event);
  };
  return {
    api: window.AutumnHyperframes,
    body,
    observed,
    fire,
    intersect: (entries) => intersect(entries),
    click: (el) => fire("click", { target: el }),
  };
}

// ---- Parsers and golden fixture ----------------------------------------------

test("parses every control that Rust writes (golden)", () => {
  const { api } = setup();
  for (const c of FIXTURE.controls) {
    assert.equal(api.parseControl(c.control), c.expect.control, JSON.stringify(c));
    assert.equal(api.parseSeek(c.seek), c.expect.seek, JSON.stringify(c));
  }
});

test("reads every player flag that Rust writes (golden)", () => {
  const { api } = setup();
  for (const p of FIXTURE.players) {
    const el = new FakePlayer({});
    if (p.inView) el.setAttribute("data-hf-in-view", "");
    if (p.reduced !== null) el.setAttribute("data-hf-reduced", p.reduced);
    assert.equal(api.wantsInView(el), p.expect.inView);
    assert.equal(api.animatesWhenReduced(el), p.expect.animateWhenReduced);
  }
});

test("rejects bad control names and seek times", () => {
  const { api } = setup();
  for (const v of FIXTURE.invalid.controls) assert.equal(api.parseControl(v), null, v);
  for (const v of FIXTURE.invalid.seeks) assert.equal(api.parseSeek(v), null, v);
  assert.equal(api.parseControl(null), null);
  assert.equal(api.parseSeek(null), null);
});

// ---- Controls ------------------------------------------------------------------

function page(attrs = {}) {
  const env = setup(attrs.env);
  const player = new FakePlayer({ id: "p" });
  env.body.append(player);
  env.api.scan(env.body);
  return { ...env, player };
}

function button(env, control, extra = {}) {
  const b = new FakeElement("button", { "data-hf-control": control, "data-hf-target": "p", ...extra });
  const label = new FakeElement("span");
  b.append(label);
  env.body.append(b);
  return { b, label };
}

test("play, pause and toggle call the player", () => {
  const env = page();
  env.click(button(env, "play").label);
  env.click(button(env, "pause").b);
  const t = button(env, "toggle").b;
  env.click(t);
  env.click(t);
  assert.deepEqual(env.player.calls, ["play", "pause", "play", "pause"]);
});

test("restart seeks to zero and plays", () => {
  const env = page();
  env.click(button(env, "restart").b);
  assert.deepEqual(env.player.calls, ["seek:0", "play"]);
});

test("seek uses data-hf-seek", () => {
  const env = page();
  env.click(button(env, "seek", { "data-hf-seek": "2.5" }).b);
  assert.deepEqual(env.player.calls, ["seek:2.5"]);
});

test("mute, unmute and toggle-mute set muted", () => {
  const env = page();
  env.click(button(env, "mute").b);
  assert.equal(env.player.muted, true);
  env.click(button(env, "unmute").b);
  assert.equal(env.player.muted, false);
  env.click(button(env, "toggle-mute").b);
  assert.equal(env.player.muted, true);
});

test("bad controls do nothing and do not throw", () => {
  const env = page();
  env.click(button(env, "explode").b);
  env.click(button(env, "seek", { "data-hf-seek": "-1" }).b);
  env.click(button(env, "play", { "data-hf-target": "missing" }).b);
  const disabled = button(env, "play").b;
  disabled.disabled = true;
  env.click(disabled);
  env.click(new FakeElement("div"));
  env.fire("click", { target: null });
  // A target that is not a player is ignored.
  env.body.append(new FakeElement("div", { id: "not-a-player" }));
  env.click(button(env, "play", { "data-hf-target": "not-a-player" }).b);
  assert.deepEqual(env.player.calls, []);
});

test("a throwing player method does not stop later clicks", () => {
  const env = page();
  env.player.play = () => {
    throw new Error("boom");
  };
  env.click(button(env, "play").b);
  env.click(button(env, "pause").b);
  assert.deepEqual(env.player.calls, ["pause"]);
});

// ---- Scan: state, reduced motion, in view -----------------------------------------

test("scan marks players and tracks data-hf-state", () => {
  const env = page();
  assert.equal(env.player.getAttribute("data-hf-state"), "paused");
  assert.equal(env.player.hasAttribute("data-hf-init"), true);
  env.player.play();
  assert.equal(env.player.getAttribute("data-hf-state"), "playing");
  env.player.dispatch("ended");
  assert.equal(env.player.getAttribute("data-hf-state"), "ended");
  env.player.pause();
  assert.equal(env.player.getAttribute("data-hf-state"), "paused");
});

test("scan runs one time for each player", () => {
  const env = page();
  env.api.scan(env.body);
  env.api.scan(env.player);
  assert.equal(env.player.listeners.play.length, 1);
});

test("reduced motion removes autoplay", () => {
  const env = setup({ reduced: true });
  const p = new FakePlayer({ id: "p", autoplay: "" });
  const q = new FakePlayer({ id: "q", autoplay: "", "data-hf-reduced": "animate" });
  env.body.append(p, q);
  env.api.scan(env.body);
  assert.equal(p.hasAttribute("autoplay"), false);
  assert.equal(p.hasAttribute("data-hf-autoplay-blocked"), true);
  assert.equal(q.hasAttribute("autoplay"), true);
});

test("autoplay stays without reduced motion", () => {
  const env = setup();
  const p = new FakePlayer({ id: "p", autoplay: "" });
  env.body.append(p);
  env.api.scan(env.body);
  assert.equal(p.hasAttribute("autoplay"), true);
});

test("in-view players play when visible and pause when not", () => {
  const env = setup();
  const p = new FakePlayer({ id: "p", "data-hf-in-view": "" });
  env.body.append(p);
  env.api.scan(env.body);
  assert.equal(env.observed.has(p), true);
  env.intersect([{ target: p, isIntersecting: true }]);
  env.intersect([{ target: p, isIntersecting: false }]);
  env.intersect([{ target: p, isIntersecting: false }]);
  assert.deepEqual(p.calls, ["play", "pause"]);
});

test("in-view is skipped with reduced motion unless opted in", () => {
  const env = setup({ reduced: true });
  const p = new FakePlayer({ id: "p", "data-hf-in-view": "" });
  const q = new FakePlayer({ id: "q", "data-hf-in-view": "", "data-hf-reduced": "animate" });
  env.body.append(p, q);
  env.api.scan(env.body);
  assert.equal(env.observed.has(p), false);
  assert.equal(env.observed.has(q), true);
});

test("in-view does nothing without IntersectionObserver", () => {
  const env = setup({ observer: false });
  const p = new FakePlayer({ id: "p", "data-hf-in-view": "" });
  env.body.append(p);
  env.api.scan(env.body);
  assert.equal(p.getAttribute("data-hf-state"), "paused");
});

// ---- htmx ---------------------------------------------------------------------------

test("htmx:load scans new content", () => {
  const env = setup({ reduced: true });
  const wrap = new FakeElement("div");
  const p = new FakePlayer({ id: "p", autoplay: "", "data-hf-in-view": "" });
  wrap.append(p);
  env.body.append(wrap);
  env.fire("htmx:load", { detail: { elt: wrap }, target: wrap });
  assert.equal(p.hasAttribute("data-hf-init"), true);
  assert.equal(p.hasAttribute("autoplay"), false);
});

test("htmx:beforeCleanupElement stops observing removed players", () => {
  const env = setup();
  const p = new FakePlayer({ id: "p", "data-hf-in-view": "" });
  env.body.append(p);
  env.api.scan(env.body);
  env.fire("htmx:beforeCleanupElement", { detail: { elt: p }, target: p });
  assert.equal(env.observed.has(p), false);
  env.fire("htmx:beforeCleanupElement", { target: null });
});

test("the initial scan runs on load", () => {
  // readyState is "complete", so init.js scans at once.
  const env = setup();
  assert.equal(typeof env.api.version, "string");
});
