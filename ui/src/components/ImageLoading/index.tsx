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
  readonly className?: string
  readonly style?: CSSProperties
  readonly width?: number
  readonly height?: number
  readonly children?: ReactNode
}

export default ImageLoading
