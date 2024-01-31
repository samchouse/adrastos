import { TanStackRouterVite } from '@tanstack/router-vite-plugin';
import react from '@vitejs/plugin-react-swc';
import { defineConfig } from 'vite';
import tsPaths from 'vite-tsconfig-paths';

export default defineConfig({
  plugins: [tsPaths(), react(), TanStackRouterVite()],
  server: {
    host: 'arch',
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
