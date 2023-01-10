import { sveltekit } from "@sveltejs/kit/vite";

/** @type {import('vite').UserConfig} */
const config = {
  plugins: [sveltekit()],
  server: {
    port: 4639,
    strictPort: true,
  },
  preview: {
    port: 4639,
    strictPort: true,
  },
  test: {
    include: ["src/**/*.{test,spec}.{js,ts}"],
  },
};

export default config;
