const getRemotePatterns = env => {
  if (!URL.canParse(env)) {
    return []
  }

  const { hostname, port, pathname } = new URL(env)
  return [
    {
      protocol: 'http',
      hostname,
      port,
      pathname: pathname.replace(/\/?$/, '/**'),
    },
    {
      protocol: 'https',
      hostname,
      port,
      pathname: pathname.replace(/\/?$/, '/**'),
    },
  ]
}

/** @type {import('next').NextConfig} */
const nextConfig = {
  productionBrowserSourceMaps: true,
  reactStrictMode: true,
  experimental: {
    proxyTimeout: 300_000,
  },
  images: {
    minimumCacheTTL: 7 * 86_400,
    remotePatterns: [
      ...getRemotePatterns(process.env.PUBLIC_URL),
    ],
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
