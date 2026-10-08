// Browser tests: run the demo app and check the players in Chromium.
//
// Build the demo first: `cargo build --example hyperframes_demo`.
// Then run: `npm --prefix tests/e2e test`.
// DEMO_BIN sets another demo binary. CHROMIUM sets another browser binary.
import { after, before, describe, test } from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createWriteStream, existsSync, mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { chromium } from "playwright";

const ROOT = new URL("../../", import.meta.url).pathname;
const BIN = process.env.DEMO_BIN || `${ROOT}target/debug/examples/hyperframes_demo`;
const LOGS = `${ROOT}tests/e2e/test-results`;
// Minimum line coverage of init.js over the whole e2e run.
const MIN_INIT_JS_LINES = 85;
// The Autumn default CSP with `frame-ancestors 'self'`: the config for `src` mode.
const FRAME_SELF_CSP =
  "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; " +
  "script-src 'self'; connect-src 'self'; form-action 'self'; frame-ancestors 'self'; " +
  "base-uri 'self'";
// Chromium warns about every sandbox with allow-scripts and allow-same-origin.
// The player uses that sandbox by default (README: "Security").
const SANDBOX_WARNING = /allow-scripts and allow-same-origin/;

const servers = [];
let browser;
// The coverage gate needs every test. A failure or a name filter skips it.
let failures = 0;
const filtered = process.execArgv.concat(process.argv).some((a) => a.includes("test-name-pattern"));

// `test`, but it counts failures for the coverage gate.
function it(name, fn) {
  return test(name, async () => {
    try {
      await fn();
    } catch (e) {
      failures += 1;
      throw e;
    }
  });
}

// Retries `fn` until it stops throwing, or throws its last error after `ms`.
async function eventually(fn, ms = 5000) {
  const end = Date.now() + ms;
  for (;;) {
    try {
      return await fn();
    } catch (e) {
      if (Date.now() > end) throw e;
      await new Promise((r) => setTimeout(r, 100));
    }
  }
}

// Asks the OS for a free port.
function freePort() {
  return new Promise((resolve, reject) => {
    const srv = createServer();
    srv.once("error", reject);
    srv.listen(0, "127.0.0.1", () => {
      const { port } = srv.address();
      srv.close(() => resolve(port));
    });
  });
}

