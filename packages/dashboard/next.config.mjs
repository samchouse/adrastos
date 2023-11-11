import { env } from './src/env.mjs';

/** @type {import('next').NextConfig} */
const nextConfig = {
  eslint: {
    ignoreDuringBuilds: true,
  },
  rewrites: () => [
    {
      source: '/home',
      destination: '/',
    },
    {
      source: '/api/:path*',
      destination: `${env.NEXT_PUBLIC_BACKEND_URL}/api/:path*`,
    },
  ],
};

export default nextConfig;
