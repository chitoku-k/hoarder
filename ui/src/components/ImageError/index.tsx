'use client'

import type { CSSProperties, FunctionComponent, ReactNode } from 'react'
import clsx from 'clsx'
import Stack from '@mui/material/Stack'
import ImageNotSupportedIcon from '@mui/icons-material/ImageNotSupported'

import styles from './styles.module.scss'

const ImageError: FunctionComponent<ImageErrorProps> = ({
  className,
  style,
  width,
  height,
  children,
}) => (
  <Stack className={clsx(styles.container, className)} style={style}>
    <Stack alignItems="center" justifyContent="center" width={width} height={height}>
      <ImageNotSupportedIcon className={styles.icon} />
    </Stack>
    {children}
  </Stack>
)

export interface ImageErrorProps {
  className?: string
  style?: CSSProperties
  width?: number
  height?: number
  children?: ReactNode
}

export default ImageError
