import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv } from "vite";
import eslintPlugin from "vite-plugin-eslint";

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current working directory.
  // Set the third parameter to '' to load all env regardless of the `VITE_` prefix.
  const env = loadEnv(mode, process.cwd(), "");
  return {
    server: {
      port: Number(env.VITE_PORT),
    },
    plugins: [
      react(),
      // automatically check when `yarn build
      eslintPlugin(),
    ],
  };
});
