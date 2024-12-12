let homepage = "/at/geesawra.industries/industries.geesawra.website/0J5SYQ0SVQTKF";

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
