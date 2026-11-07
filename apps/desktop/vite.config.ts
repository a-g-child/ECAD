import { defineConfig } from 'vite';
export default defineConfig({
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    port: 1420,
    strictPort: true,
    proxy: { '/bridge': 'http://127.0.0.1:3838' },
  },
  build: { target: 'es2022' },
});
