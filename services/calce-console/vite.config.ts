import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    port: 38100,
    proxy: {
      "/v1": {
        target: "http://localhost:35701",
        changeOrigin: true,
      },
      "/auth": {
        target: "http://localhost:35701",
        changeOrigin: true,
      },
    },
  },
});
