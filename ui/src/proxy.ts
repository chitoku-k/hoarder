import type { NextRequest, ProxyConfig } from 'next/server'
import { NextResponse } from 'next/server'

const API_URL = process.env.API_URL
if (typeof API_URL === 'undefined') {
  throw new Error('API_URL must be set')
}

export const config = {
  matcher: [
    '/graphql/:path*',
    '/objects',
    '/thumbnails/:id',
  ],
} satisfies ProxyConfig

export default function proxy(request: NextRequest) {
  return NextResponse.rewrite(new URL(`${request.nextUrl.pathname}${request.nextUrl.search}`, API_URL))
}
