'use client'

import type { CSSProperties, FunctionComponent, ReactNode } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'

import ImageError from '@/components/ImageError'
import ImageLoading from '@/components/ImageLoading'

const Image: FunctionComponent<ImageProps> = ({
  className,
  style,
  width,
  height,
  children,
}) => (
  <ErrorBoundary fallback={<ImageError className={className} style={style} width={width} height={height} />}>
    <Suspense fallback={<ImageLoading className={className} style={style} width={width} height={height} />}>
      {children}
    </Suspense>
  </ErrorBoundary>
)

export interface ImageProps {
  className?: string
  style?: CSSProperties
  width?: number
  height?: number
  children?: ReactNode
}

export default Image
