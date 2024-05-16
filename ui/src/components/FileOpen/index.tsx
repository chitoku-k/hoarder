'use client'

import type { ChangeEvent, FunctionComponent, ReactNode } from 'react'
import { useCallback } from 'react'

import styles from './styles.module.scss'

const FileOpen: FunctionComponent<FileOpenProps> = ({
  className,
  accept,
  multiple,
  onSelect,
  children,
}) => {
  const handleChange = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    if (!e.currentTarget.files) {
      return
    }

    onSelect?.(Promise.resolve([ ...e.currentTarget.files ]))
  }, [ onSelect ])

  return (
    <div className={className}>
      <input className={styles.input} type="file" onChange={handleChange} value="" accept={accept} multiple={multiple} />
      {children}
    </div>
  )
}

export interface FileOpenProps {
  className?: string
  accept?: string
  multiple?: boolean
  onSelect?: (files: Promise<File[]>) => void
  children?: ReactNode
}

export default FileOpen