async function startServer(env = {}) {
  const port = await freePort();
  mkdirSync(LOGS, { recursive: true });
  const log = createWriteStream(`${LOGS}/server-${port}.log`);
  const child = spawn(BIN, [], {
    cwd: ROOT,
    env: { ...process.env, AUTUMN_SERVER__PORT: String(port), NO_COLOR: "1", ...env },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.pipe(log);
  child.stderr.pipe(log);
  servers.push(child);
  const url = `http://127.0.0.1:${port}`;
  for (let i = 0; i < 100; i++) {
    if (child.exitCode !== null) {
      throw new Error(`demo exited with ${child.exitCode}; see ${LOGS}/server-${port}.log`);
    }
    try {
      const res = await fetch(url);
      // Check that this is our demo, not another server on the port.
      if (res.ok && (await res.text()).includes("<title>HyperFrames demo</title>")) return url;
    } catch {
      // The server is not ready.
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`demo did not start on ${url}`);
}

// V8 block coverage of init.js: the highest count for each source character.
const coverage = { source: null, counts: null };

function mergeCoverage(entries) {
  for (const entry of entries) {
    if (!/\/_plugins\/hyperframes\/init\.[0-9a-f]{8}\.js$/.test(entry.url)) continue;
    const src = entry.source;
    if (coverage.source === null) {
      coverage.source = src;
      coverage.counts = new Uint32Array(src.length);
    }
    const run = new Uint32Array(src.length);
    // Paint outer ranges first, then the nested ranges over them.
    const ranges = entry.functions
      .flatMap((f) => f.ranges)
      .sort((a, b) => a.startOffset - b.startOffset || b.endOffset - a.endOffset);
    for (const r of ranges) run.fill(r.count, r.startOffset, r.endOffset);
    for (let i = 0; i < run.length; i++) {
      if (run[i] > coverage.counts[i]) coverage.counts[i] = run[i];
    }
  }
}

// Line coverage: a code line is covered when one of its characters ran.
function lineCoverage() {
  let offset = 0;
  let code = 0;
  let hit = 0;
  for (const line of coverage.source.split("\n")) {
    const t = line.trim();
    const comment = t.startsWith("//") || t.startsWith("/*") || t.startsWith("*");
    if (t !== "" && !comment) {
      code += 1;
      let covered = false;
      for (let i = 0; i < line.length && !covered; i++) {
        covered = line[i] !== " " && coverage.counts[offset + i] > 0;
      }
      if (covered) hit += 1;
    }
    offset += line.length + 1;
  }
  return { code, hit, pct: (100 * hit) / code };
}

// Opens a page. Records console errors, page errors, CSP violations and
// requests to other origins.
async function open(base, path = "/", options = {}) {
  const context = await browser.newContext({ viewport: { width: 1200, height: 800 }, ...options });
  const page = await context.newPage();
  await page.coverage.startJSCoverage({ resetOnNavigation: false });
  const close = context.close.bind(context);
  context.close = async () => {
    mergeCoverage(await page.coverage.stopJSCoverage());
    return close();
  };
  const errors = [];
  const foreign = [];
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(m.text());
    if (m.type() === "warning" && !SANDBOX_WARNING.test(m.text())) errors.push(m.text());
  });
  page.on("pageerror", (e) => errors.push(String(e)));
  page.on("request", (r) => {
    if (!r.url().startsWith(base) && !r.url().startsWith("about:")) foreign.push(r.url());
  });
  await page.addInitScript(() => {
    window.__csp = [];
    document.addEventListener("securitypolicyviolation", (e) => {
      window.__csp.push(`${e.violatedDirective} ${e.blockedURI}`);
    });
    // Record the targets that init.js stops observing.
    window.__unobserved = [];
    const io = window.IntersectionObserver && window.IntersectionObserver.prototype;
    if (io) {
      const unobserve = io.unobserve;
      io.unobserve = function (el) {
        window.__unobserved.push(el.id);
        return unobserve.call(this, el);
      };
    }
  });
  await page.goto(base + path);
  return { page, context, errors, foreign };
}

const ready = (page, id, timeout = 10000) =>
  page.waitForFunction((i) => document.getElementById(i)?.ready === true, id, { timeout });

// Seeks a player, then waits for the time and two frames in the iframe.
async function seek(page, id, t) {
  await page.evaluate(
    async ([i, s]) => {
      const p = document.getElementById(i);
      p.seek(s);
      const end = performance.now() + 5000;
      while (Math.abs(p.currentTime - s) > 0.05 && performance.now() < end) {
        await new Promise((r) => setTimeout(r, 20));
      }
      const w = p.iframeElement.contentWindow;
      await new Promise((r) => w.requestAnimationFrame(() => w.requestAnimationFrame(r)));
    },
    [id, t],
  );
}

// Waits until the player has loaded its assets, so a queued autoplay would have started.
const settled = (page, id) =>
  page.waitForFunction((i) => document.getElementById(i)?.assetsReady === true, id, { timeout: 10000 });

// The computed visibility of clip `clip` inside player `id`.
const visibility = (page, id, clip) =>
  page.evaluate(
    ([i, c]) => {
      const d = document.getElementById(i).iframeElement.contentDocument;
      return d.defaultView.getComputedStyle(d.getElementById(c)).visibility;
    },
    [id, clip],
  );

const player = (page, id) =>
  page.evaluate((i) => {
    const p = document.getElementById(i);
    return {
      ready: p.ready,
      paused: p.paused,
      duration: p.duration,
      currentTime: p.currentTime,
      muted: p.muted,
      state: p.getAttribute("data-hf-state"),
      autoplay: p.hasAttribute("autoplay"),
    };
  }, id);

let url;
let frameUrl;

before(async () => {
  assert.ok(existsSync(BIN), `build the demo first: cargo build --example hyperframes_demo (${BIN})`);
  url = await startServer();
  // This server also has no CORS for the null origin (as in production).
  frameUrl = await startServer({
    AUTUMN_SECURITY__HEADERS__X_FRAME_OPTIONS: "SAMEORIGIN",
    AUTUMN_SECURITY__HEADERS__CONTENT_SECURITY_POLICY: FRAME_SELF_CSP,
    AUTUMN_CORS__ALLOWED_ORIGINS: "https://example.com",
  });
  browser = await chromium.launch(
    process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {},
  );
});

after(async () => {
  await browser?.close();
  for (const s of servers) s.kill();
  if (failures > 0 || filtered) {
    console.log("# init.js coverage gate skipped: not a full, green run");
    return;
  }
  assert.ok(coverage.source, "the e2e run loaded init.js");
  const { code, hit, pct } = lineCoverage();
  console.log(`# init.js line coverage: ${pct.toFixed(1)}% (${hit}/${code})`);
  assert.ok(pct >= MIN_INIT_JS_LINES, `init.js line coverage ${pct.toFixed(1)}% < ${MIN_INIT_JS_LINES}%`);
});

describe("default CSP, srcdoc mode", () => {
  it("every player gets ready with no errors and no other origin", async () => {
    const { page, context, errors, foreign } = await open(url);
    for (const id of ["intro-player", "plans-player", "media-player", "video-player", "opaque-player", "in-view-player"]) {
      await ready(page, id);
    }
    assert.deepEqual((await player(page, "intro-player")).duration, 6);
    assert.deepEqual((await player(page, "plans-player")).duration, 6);
    assert.deepEqual((await player(page, "in-view-player")).duration, 3);
    assert.deepEqual(await page.evaluate(() => window.__csp), []);
    assert.deepEqual(errors, []);
    assert.deepEqual(foreign, [], "no request to a CDN");
    await context.close();
  });

  it("the srcdoc loads the vendored runtime one time", async () => {
    const { page, context } = await open(url);
    await ready(page, "intro-player");
    const scripts = await page.evaluate(() =>
      Array.from(document.getElementById("intro-player").iframeElement.contentDocument.scripts).map(
        (s) => new URL(s.src).pathname,
      ),
    );
    assert.equal(scripts.length, 1, JSON.stringify(scripts));
    // The srcdoc links the plain URL, so the player adds no second copy.
    assert.equal(scripts[0], "/static/_plugins/hyperframes/hyperframe.runtime.iife.js");
    await context.close();
  });

  it("clips show only in their time window", async () => {
    const { page, context } = await open(url);
    await ready(page, "intro-player");
    const at = async (t) => {
      await seek(page, "intro-player", t);
      const out = {};
      for (const c of ["title", "tagline", "logo", "outro"]) {
        out[c] = await visibility(page, "intro-player", c);
      }
      return out;
    };
    const expect = (t, want) => eventually(async () => assert.deepEqual(await at(t), want));
    await expect(0.5, { title: "visible", tagline: "hidden", logo: "hidden", outro: "hidden" });
    // `logo` starts 0.5 s before `title` ends (Start::after("title").minus(500 ms)).
    await expect(1.75, { title: "visible", tagline: "hidden", logo: "visible", outro: "hidden" });
    await expect(2.5, { title: "hidden", tagline: "visible", logo: "visible", outro: "hidden" });
    await expect(4.5, { title: "hidden", tagline: "hidden", logo: "hidden", outro: "visible" });
    await context.close();
  });

  it("the runtime seeks CSS animations from the composition stylesheet", async () => {
    const { page, context } = await open(url);
    await ready(page, "intro-player");
    const opacity = async (t) => {
      await seek(page, "intro-player", t);
      return page.evaluate(() => {
        const d = document.getElementById("intro-player").iframeElement.contentDocument;
        return Number(d.defaultView.getComputedStyle(d.querySelector("#title h1")).opacity);
      });
    };
    await eventually(async () => assert.equal(await opacity(0), 0));
    await eventually(async () => {
      const mid = await opacity(0.4);
      assert.ok(mid > 0 && mid < 1, `mid-animation opacity ${mid}`);
    });
    await eventually(async () => assert.equal(await opacity(1.5), 1));
    await context.close();
  });

  it("nested compositions play with their own values and defaults", async () => {
    const { page, context } = await open(url);
    await ready(page, "plans-player");
    const shown = async (t) => {
      await seek(page, "plans-player", t);
      return page.evaluate(() => {
        const d = document.getElementById("plans-player").iframeElement.contentDocument;
        return Array.from(d.querySelectorAll("h1"))
          .filter((h) => d.defaultView.getComputedStyle(h.closest("[data-start]")).visibility === "visible")
          .map((h) => `${h.textContent} ${d.defaultView.getComputedStyle(h).color}`);
      });
    };
    // "Pro" sets its own accent. "Team" keeps the default from the template <html>.
    await eventually(async () => assert.deepEqual(await shown(1), ["Pro rgb(94, 176, 240)"]));
    await eventually(async () => assert.deepEqual(await shown(4), ["Team rgb(240, 163, 94)"]));
    await context.close();
  });

  it("control buttons drive the player and data-hf-state follows", async () => {
    const { page, context, errors } = await open(url);
    await ready(page, "intro-player");
    const btn = (c) => `[data-hf-target="intro-player"][data-hf-control="${c}"]`;
    await page.click(btn("play"));
    await page.waitForFunction(() => document.getElementById("intro-player").getAttribute("data-hf-state") === "playing");
    await page.click(btn("pause"));
    await page.waitForFunction(() => document.getElementById("intro-player").getAttribute("data-hf-state") === "paused");
    await page.click(btn("play"));
    await page.waitForFunction(() => document.getElementById("intro-player").getAttribute("data-hf-state") === "playing");
    // Seek pauses the player. init.js updates the state.
    await page.click(btn("seek"));
    await page.waitForFunction(() => Math.abs(document.getElementById("intro-player").currentTime - 2.5) < 0.05);
    await page.waitForFunction(() => document.getElementById("intro-player").getAttribute("data-hf-state") === "paused");
    await page.click(btn("toggle-mute"));
    assert.equal((await player(page, "intro-player")).muted, true);
    await page.click(btn("restart"));
    await page.waitForFunction(() => !document.getElementById("intro-player").paused);
    assert.ok((await player(page, "intro-player")).currentTime < 2.5);
    assert.deepEqual(errors, []);
    await context.close();
  });

  it("an in-view player plays when visible and pauses when not", async () => {
    const { page, context } = await open(url);
    await ready(page, "in-view-player");
    assert.equal((await player(page, "in-view-player")).paused, true);
    await page.locator("#in-view-player").scrollIntoViewIfNeeded();
    await page.waitForFunction(() => !document.getElementById("in-view-player").paused);
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.waitForFunction(() => document.getElementById("in-view-player").paused);
    await context.close();
  });

  it("htmx adds a player that gets ready, autoplays and takes controls", async () => {
    const { page, context, errors } = await open(url);
    await page.waitForFunction(() => window.htmx && window.AutumnHyperframes);
    await page.click("#more");
    const id = await page.waitForFunction(() => document.querySelector("#cards hyperframes-player")?.id);
    const pid = await id.jsonValue();
    await ready(page, pid);
    const info = await player(page, pid);
    assert.equal(info.duration, 3);
    assert.equal(
      await page.evaluate((i) => document.getElementById(i).hasAttribute("data-hf-autoplay"), pid),
      true,
    );
    // init.js adds `autoplay` before ready, or plays after ready.
    await page.waitForFunction((i) => !document.getElementById(i).paused, pid);
    await page.evaluate((i) => document.getElementById(i).pause(), pid);
    await page.click(`[data-hf-target="${pid}"]`);
    await page.waitForFunction((i) => !document.getElementById(i).paused, pid);
    assert.deepEqual(errors, []);
    await context.close();
  });

  it("htmx cleanup stops observing a removed in-view player", async () => {
    const { page, context, errors } = await open(url);
    await ready(page, "in-view-player");
    await page.waitForFunction(() => window.htmx && window.AutumnHyperframes);
    // htmx runs the cleanup when it swaps the section out.
    await page.evaluate(() => window.htmx.swap("#in-view-section", "<p>gone</p>", { swapStyle: "outerHTML" }));
    await page.waitForFunction(() => !document.getElementById("in-view-player"));
    assert.ok(
      (await page.evaluate(() => window.__unobserved)).includes("in-view-player"),
      "init.js unobserved the removed player",
    );
    assert.deepEqual(errors, []);
    await context.close();
  });

  it("media clips play in a composition", async () => {
    const { page, context, errors } = await open(url);
    await ready(page, "media-player");
    assert.equal((await player(page, "media-player")).duration, 2);
    await eventually(async () => {
      await seek(page, "media-player", 1);
      const state = await page.evaluate(() => {
        const d = document.getElementById("media-player").iframeElement.contentDocument;
        const v = d.getElementById("footage");
        const a = d.getElementById("tone");
        const img = d.getElementById("badge");
        return {
          video: [d.defaultView.getComputedStyle(v).visibility, Math.round(v.currentTime * 10) / 10, v.muted],
          audio: a.getAttribute("data-volume"),
          image: [d.defaultView.getComputedStyle(img).visibility, img.getAttribute("data-var-src")],
        };
      });
      assert.deepEqual(state, {
        video: ["visible", 1, true],
        audio: "0.5",
        image: ["visible", "badge"],
      });
    });
    assert.deepEqual(errors, []);
    await context.close();
  });

  it("a video file plays in the player", async () => {
    const { page, context, errors } = await open(url);
    await ready(page, "video-player");
    assert.equal(Math.round((await player(page, "video-player")).duration), 2);
    await page.evaluate(() => document.getElementById("video-player").play());
    await page.waitForFunction(() => document.getElementById("video-player").currentTime > 0.2);
    assert.deepEqual(errors, []);
    await context.close();
  });

  it("the opaque sandbox plays a srcdoc composition with no CORS errors", async () => {
    // frameUrl sends no CORS headers for the null origin, as in production.
    const { page, context, errors } = await open(frameUrl);
    await ready(page, "opaque-player");
    assert.equal((await player(page, "opaque-player")).duration, 6);
    assert.equal(
      await page.evaluate(() =>
        document.getElementById("opaque-player").iframeElement.getAttribute("sandbox").includes("allow-same-origin"),
      ),
      false,
    );
    await settled(page, "opaque-player");
    assert.deepEqual(errors, []);
    await context.close();
  });

  it("a composition page works on its own", async () => {
    const { page, context, errors, foreign } = await open(url, "/compositions/intro");
    await page.waitForFunction(() => typeof window.__hyperframes === "object");
    const root = await page.evaluate(() => {
      const r = document.querySelector("[data-composition-id]");
      return [r.id, r.getAttribute("data-duration"), r.getBoundingClientRect().width];
    });
    assert.deepEqual(root, ["intro", "6", 1280]);
    assert.deepEqual(errors, []);
    assert.deepEqual(foreign, []);
    await context.close();
  });
});

describe("reduced motion", () => {
  it("no autoplay and no in-view play", async () => {
    const { page, context, errors } = await open(url, "/", { reducedMotion: "reduce" });
    await ready(page, "in-view-player");
    await page.locator("#in-view-player").scrollIntoViewIfNeeded();
    await settled(page, "in-view-player");
    await page.waitForTimeout(500);
    assert.equal((await player(page, "in-view-player")).paused, true);
    await page.waitForFunction(() => window.htmx && window.AutumnHyperframes);
    await page.click("#more");
    const id = await page.waitForFunction(() => document.querySelector("#cards hyperframes-player")?.id);
    const pid = await id.jsonValue();
    await ready(page, pid);
    await settled(page, pid);
    const info = await player(page, pid);
    assert.equal(info.autoplay, false, "init.js did not add autoplay");
    assert.equal(info.paused, true);
    assert.equal(
      await page.evaluate((i) => document.getElementById(i).hasAttribute("data-hf-autoplay-blocked"), pid),
      true,
    );
    // The user can still play it.
    await page.click(`[data-hf-target="${pid}"]`);
    await page.waitForFunction((i) => !document.getElementById(i).paused, pid);
    assert.deepEqual(errors, []);
    await context.close();
  });
});

describe("src mode", () => {
  it("plays a composition URL when the app allows same-origin frames", async () => {
    const { page, context, errors, foreign } = await open(frameUrl, "/src-mode");
    await ready(page, "src-player");
    assert.equal((await player(page, "src-player")).duration, 6);
    await eventually(async () => {
      await seek(page, "src-player", 2.5);
      assert.equal(await visibility(page, "src-player", "tagline"), "visible");
    });
    assert.deepEqual(errors, []);
    assert.deepEqual(foreign, []);
    await context.close();
  });

  it("the Autumn default headers block the frame", async () => {
    const { page, context, errors } = await open(url, "/src-mode");
    await eventually(() =>
      assert.ok(
        errors.some((e) => e.includes("frame-ancestors")),
        `expected a frame-ancestors error: ${JSON.stringify(errors)}`,
      ),
    10000);
    assert.equal((await player(page, "src-player")).ready, false);
    await context.close();
  });
});
