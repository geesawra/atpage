import init, { resolve } from "./pkg/atpage.js";

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
          await init();
          const res = await resolve(event);
          return res;
        } catch (error) {
          console.log("[SW] Fetch error:", error);
          return;
        }
      })()
    );
});
