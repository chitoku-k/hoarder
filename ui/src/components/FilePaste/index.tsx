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

  const handleInput = useCallback((e: FormEvent<HTMLElement>) => {
    const focusNode = document.getSelection()?.focusNode
    if (focusNode !== e.currentTarget && e.nativeEvent.constructor.name === 'InputEvent') {
      focusNode?.parentNode?.removeChild(focusNode)
    }
  }, [])

  const handleCompositionUpdate = useCallback((e: FormEvent<HTMLElement>) => {
    e.currentTarget.blur()
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
      onInput={handleInput}
      onCompositionUpdate={handleCompositionUpdate}
    >
      {children}
    </div>
  )
}

export interface FilePasteProps {
  readonly className?: string
  readonly onSelect?: (files: Promise<readonly File[]>) => void
  readonly children?: ReactNode
}

export default FilePaste
