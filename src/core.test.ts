import { test } from "node:test";
import assert from "node:assert/strict";
import { address } from "./core.ts";
test("normalise les adresses et recherches", () => {
  assert.equal(address("example.org"), "https://example.org/");
  assert.equal(address("localhost:8000"), "http://localhost:8000/");
  assert.equal(
    address("une idée"),
    "https://duckduckgo.com/?q=une%20id%C3%A9e",
  );
  assert.equal(
    address("NOVA", "brave"),
    "https://search.brave.com/search?q=NOVA",
  );
});
test("refuse les protocoles actifs, les fichiers et les identifiants", () => {
  for (const value of [
    "javascript:alert(1)",
    "file:///etc/passwd",
    "data:text/html,test",
    "https://user:pass@example.org",
    "",
  ])
    assert.throws(() => address(value));
});
