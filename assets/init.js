/* autumn-plugin-hyperframes: init.js
 *
 * Reads the data-hf-* attributes that the Rust builders write:
 *   - data-hf-control / data-hf-target / data-hf-seek: control buttons.
 *   - data-hf-autoplay: autoplay when motion is allowed.
 *   - data-hf-in-view: play when visible, pause when not.
 *   - data-hf-reduced="animate": play even with prefers-reduced-motion.
 * Sets data-hf-state ("playing", "paused", "ended") on each player.
 * Scans on load and on htmx:load. Stops observing on htmx:beforeCleanupElement.
 * Set-up players are kept in a WeakSet, not in a DOM attribute, because htmx
 * puts the DOM into its history snapshot.
 * A bad value is ignored. It does not stop the scan.
 */
(function () {
  "use strict";

  var PLAYER = "hyperframes-player";
  var CONTROLS = ["play", "pause", "toggle", "restart", "seek", "mute", "unmute", "toggle-mute"];
  // Seconds with at most three decimals. The same grammar as Rust `seconds()`.
  var RE_SEEK = /^\d+(?:\.\d{1,3})?$/;
  var doc = window.document;
  var observer = null;
  var done = new WeakSet();

  function parseControl(value) {
    return typeof value === "string" && CONTROLS.indexOf(value) !== -1 ? value : null;
  }

  function parseSeek(value) {
    if (typeof value !== "string" || !RE_SEEK.test(value)) return null;
    return Number(value);
  }

  function reducedMotion() {
    return !!(
      window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches
    );
  }

  function animatesWhenReduced(player) {
    return player.getAttribute("data-hf-reduced") === "animate";
  }

  function allowsMotion(player) {
    return !reducedMotion() || animatesWhenReduced(player);
  }

  function wantsInView(player) {
    return player.hasAttribute("data-hf-in-view");
  }

  function wantsAutoplay(player) {
    return player.hasAttribute("data-hf-autoplay");
  }

  function isPlayer(el) {
    return !!el && typeof el.tagName === "string" && el.tagName.toLowerCase() === PLAYER;
  }

  // Calls a player method. A throw (for example, a blocked play) is ignored.
  function call(player, method, arg) {
    try {
      var result = player[method](arg);
      if (result && typeof result.catch === "function") result.catch(function () {});
    } catch (e) {
      /* ignored */
    }
  }

  function run(control, player, seek) {
    switch (control) {
      case "play":
        return call(player, "play");
      case "pause":
        return call(player, "pause");
      case "toggle":
        return call(player, player.paused ? "play" : "pause");
      case "restart":
        call(player, "seek", 0);
        return call(player, "play");
      case "seek":
        // The player pauses on seek but sends no "pause" event.
        if (seek !== null) call(player, "seek", seek);
        return syncState(player);
      case "mute":
        player.muted = true;
        return undefined;
      case "unmute":
        player.muted = false;
        return undefined;
      default:
        player.muted = !player.muted;
        return undefined;
    }
  }

  function onClick(event) {
    var target = event && event.target;
    if (!target || typeof target.closest !== "function") return;
    var button = target.closest("[data-hf-control]");
    if (!button || button.disabled) return;
    var control = parseControl(button.getAttribute("data-hf-control"));
    var id = button.getAttribute("data-hf-target");
    var player = control && id ? doc.getElementById(id) : null;
    if (!isPlayer(player)) return;
    run(control, player, parseSeek(button.getAttribute("data-hf-seek")));
  }

  function setState(player, state) {
    player.setAttribute("data-hf-state", state);
  }

  function syncState(player) {
    setState(player, player.paused === false ? "playing" : "paused");
  }

  function onIntersect(entries) {
    for (var i = 0; i < entries.length; i++) {
      var player = entries[i].target;
      if (entries[i].isIntersecting) {
        if (player.paused !== false) call(player, "play");
      } else if (player.paused === false) {
        call(player, "pause");
      }
    }
  }

  function getObserver() {
    if (!observer && typeof window.IntersectionObserver === "function") {
      observer = new window.IntersectionObserver(onIntersect, { threshold: 0.5 });
    }
    return observer;
  }

  function players(root) {
    if (!root) return [];
    var found = isPlayer(root) ? [root] : [];
    if (typeof root.querySelectorAll === "function") {
      var inner = root.querySelectorAll(PLAYER);
      for (var i = 0; i < inner.length; i++) found.push(inner[i]);
    }
    return found;
  }

  function setup(player) {
    if (done.has(player)) return;
    done.add(player);
    syncState(player);
    player.addEventListener("play", function () {
      setState(player, "playing");
    });
    player.addEventListener("pause", function () {
      setState(player, "paused");
    });
    player.addEventListener("ended", function () {
      setState(player, "ended");
    });
    if (!allowsMotion(player)) {
      if (wantsAutoplay(player) || player.hasAttribute("autoplay")) {
        // A hand-written autoplay can race "ready"; data-hf-autoplay cannot.
        player.removeAttribute("autoplay");
        player.setAttribute("data-hf-autoplay-blocked", "");
      }
    } else if (wantsAutoplay(player)) {
      // The player reads autoplay at "ready". After ready, play at once.
      if (player.ready === true) call(player, "play");
      else player.setAttribute("autoplay", "");
    }
    if (wantsInView(player) && allowsMotion(player)) {
      var io = getObserver();
      if (io) io.observe(player);
    }
  }

  function scan(root) {
    var list = players(root || doc.body);
    for (var i = 0; i < list.length; i++) setup(list[i]);
  }

  function cleanup(root) {
    if (!observer) return;
    var list = players(root);
    for (var i = 0; i < list.length; i++) observer.unobserve(list[i]);
  }

  function eventRoot(event) {
    return (event && event.detail && event.detail.elt) || (event && event.target) || null;
  }

  doc.addEventListener("click", onClick);
  doc.addEventListener("htmx:load", function (event) {
    scan(eventRoot(event));
  });
  // htmx sends this event for each removed element, so only a player needs work.
  doc.addEventListener("htmx:beforeCleanupElement", function (event) {
    var el = eventRoot(event);
    if (observer && isPlayer(el)) observer.unobserve(el);
  });

  window.AutumnHyperframes = {
    version: "0.1.0",
    scan: scan,
    cleanup: cleanup,
    parseControl: parseControl,
    parseSeek: parseSeek,
    wantsInView: wantsInView,
    wantsAutoplay: wantsAutoplay,
    animatesWhenReduced: animatesWhenReduced,
  };

  if (doc.readyState === "loading") {
    doc.addEventListener("DOMContentLoaded", function () {
      scan(doc.body);
    });
  } else {
    scan(doc.body);
  }
})();
