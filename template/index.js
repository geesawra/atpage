var homepage = "REPLACE_ME";

let current_location = window.location.pathname;

if (current_location != null && current_location != "/"  && current_location.startsWith("/at/")) {
  homepage = current_location;
}

navigator.serviceWorker.ready.then(() => {
  window.location.replace(homepage);
});

var sw_path = "./sw.js";

if (navigator.userAgent.toLowerCase().indexOf('firefox') !== -1) {
  sw_path = "./sw_nomod.js";
}
if ("serviceWorker" in navigator) {
  try {
    const registration = await navigator.serviceWorker.register(sw_path, {
      type: "module",
    });

    if (registration.active) {
      // we're already installed and active, redirect
      window.location.replace(homepage);
    }
  } catch (error) {
    console.error(`Registration failed with ${error}`);
  }
}
