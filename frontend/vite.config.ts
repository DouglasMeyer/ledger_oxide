import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    host: true,
    proxy: {
      "/graphql": {
        target: process.env.VITE_API_URL || "http://localhost:4000",
        changeOrigin: true,
      },
    },
  },
});
