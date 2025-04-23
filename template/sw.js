import init, { resolve, is_at } from "/mod/atpage_renderer.js";

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
            initialized = true;
          });
        }

        if (!(await is_at(event))) {
          return fetch(event.request);
        }

        return await resolve(event);
      } catch (error) {
        console.log("atpage fetch error: ", error, event);
      }
    })(),
  );
});
