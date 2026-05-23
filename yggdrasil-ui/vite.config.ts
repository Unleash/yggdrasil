import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      env: resolve(__dirname, 'src/wasm/env.js'),
    },
  },
  server: {
    proxy: {
      '/unleash-sandbox': {
        target: 'https://sandbox.getunleash.io',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/unleash-sandbox/, ''),
        configure(proxy) {
          proxy.on('proxyReq', (proxyReq) => {
            proxyReq.setHeader('origin', 'https://sandbox.getunleash.io');
          });
        },
      },
    },
  },
});
