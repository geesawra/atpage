var homepage = "/at/geesawra.industries/industries.geesawra.website/0J752KTKC1NYS";

const location_params = new URLSearchParams(window.location.search);
const redir_url = String(location_params.get('redir'));

if (redir_url != null && redir_url.startsWith("/at/")) {
  homepage = redir_url;
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
