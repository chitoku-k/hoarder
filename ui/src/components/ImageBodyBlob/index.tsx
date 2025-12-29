'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback } from 'react'
import { useErrorBoundary } from 'react-error-boundary'

const ImageBodyBlob: FunctionComponent<ImageBodyBlobProps> = ({
  src,
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

  const ref = useCallback((node: HTMLImageElement) => {
    node.src = URL.createObjectURL(src)

    // Revoke object URL on clean up.
    return () => {
      URL.revokeObjectURL(node.src)
    }
  }, [ src ])

  return (
    // eslint-disable-next-line @next/next/no-img-element
    <img ref={ref} onLoad={handleLoad} onError={handleError} {...props} />
  )
}

export interface ImageBodyBlobProps extends Omit<ComponentPropsWithoutRef<'img'>, 'src'> {
  readonly src: Blob
}

export default ImageBodyBlob
