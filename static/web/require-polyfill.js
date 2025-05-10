/**
 * Simple require() polyfill for browser compatibility
 *
 * This script creates a simple CommonJS-style require() function in the browser
 * to help with compatibility issues when modules are incorrectly bundled.
 */

// Create a module cache
window.__modules = window.__modules || {};

// Polyfill for require() function
window.require = function (id) {
  // Check if the module is already in cache
  if (window.__modules[id]) {
    return window.__modules[id].exports;
  }

  // Create a new module and add it to cache
  const module = {
    id: id,
    exports: {},
    loaded: false,
  };

  window.__modules[id] = module;

  console.log(`[require polyfill] Module requested: ${id}`);

  // For known common modules, provide shims
  if (id === "path") {
    module.exports = {
      join: (...parts) => parts.join("/").replace(/\/+/g, "/"),
      resolve: (...parts) => parts.join("/").replace(/\/+/g, "/"),
      basename: (path, ext) => {
        const base = path.split("/").pop();
        return ext && base.endsWith(ext) ? base.slice(0, -ext.length) : base;
      },
      dirname: (path) => {
        const parts = path.split("/");
        parts.pop();
        return parts.join("/") || ".";
      },
      extname: (path) => {
        const i = path.lastIndexOf(".");
        return i < 0 ? "" : path.substring(i);
      },
    };
  } else if (id === "fs") {
    // Provide empty/mock implementations for node fs module
    module.exports = {
      readFileSync: () => {
        throw new Error("fs.readFileSync is not available in the browser");
      },
      writeFileSync: () => {
        throw new Error("fs.writeFileSync is not available in the browser");
      },
      existsSync: () => false,
      promises: {
        readFile: () =>
          Promise.reject(
            new Error("fs.promises.readFile is not available in the browser"),
          ),
        writeFile: () =>
          Promise.reject(
            new Error("fs.promises.writeFile is not available in the browser"),
          ),
      },
    };
  } else if (id === "process") {
    module.exports = { env: {} };
  } else if (id === "buffer") {
    module.exports = {
      Buffer: window.Buffer || { from: () => new Uint8Array() },
    };
  } else if (id === "url") {
    module.exports = { URL: window.URL };
  } else if (id === "crypto") {
    // Mock crypto with browser's crypto if available
    module.exports = window.crypto || {};
  } else {
    console.warn(`[require polyfill] Unhandled module: ${id}`);
    // For unhandled modules, return an empty object to prevent errors
    module.exports = {};
  }

  module.loaded = true;
  return module.exports;
};

// Also provide require.resolve for compatibility
window.require.resolve = (id) => id;

console.log("[require polyfill] CommonJS require() polyfill loaded");
