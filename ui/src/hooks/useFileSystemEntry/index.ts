import { useCallback } from 'react'

export type Folder = (File | Folder)[]

const isDirectoryEntry = (entry: FileSystemEntry): entry is FileSystemDirectoryEntry => entry.isDirectory

const isFileEntry = (entry: FileSystemEntry): entry is FileSystemFileEntry => entry.isFile

function* walk(entries: (File | Folder)[]): Generator<File> {
  for (const entry of entries) {
    if (Array.isArray(entry)) {
      yield* walk(entry)
    } else {
      yield entry
    }
  }
}

export function useFileSystemEntry<Flatten extends boolean | undefined>(options?: UseFileSystemEntryOptions<Flatten>) {
  const { signal, flatten } = options ?? {}

  const readDirectoryEntry = useCallback(async function* (entry: FileSystemDirectoryEntry) {
    const reader = entry.createReader()
    while (true) {
      signal?.throwIfAborted()

      const value = await new Promise<FileSystemEntry[]>((resolve, reject) => reader.readEntries(resolve, reject))
      if (!value.length) {
        break
      }

      yield* value
    }
  }, [ signal ])

  const readFileEntry = useCallback(async (entry: FileSystemFileEntry) => {
    signal?.throwIfAborted()
    return await new Promise<File>((resolve, reject) => entry.file(resolve, reject))
  }, [ signal ])

  const readEntry = useCallback(async (entry: FileSystemEntry) => {
    const promises: Promise<File | Folder>[] = []

    if (isDirectoryEntry(entry)) {
      promises.push(Array.fromAsync(readDirectoryEntry(entry), readEntry))
    }

    if (isFileEntry(entry)) {
      promises.push(readFileEntry(entry))
    }

    const all = await Promise.all(promises)
    if (flatten) {
      return [ ...walk(all) ]
    }

    return all as UseFileSystemEntryReturnValue<Flatten>
  }, [ readDirectoryEntry, readFileEntry, flatten ])

  return readEntry
}

export interface UseFileSystemEntryOptions<Flatten extends boolean | undefined> {
  signal?: AbortSignal
  flatten?: Flatten
}

export type UseFileSystemEntryReturnValue<Flatten> = Flatten extends true ? File[] : (File | Folder)[]
