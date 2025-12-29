'use client'

import type { FunctionComponent, SyntheticEvent } from 'react'
import { useCallback } from 'react'
import { useErrorBoundary } from 'react-error-boundary'
import type { ImageProps } from 'next/image'
import Image from 'next/image'

const ImageBodyNext: FunctionComponent<ImageBodyNextProps> = ({
  onLoad,
  onError,
  ...props
}) => {
  const { showBoundary } = useErrorBoundary()

  const showError = useCallback((e?: unknown) => {
    showBoundary(new Error('Error loading the image', { cause: e }))
  }, [ showBoundary ])

  const handleLoad = useCallback((e: SyntheticEvent<HTMLImageElement>) => {
    onLoad?.(e)
    e.currentTarget.decode().catch(showError)
  }, [ onLoad, showError ])

  const handleError = useCallback((e: SyntheticEvent<HTMLImageElement>) => {
    onError?.(e)
    showError()
  }, [ onError, showError ])

  return (
    <Image onLoad={handleLoad} onError={handleError} {...props} />
  )
}

export type ImageBodyNextProps = ImageProps

export default ImageBodyNext
