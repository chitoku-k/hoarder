'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useMemo } from 'react'
import { useErrorBoundary } from 'react-error-boundary'

const ImageBodyBlob: FunctionComponent<ImageBodyBlobProps> = ({
  src,
  onError,
  ...props
}) => {
  const url = useMemo(() => URL.createObjectURL(src), [ src ])

  const { showBoundary } = useErrorBoundary()

  const handleError = useCallback((e: SyntheticEvent<HTMLImageElement>) => {
    onError?.(e)
    showBoundary(new Error('Error loading the image'))
  }, [ onError, showBoundary ])

  const ref = useCallback((node: HTMLImageElement | null) => {
    // Revoke object URL on clean up.
    return () => {
      if (node?.src) {
        URL.revokeObjectURL(node.src)
      }
    }
  }, [])

  return (
    // eslint-disable-next-line @next/next/no-img-element
    <img ref={ref} src={url} onError={handleError} {...props} />
  )
}

export interface ImageBodyBlobProps extends Omit<ComponentPropsWithoutRef<'img'>, 'src'> {
  src: Blob
}

export default ImageBodyBlob
