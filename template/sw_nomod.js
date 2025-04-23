importScripts("/nomod/atpage_renderer.js");

const { resolve, init_wasm_log } = wasm_bindgen;

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
            console.log("initialize_wasm finished running!()");
            init_wasm_log();
            initialized = true;
          });
        }

        console.log("was initialized:", initialized);
        const res = await resolve(event);
        return res;
      } catch (error) {
        console.log("[SW] Fetch error:", error, event);
        return;
      }
    })(),
  );
});
