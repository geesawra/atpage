let homepage = "./at/geesawra.industries/industries.geesawra.webpages/06Q5QFGDCM188";

const broadcast = new BroadcastChannel('sw');

broadcast.onmessage = (event) => {
  if (event.data) {
    if (event.data.type === 'ACTIVATED') {
      console.log(`service worker sent message saying we ${event.data.type}, redirect`);

      window.location.replace(homepage);
    }
  }
};

if ("serviceWorker" in navigator) {
  try {
    await navigator.serviceWorker.register("./sw.js", {
      type: "module",
    });
  } catch (error) {
    console.error(`Registration failed with ${error}`);
  }
}
