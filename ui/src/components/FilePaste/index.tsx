'use client'

import type { ClipboardEvent, DragEvent, FormEvent, FunctionComponent, ReactNode } from 'react'
import { useCallback, useEffect, useState } from 'react'
import clsx from 'clsx'

import styles from './styles.module.scss'

const FilePaste: FunctionComponent<FilePasteProps> = ({
  className,
  onSelect,
  children,
}) => {
  useEffect(() => {
    const handlePaste = (e: globalThis.ClipboardEvent) => {
      if (e.clipboardData?.files.length) {
        onSelect?.(Promise.resolve([ ...e.clipboardData.files ]))
      }
    }

    window.addEventListener('paste', handlePaste)

    return () => {
      window.removeEventListener('paste', handlePaste)
    }
  }, [ onSelect ])

  const [ contentEditable, setContentEditable ] = useState(true)

  const handleClick = useCallback(() => {
    setContentEditable(false)
    requestAnimationFrame(() => {
      setContentEditable(true)
    })
  }, [])

  const handleDrop = useCallback((e: DragEvent<HTMLElement>) => {
    e.preventDefault()
  }, [])

  const handleCut = useCallback((e: ClipboardEvent<HTMLElement>) => {
    e.preventDefault()
  }, [])

  const handleCopy = useCallback((e: ClipboardEvent<HTMLElement>) => {
    e.preventDefault()
  }, [])

  const handlePaste = useCallback((e: ClipboardEvent<HTMLElement>) => {
    e.preventDefault()
    e.stopPropagation()
    onSelect?.(Promise.resolve([ ...e.clipboardData.files ]))
  }, [ onSelect ])

  const handleBeforeInput = useCallback((e: FormEvent<HTMLElement>) => {
    e.preventDefault()
  }, [])

  return (
    <div
      className={clsx(styles.container, className)}
      contentEditable={contentEditable}
      suppressContentEditableWarning
      onClick={handleClick}
      onDrop={handleDrop}
      onCut={handleCut}
      onCopy={handleCopy}
      onPaste={handlePaste}
      onBeforeInput={handleBeforeInput}
    >
      {children}
    </div>
  )
}

export interface FilePasteProps {
  className?: string
  onSelect?: (files: Promise<File[]>) => void
  children?: ReactNode
}

export default FilePaste
