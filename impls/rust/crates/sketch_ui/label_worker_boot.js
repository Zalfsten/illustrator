// Starts the label typesetting worker (#60). Trunk builds label_worker.js and
// label_worker_bg.wasm (no-modules target); this script loads them and turns
// a failed load into an uncaught error, which the page sees as the worker's
// `error` event (a rejected promise inside a worker would go unnoticed).
importScripts("./label_worker.js");
wasm_bindgen({ module_or_path: "./label_worker_bg.wasm" }).catch((e) => {
  setTimeout(() => {
    throw new Error("typesetter module failed to load: " + e);
  });
});
