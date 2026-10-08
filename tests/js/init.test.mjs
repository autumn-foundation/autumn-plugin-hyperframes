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
    this.ready = false;
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
  // Upstream seek() pauses without a "pause" event.
  seek(t) {
    this.calls.push(`seek:${t}`);
    this.currentTime = t;
    this.paused = true;
  }
}

function setup({ reduced = false, observer = true, readyState = "complete", before = [] } = {}) {
  const body = new FakeElement("body");
  body.append(...before);
  const listeners = {};
  const observed = new Set();
  const unobservedLog = [];
  let intersect = null;
  const document = {
    body,
    readyState,
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
        unobservedLog.push(el);
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
    unobserved: () => [...unobservedLog],
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
    if (p.autoplay) el.setAttribute("data-hf-autoplay", "");
    if (p.reduced !== null) el.setAttribute("data-hf-reduced", p.reduced);
    assert.equal(api.wantsInView(el), p.expect.inView);
    assert.equal(api.wantsAutoplay(el), p.expect.autoplay);
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

test("seek updates data-hf-state (upstream seek pauses without an event)", () => {
  const env = page();
  env.player.play();
  assert.equal(env.player.getAttribute("data-hf-state"), "playing");
  env.click(button(env, "seek", { "data-hf-seek": "1" }).b);
  assert.equal(env.player.getAttribute("data-hf-state"), "paused");
  env.click(button(env, "restart").b);
  assert.equal(env.player.getAttribute("data-hf-state"), "playing");
});

test("a rejected play() promise does not stop later clicks", async () => {
  const env = page();
  let rejected = 0;
  env.player.play = () => {
    rejected += 1;
    return Promise.reject(new Error("blocked"));
  };
  env.click(button(env, "play").b);
  env.click(button(env, "pause").b);
  await new Promise((r) => setTimeout(r, 0));
  assert.equal(rejected, 1);
  assert.deepEqual(env.player.calls, ["pause"]);
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

test("scan tracks data-hf-state and leaves no marker in the DOM", () => {
  const env = page();
  assert.equal(env.player.getAttribute("data-hf-state"), "paused");
  // A DOM marker would go into the htmx history snapshot.
  assert.equal([...env.player.attrs.keys()].some((k) => k.endsWith("-init")), false);
  env.player.play();
  assert.equal(env.player.getAttribute("data-hf-state"), "playing");
  env.player.dispatch("ended");
  assert.equal(env.player.getAttribute("data-hf-state"), "ended");
  env.player.pause();
  assert.equal(env.player.getAttribute("data-hf-state"), "paused");
});

test("a player restored from htmx history is set up again", () => {
  const env = setup();
  // The snapshot keeps the attributes of the old page.
  const p = new FakePlayer({ id: "p", "data-hf-state": "playing", "data-hf-init": "" });
  env.body.append(p);
  env.fire("htmx:load", { detail: { elt: p }, target: p });
  assert.equal(p.getAttribute("data-hf-state"), "paused");
  assert.equal(p.listeners.play.length, 1);
});

test("scan runs one time for each player", () => {
  const env = page();
  env.api.scan(env.body);
  env.api.scan(env.player);
  assert.equal(env.player.listeners.play.length, 1);
});

test("reduced motion blocks data-hf-autoplay and a hand-written autoplay", () => {
  const env = setup({ reduced: true });
  const p = new FakePlayer({ id: "p", "data-hf-autoplay": "" });
  const q = new FakePlayer({ id: "q", "data-hf-autoplay": "", "data-hf-reduced": "animate" });
  const r = new FakePlayer({ id: "r", autoplay: "" });
  env.body.append(p, q, r);
  env.api.scan(env.body);
  assert.equal(p.hasAttribute("autoplay"), false);
  assert.equal(p.hasAttribute("data-hf-autoplay-blocked"), true);
  assert.equal(q.hasAttribute("autoplay"), true);
  assert.equal(r.hasAttribute("autoplay"), false);
  assert.equal(r.hasAttribute("data-hf-autoplay-blocked"), true);
});

test("data-hf-autoplay adds autoplay before ready and plays after ready", () => {
  const env = setup();
  const early = new FakePlayer({ id: "a", "data-hf-autoplay": "" });
  const late = new FakePlayer({ id: "b", "data-hf-autoplay": "" });
  late.ready = true;
  env.body.append(early, late);
  env.api.scan(env.body);
  assert.equal(early.hasAttribute("autoplay"), true);
  assert.deepEqual(early.calls, []);
  assert.deepEqual(late.calls, ["play"]);
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
  env.api.cleanup(env.body);
  assert.deepEqual(p.calls, []);
});

// ---- htmx ---------------------------------------------------------------------------

test("htmx:load scans new content", () => {
  const env = setup({ reduced: true });
  const wrap = new FakeElement("div");
  const p = new FakePlayer({ id: "p", autoplay: "", "data-hf-in-view": "" });
  wrap.append(p);
  env.body.append(wrap);
  env.fire("htmx:load", { detail: { elt: wrap }, target: wrap });
  assert.equal(p.getAttribute("data-hf-state"), "paused");
  assert.equal(p.hasAttribute("autoplay"), false);
});

test("htmx:beforeCleanupElement unobserves the player it names", () => {
  const env = setup();
  const wrap = new FakeElement("div");
  const p = new FakePlayer({ id: "p", "data-hf-in-view": "" });
  wrap.append(p);
  env.body.append(wrap);
  env.api.scan(env.body);
  // htmx fires the event on each removed element, so a wrapper does no work.
  env.fire("htmx:beforeCleanupElement", { detail: { elt: wrap }, target: wrap });
  assert.deepEqual(env.unobserved(), []);
  env.fire("htmx:beforeCleanupElement", { detail: { elt: p }, target: p });
  assert.deepEqual(env.unobserved(), [p]);
  env.fire("htmx:beforeCleanupElement", { target: null });
});

test("cleanup(root) unobserves every player in the subtree", () => {
  const env = setup();
  const wrap = new FakeElement("div");
  const p = new FakePlayer({ id: "p", "data-hf-in-view": "" });
  wrap.append(p);
  env.body.append(wrap);
  env.api.scan(env.body);
  env.api.cleanup(wrap);
  assert.deepEqual(env.unobserved(), [p]);
});

test("the initial scan runs at once when the page is parsed", () => {
  const p = new FakePlayer({ id: "p" });
  setup({ before: [p] });
  assert.equal(p.getAttribute("data-hf-state"), "paused");
});

test("the initial scan waits for DOMContentLoaded while the page loads", () => {
  const p = new FakePlayer({ id: "p" });
  const env = setup({ readyState: "loading", before: [p] });
  assert.equal(p.getAttribute("data-hf-state"), null);
  env.fire("DOMContentLoaded", {});
  assert.equal(p.getAttribute("data-hf-state"), "paused");
});
