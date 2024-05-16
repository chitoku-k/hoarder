'use client'

import type { FunctionComponent, SyntheticEvent } from 'react'
import { useCallback } from 'react'
import { useErrorBoundary } from 'react-error-boundary'
import type { ImageProps } from 'next/image'
import Image from 'next/image'

const ImageBodyNext: FunctionComponent<ImageBodyNextProps> = ({
  onError,
  ...props
}) => {
  const { showBoundary } = useErrorBoundary()

  const handleError = useCallback((e: SyntheticEvent<HTMLImageElement>) => {
    onError?.(e)
    showBoundary(new Error('Error loading the image'))
  }, [ onError, showBoundary ])

  return (
    <Image onError={handleError} {...props} />
  )
}

export type ImageBodyNextProps = ImageProps

export default ImageBodyNext
