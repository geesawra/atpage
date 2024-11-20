import init, { resolve, init_wasm_log } from "./pkg/atpage.js";


const broadcast = new BroadcastChannel('sw');

self.addEventListener("install", () => {
  (async () => {
    await init().then(() => {
      init_wasm_log();
    });
  })();
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  broadcast.postMessage({ type: 'ACTIVATED' })
  event.waitUntil(clients.claim());
});

self.addEventListener("fetch", (event) => {
  event.respondWith(
    (async () => {
      try {
        const res = await resolve(event);
        return res;
      } catch (error) {
        console.log("[SW] Fetch error:", error);
        return;
      }
    })()
  );
});
