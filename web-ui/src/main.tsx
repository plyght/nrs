import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import "./index.css";

// Add debugging to help troubleshoot
console.log("Starting React application...");
console.log("Environment:", import.meta.env);

// Check if the root element exists
const rootElement = document.getElementById("root");
if (!rootElement) {
  console.error(
    "Root element not found! Check if the index.html contains div#root",
  );
  // Create one if it doesn't exist (fallback)
  const newRoot = document.createElement("div");
  newRoot.id = "root";
  document.body.appendChild(newRoot);
  console.log("Created new root element as fallback");
}

try {
  ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </React.StrictMode>,
  );
  console.log("React app rendered successfully");
} catch (error) {
  console.error("Failed to render React app:", error);
  // Show a visible error on the page
  document.body.innerHTML = `
    <div style="padding: 20px; font-family: sans-serif; color: #333;">
      <h1 style="color: #d32f2f;">React Rendering Error</h1>
      <p>There was a problem loading the app: ${error instanceof Error ? error.message : String(error)}</p>
      <pre style="background: #f5f5f5; padding: 10px; border-radius: 4px; overflow: auto;">${error instanceof Error ? error.stack : "No stack trace available"}</pre>
    </div>
  `;
}
