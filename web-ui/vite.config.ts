import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      "/api": {
        target: "http://localhost:4321",
        changeOrigin: true,
      },
    },
    // Adding historyApiFallback for the SPA router
    historyApiFallback: true,
  },
  build: {
    outDir: "../static/web",
    emptyOutDir: true,
    // Ensure assets are correctly referenced with absolute paths starting at root
    assetsDir: "assets",
    // Generate a proper base path for production - use relative paths
    base: "./",
    // Improve source map for debugging
    sourcemap: true,
    // Force ESM format and prevent require() usage
    modulePreload: {
      polyfill: true,
    },
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
      },
      output: {
        // Ensure ESM and prevent "require is not defined" errors
        format: "es",
        // Make sure we're using proper entry point naming
        entryFileNames: "assets/[name]-[hash].js",
        chunkFileNames: "assets/[name]-[hash].js",
        assetFileNames: "assets/[name]-[hash].[ext]",
      },
    },
  },
  // Make sure we're handling resolved modules correctly
  resolve: {
    extensions: [".mjs", ".js", ".ts", ".jsx", ".tsx", ".json"],
    // Explicitly exclude node built-ins to prevent require statements
    conditions: ["browser", "module", "jsnext:main", "jsnext"],
  },
  // Prevent node-specific globals and modules
  define: {
    "process.env": {},
  },
});
