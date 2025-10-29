import type { NextConfig } from 'next'

const API_URL = process.env.API_URL
if (typeof API_URL === 'undefined') {
  throw new Error('API_URL must be set')
}

const nextConfig = {
  productionBrowserSourceMaps: true,
  experimental: {
    proxyTimeout: 300_000,
  },
  images: {
    minimumCacheTTL: 7 * 86_400,
    qualities: [ 100 ],
  },
  output: 'standalone',
  rewrites: () => [
    {
      source: '/graphql/:path*',
      destination: `${API_URL}/graphql/:path*`,
    },
    {
      source: '/objects',
      destination: `${API_URL}/objects`,
    },
    {
      source: '/thumbnails/:id',
      destination: `${API_URL}/thumbnails/:id`,
    },
  ],
} satisfies NextConfig

export default nextConfig
