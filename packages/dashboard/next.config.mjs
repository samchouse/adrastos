import { env } from './src/env.mjs';

/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: true
  },
  rewrites: async () => [
    {
      source: '/home',
      destination: '/'
    },
    {
      source: '/api/:path*',
      destination: `${env.NEXT_PUBLIC_BACKEND_URL}/:path*`
    }
  ]
};

export default nextConfig;
