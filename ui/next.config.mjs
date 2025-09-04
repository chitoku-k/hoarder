/** @type {import('next').NextConfig} */
const nextConfig = {
  productionBrowserSourceMaps: true,
  reactStrictMode: true,
  experimental: {
    proxyTimeout: 300_000,
  },
  images: {
    minimumCacheTTL: 7 * 86_400,
    qualities: [ 100 ],
  },
  output: 'standalone',
  rewrites: async () => [
    {
      source: '/graphql/:path*',
      destination: `${process.env.API_URL}/graphql/:path*`,
    },
    {
      source: '/objects',
      destination: `${process.env.API_URL}/objects`,
    },
    {
      source: '/thumbnails/:id',
      destination: `${process.env.API_URL}/thumbnails/:id`,
    },
  ],
}

export default nextConfig
