importScripts("/nomod/atpage_renderer.js");

const { resolve, is_at } = wasm_bindgen;

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
          await wasm_bindgen({
            module_or_path: "/nomod/atpage_renderer_bg.wasm",
          }).then(() => {
            initialized = true;
          });
        }

        if (!(await is_at(event))) {
          return fetch(event.request);
        }

        return await resolve(event);
      } catch (error) {
        console.log("atpage fetch error: ", error, event);
        return;
      }
    })(),
  );
});
