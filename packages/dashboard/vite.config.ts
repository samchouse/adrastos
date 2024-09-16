import { TanStackRouterVite } from '@tanstack/router-vite-plugin';
import react from '@vitejs/plugin-react-swc';
import million from 'million/compiler';
import { defineConfig } from 'vite';
import tsPaths from 'vite-tsconfig-paths';

export default defineConfig({
  plugins: [
    tsPaths(),
    million.vite({ auto: true }),
    react(),
    TanStackRouterVite(),
  ],
  server: {
    host: '0.0.0.0',
    https: {
      cert: '../../certs/cert.pem',
      key: '../../certs/key.pem',
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          if (
            id.includes('node_modules') &&
            (id.includes('react-dom') ||
              id.includes('zod') ||
              id.includes('@radix-ui'))
          )
            return 'vendor';
        },
      },
    },
  },
});
