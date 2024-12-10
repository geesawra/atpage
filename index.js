let homepage = "/at/geesawra.industries/industries.geesawra.website/0J560QSE5DNTS";

navigator.serviceWorker.ready.then(() => {
  window.location.replace(homepage);
});

if ("serviceWorker" in navigator) {
  try {
    const registration = await navigator.serviceWorker.register("./sw.js", {
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
