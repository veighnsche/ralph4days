import React from "react";
import ReactDOM from "react-dom/client";
import { ThemeProvider } from "@/lib/theme-provider";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark">
      <App />
    </ThemeProvider>
  </React.StrictMode>
);
