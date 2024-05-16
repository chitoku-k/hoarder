'use client'

import type { CSSProperties, FunctionComponent, ReactNode } from 'react'
import Skeleton from '@mui/material/Skeleton'
import Stack from '@mui/material/Stack'

const ImageLoading: FunctionComponent<ImageLoadingProps> = ({
  className,
  style,
  width,
  height,
  children,
}) => (
  <Stack className={className} style={style}>
    <Skeleton variant="rectangular" width={width} height={height} />
    {children}
  </Stack>
)

export interface ImageLoadingProps {
  className?: string
  style?: CSSProperties
  width?: number
  height?: number
  children?: ReactNode
}

export default ImageLoading
