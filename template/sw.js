import init, { resolve, init_wasm_log } from "/mod/atpage_renderer.js";

var initialized = false;

self.addEventListener("install", () => {
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(clients.claim());
});

self.addEventListener("fetch", (event) => {
  event.respondWith(
    (async () => {
      try {
        if (!initialized) {
          await init().then(() => {
            init_wasm_log();
            initialized = true;
          });
        }
        const res = await resolve(event);
        return res;
      } catch (error) {
        console.log("[SW] Fetch error:", error, event);
        return;
      }
    })(),
  );
});
