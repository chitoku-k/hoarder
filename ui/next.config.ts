import type { NextConfig } from 'next'

const nextConfig = {
  productionBrowserSourceMaps: true,
  experimental: {
    proxyClientMaxBodySize: 1024 ** 3,
    proxyTimeout: 300_000,
  },
  images: {
    minimumCacheTTL: 7 * 86_400,
    qualities: [ 100 ],
  },
  output: 'standalone',
} satisfies NextConfig

export default nextConfig
