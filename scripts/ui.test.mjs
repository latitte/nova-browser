import { test } from "node:test";
import assert from "node:assert/strict";
import { readdir } from "node:fs/promises";
import { JSDOM } from "jsdom";

test("interface : onglets, paramètres persistants, favoris vides et refus de navigation web", async (t) => {
  const dom = new JSDOM('<!doctype html><div id="app"></div>', {
    url: "http://localhost:1420/",
    pretendToBeVisual: true,
  });
  const media = {
    matches: true,
    addEventListener() {},
    removeEventListener() {},
  };
  for (const name of [
    "window",
    "document",
    "navigator",
    "HTMLElement",
    "HTMLInputElement",
    "Element",
    "Node",
    "Text",
    "Comment",
    "Event",
    "MouseEvent",
    "KeyboardEvent",
    "CustomEvent",
    "MutationObserver",
    "localStorage",
    "getComputedStyle",
  ]) {
    Object.defineProperty(globalThis, name, {
      value:
        name === "getComputedStyle"
          ? dom.window.getComputedStyle.bind(dom.window)
          : dom.window[name],
      configurable: true,
    });
  }
  for (const name of Object.getOwnPropertyNames(dom.window).filter(
    (name) =>
      /^(HTML|SVG)/.test(name) ||
      ["Document", "DocumentFragment", "ShadowRoot"].includes(name),
  )) {
    Object.defineProperty(globalThis, name, {
      value: dom.window[name],
      configurable: true,
    });
  }
  globalThis.matchMedia = () => media;
  globalThis.ResizeObserver = class {
    observe() {}
    disconnect() {}
  };
  const intervals = [];
  const originalInterval = globalThis.setInterval;
  globalThis.setInterval = (...args) => {
    const timer = originalInterval(...args);
    intervals.push(timer);
    return timer;
  };
  t.after(() => {
    intervals.forEach(clearInterval);
    globalThis.setInterval = originalInterval;
    dom.window.close();
  });
  const js = (await readdir(new URL("../dist/assets/", import.meta.url))).find(
    (f) => f.endsWith(".js"),
  );
  await import(new URL(`../dist/assets/${js}`, import.meta.url));
  const settle = () => new Promise((resolve) => setTimeout(resolve, 25));
  const click = async (selector) => {
    const el = document.querySelector(selector);
    assert.ok(el, selector);
    el.click();
    await settle();
  };
  await settle();
  assert.equal(document.querySelectorAll(".tab").length, 1);
  await click(".new-tab");
  assert.equal(document.querySelectorAll(".tab").length, 2);
  await click(".tab.active .tab-close");
  assert.equal(document.querySelectorAll(".tab").length, 1);
  await click('[title="Paramètres"]');
  assert.match(document.querySelector("h1").textContent, /détails/);
  const light = [...document.querySelectorAll(".segmented button")].find(
    (b) => b.textContent === "Clair",
  );
  light.click();
  await settle();
  assert.ok(document.querySelector(".app.light"));
  assert.equal(
    JSON.parse(localStorage.getItem("nova-settings")).theme,
    "light",
  );
  await click('[title="Fermer les paramètres"]');
  const favorites = [
    ...document.querySelectorAll(".sidebar-bottom>button"),
  ].find((b) => b.textContent.includes("Favoris"));
  favorites.click();
  await settle();
  assert.match(
    document.querySelector(".empty").textContent,
    /Ajoutez une page/,
  );
  await click('[title="Fermer le panneau"]');
  const input = document.querySelector(".hero-search input");
  input.value = "example.org";
  input.dispatchEvent(new Event("input", { bubbles: true }));
  document
    .querySelector(".hero-search")
    .dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
  await settle();
  assert.match(
    document.querySelector('[role="alert"]').textContent,
    /application desktop/,
  );
  assert.ok(
    document.querySelector(".newtab-page"),
    "aucune navigation simulée",
  );
  intervals.forEach(clearInterval);
  globalThis.setInterval = originalInterval;
  dom.window.close();
});
