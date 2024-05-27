'use client'

import type { DragEvent, ReactNode } from 'react'
import { useCallback, useEffect } from 'react'

import type { Folder } from '@/hooks'
import { useFileSystemEntry } from '@/hooks'

const FileDrop = function <Flatten extends boolean | undefined>({
  className,
  signal,
  flatten,
  onSelect,
  children,
}: FileDropProps<Flatten>) {
  const readFileSystemEntry = useFileSystemEntry({ signal, flatten })

  useEffect(() => {
    const handleDragOver = (e: globalThis.DragEvent) => {
      e.preventDefault()

      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = 'none'
      }
    }

    const handleDrop = (e: globalThis.DragEvent) => {
      e.preventDefault()
    }

    window.addEventListener('dragover', handleDragOver)
    window.addEventListener('drop', handleDrop)

    return () => {
      window.removeEventListener('dragover', handleDragOver)
      window.removeEventListener('drop', handleDrop)
    }
  }, [])

  const handleDragOver = useCallback((e: DragEvent<HTMLElement>) => {
    e.preventDefault()
    e.stopPropagation()

    e.dataTransfer.dropEffect = e.dataTransfer.types.includes('Files')
      ? 'copy'
      : 'none'
  }, [])

  const handleDrop = useCallback((e: DragEvent<HTMLElement>) => {
    e.preventDefault()
    e.stopPropagation()

    const promises: Promise<FileDropEntries<Flatten>>[] = []
    for (const item of e.dataTransfer.items) {
      const entry = item.webkitGetAsEntry()
      if (entry) {
        promises.push(readFileSystemEntry(entry))
        continue
      }

      const file = item.getAsFile()
      if (file) {
        promises.push(Promise.resolve([ file ]))
        continue
      }
    }

    const all = Promise.all(promises)
    if (flatten) {
      onSelect?.(all.then(entries => entries.flat()) as Promise<FileDropEntries<Flatten>>)
    } else {
      onSelect?.(all as Promise<FileDropEntries<Flatten>>)
    }
  }, [ onSelect, readFileSystemEntry, flatten ])

  return (
    <div className={className} onDragOver={handleDragOver} onDrop={handleDrop}>
      {children}
    </div>
  )
}

export interface FileDropProps<Flatten extends boolean | undefined> {
  className?: string
  signal?: AbortSignal
  flatten?: Flatten
  onSelect?: FileDropOnSelect<Flatten>
  children?: ReactNode
}

export type FileDropEntries<Flatten> = Flatten extends true ? File[] : (File | Folder)[]

export type FileDropOnSelect<Flatten> = (entries: Promise<FileDropEntries<Flatten>>) => void

export default FileDrop
