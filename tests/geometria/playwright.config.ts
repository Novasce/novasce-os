import { defineConfig } from "@playwright/test";

const raiz = new URL("../..", import.meta.url).pathname;

export default defineConfig({
  testDir: ".",
  use: { viewport: { width: 1060, height: 576 } },
  webServer: [
    { command: "npm exec -w apps/comecar -- vite --port 4310 --strictPort", cwd: raiz, url: "http://localhost:4310", env: { VITE_PONTE: "falsa" } },
    { command: "npm exec -w apps/ajuda -- vite --port 4311 --strictPort", cwd: raiz, url: "http://localhost:4311", env: { VITE_PONTE: "falsa" } },
  ],
});
