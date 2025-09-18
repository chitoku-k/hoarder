'use client'

import type { FunctionComponent, Ref } from 'react'
import { useCallback, useMemo, useState } from 'react'
import type { Components } from 'react-virtuoso'
import { Virtuoso } from 'react-virtuoso'
import clsx from 'clsx'
import { v4 as uuid } from 'uuid'
import type { ISize } from 'image-size/types/interface'
import Button from '@mui/material/Button'
import IconButton from '@mui/material/IconButton'
import ImageList from '@mui/material/ImageList'
import ImageListItem from '@mui/material/ImageListItem'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import AddPhotoAlternateOutlinedIcon from '@mui/icons-material/AddPhotoAlternateOutlined'

import FileDrop from '@/components/FileDrop'
import FileOpen from '@/components/FileOpen'
import FilePaste from '@/components/FilePaste'
import MediumItemFileAppendDialog from '@/components/MediumItemFileAppendDialog'
import MediumItemFileErrorDialog from '@/components/MediumItemFileErrorDialog'
import MediumItemImageItem from '@/components/MediumItemImageItem'
import MediumItemImageItemBar from '@/components/MediumItemImageItemBar'
import type { Replica } from '@/types'

import styles from './styles.module.scss'

export const isReplica = (replica: Replica | ReplicaCreate) => 'id' in replica

const FILE_MAX_INPUT_SIZE = 512 * 1024
const FILE_APPEND_CONFIRM_DIALOG_THRESHOLD = 10

const MediumItemImageEdit: FunctionComponent<MediumItemImageEditProps> = ({
  className,
  gap,
  replicas,
  setReplicas,
  removingReplicas,
  removeReplica,
  restoreReplica,
}) => {
  const [ abortController, setAbortController ] = useState(new AbortController())
  const [ appending, setAppending ] = useState(0)
  const [ appended, setAppended ] = useState(0)
  const [ appendFiles, setAppendFiles ] = useState(new Map<string, Promise<File[]>>())
  const [ error, setError ] = useState<unknown>(null)
  const [ errorFiles, setErrorFiles ] = useState<File[]>([])

  const handleChangeName = useCallback((idx: number, name: string) => {
    setReplicas(replicas => {
      const replica = replicas[idx]
      if (!replica || isReplica(replica)) {
        return replicas
      }

      return replicas.with(idx, {
        ...replica,
        name,
      })
    })
  }, [ setReplicas ])

  const handleMoveUp = useCallback((idx: number) => {
    setReplicas(replicas => idx === 0 ? replicas : [
      ...replicas.slice(0, idx - 1),
      ...replicas.slice(idx, idx + 1),
      ...replicas.slice(idx - 1, idx),
      ...replicas.slice(idx + 1),
    ])
  }, [ setReplicas ])

  const handleMoveDown = useCallback((idx: number) => {
    setReplicas(replicas => idx === replicas.length - 1 ? replicas : [
      ...replicas.slice(0, idx),
      ...replicas.slice(idx + 1, idx + 2),
      ...replicas.slice(idx, idx + 1),
      ...replicas.slice(idx + 2),
    ])
  }, [ setReplicas ])

  const handleRemove = useCallback((replica: Replica | ReplicaCreate) => {
    removeReplica(replica)
  }, [ removeReplica ])

  const handleRestore = useCallback((replica: Replica) => {
    restoreReplica?.(replica)
  }, [ restoreReplica ])

  const handleAppendFiles = useCallback(async (files: File[]) => {
    const { signal } = abortController
    const rejectOnAbort = new Promise<never>((_resolve, reject) => {
      signal.addEventListener('abort', reject, { once: true })
    })

    setAppending(files.length)

    const results = await Promise.all(files.map(async (file): Promise<ReplicaMetadataResult> => {
      try {
        const blob = file.slice(0, FILE_MAX_INPUT_SIZE)
        const buffer = await Promise.race([ blob.arrayBuffer(), rejectOnAbort ])
        const { promise, resolve, reject } = Promise.withResolvers<ISize>()

        const worker = new Worker(new URL('./worker.ts', import.meta.url))
        worker.addEventListener('message', (e: MessageEvent<ISize>) => resolve(e.data))
        worker.addEventListener('error', reject)
        worker.postMessage(buffer, [ buffer ])

        const size = await promise
        const [ width, height ] = !size.orientation || size.orientation <= 4
          ? [ size.width, size.height ]
          : [ size.height, size.width ]
        return {
          status: 'succeeded',
          value: {
            tempid: uuid(),
            name: file.name,
            size: file.size,
            width,
            height,
            lastModified: new Date(file.lastModified),
            blob: new Blob([ await file.arrayBuffer() ]),
          },
        }
      } catch (e) {
        console.warn('Error reading a file\n', file, e)
        return {
          status: 'failed',
          file,
        }
      }
    }))

    setAppending(0)

    if (signal.aborted) {
      return
    }

    const newReplicas: ReplicaCreate[] = []
    const newErrorFiles: File[] = []

    for (const result of results) {
      if (result.status === 'succeeded') {
        newReplicas.push(result.value)
      } else {
        newErrorFiles.push(result.file)
      }
    }

    setAppended(newReplicas.length)
    setErrorFiles(errorFiles => [
      ...errorFiles,
      ...newErrorFiles,
    ])
    setReplicas(replicas => [
      ...replicas,
      ...newReplicas,
    ])
  }, [ abortController, setReplicas ])

  const handleCloseAppendFiles = useCallback((id: string) => {
    setAppendFiles(appendFiles => {
      const newAppendFiles = new Map(appendFiles)
      newAppendFiles.delete(id)
      return newAppendFiles
    })
  }, [])

  const handleSelectFiles = useCallback((entries: Promise<File[]>) => {
    const id = uuid()
    const newAppendFiles = (async () => {
      try {
        const files = await entries
        if (files.length > FILE_APPEND_CONFIRM_DIALOG_THRESHOLD) {
          return files
        }

        void handleAppendFiles(files)
        handleCloseAppendFiles(id)
        return []
      } catch (e) {
        console.error('Error appending selected files\n', e)

        if (!(e instanceof Error) || e.name !== 'AbortError') {
          setError(e)
        }

        handleCloseAppendFiles(id)
        return []
      }
    })()

    setAppendFiles(appendFiles => new Map([ ...appendFiles.set(id, newAppendFiles) ]))
  }, [ handleAppendFiles, handleCloseAppendFiles ])

  const handleCancelAppendFiles = useCallback(() => {
    abortController.abort()
    setAbortController(new AbortController())
  }, [ abortController ])

  const handleCloseError = useCallback(() => {
    setError(null)
    setErrorFiles([])
  }, [])

  const handleCloseAppended = useCallback(() => {
    setAppended(0)
  }, [])

  const computeItemKey = useCallback((_index: number, { current }: ReplicaItem) => isReplica(current) ? current.id : current.tempid, [])

  const components: Components<ReplicaItem> = useMemo(() => ({
    List: ({ children, ref, ...rest }) => (
      <ImageList
        ref={ref as Ref<HTMLUListElement>}
        className={clsx(styles.imageList, className)}
        gap={gap}
        cols={1}
        {...rest}
      >
        {children ?? []}
      </ImageList>
    ),
    Item: ({ item, ...rest }) => (
      <ImageListItem
        className={styles.imageListItem}
        sx={{
          height: typeof item.current.height === 'number' && Number.isFinite(item.current.height)
            ? `min(100%, ${item.current.height.toString()}px) !important`
            : null,
        }}
        {...rest}
      />
    ),
  }), [ className, gap ])

  const itemContent = useCallback((index: number, item: ReplicaItem) => isReplica(item.current) ? (
    <MediumItemImageItem className={clsx(styles.imageItem, item.removing && styles.removingImageItem)} replica={item.current} fixed>
      <MediumItemImageItemBar
        index={index}
        total={item.total}
        currentIndex={item.currentIndex}
        currentTotal={item.currentTotal}
        removing={item.removing}
        replica={item.current}
        onMoveUp={handleMoveUp}
        onMoveDown={handleMoveDown}
        onRemove={handleRemove}
        onRestore={handleRestore}
      />
    </MediumItemImageItem>
  ) : (
    <MediumItemImageItem className={styles.imageItem} replica={item.current} fixed>
      <MediumItemImageItemBar
        index={index}
        total={item.total}
        currentIndex={item.currentIndex}
        currentTotal={item.currentTotal}
        removing={item.removing}
        replica={item.current}
        name={item.current.name}
        onChangeName={handleChangeName}
        onMoveUp={handleMoveUp}
        onMoveDown={handleMoveDown}
        onRemove={handleRemove}
      />
    </MediumItemImageItem>
  ), [ handleChangeName, handleMoveUp, handleMoveDown, handleRestore, handleRemove ])

  const itemSize = useCallback((el: HTMLElement) => el.getBoundingClientRect().height, [])

  const currentReplicas = replicas.filter(r => !isReplica(r) || !removingReplicas?.some(({ id }) => id === r.id))
  const currentItems = replicas.map((current): ReplicaItem => ({
    total: replicas.length,
    current,
    currentIndex: currentReplicas.indexOf(current),
    currentTotal: replicas.length - (removingReplicas?.length ?? 0),
    removing: isReplica(current) && !currentReplicas.includes(current),
  }))

  const [ currentAppend ] = appendFiles

  return (
    <Stack className={styles.container}>
      <Virtuoso
        className={styles.imageListContainer}
        data={currentItems}
        initialItemCount={currentItems.length}
        increaseViewportBy={4096}
        computeItemKey={computeItemKey}
        components={components}
        itemContent={itemContent}
        itemSize={itemSize}
        useWindowScroll
      />
      <Stack className={styles.addButtonContainer}>
        <IconButton className={styles.addButton} component="label">
          <FilePaste className={styles.addArea} onSelect={handleSelectFiles}>
            <FileDrop className={styles.addArea} onSelect={handleSelectFiles} signal={abortController.signal} flatten>
              <FileOpen className={styles.addArea} accept="image/*" multiple onSelect={handleSelectFiles} />
            </FileDrop>
          </FilePaste>
          <AddPhotoAlternateOutlinedIcon color="inherit" fontSize="inherit" />
        </IconButton>
      </Stack>
      {error ? (
        <MediumItemFileErrorDialog error={error} close={handleCloseError} />
      ) : errorFiles.length ? (
        <MediumItemFileErrorDialog files={errorFiles} close={handleCloseError} />
      ) : currentAppend ? (
        <MediumItemFileAppendDialog
          entries={currentAppend[1]}
          onAppend={handleAppendFiles}
          close={() => handleCloseAppendFiles(currentAppend[0])}
          cancel={handleCancelAppendFiles}
        />
      ) : null}
      {appending ? (
        <Portal>
          <Snackbar
            open
            anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
            message={`${appending.toString()} 件のメディアを追加しています...`}
            action={
              <Button color="secondary" onClick={handleCancelAppendFiles}>
                キャンセル
              </Button>
            }
          />
        </Portal>
      ) : appended ? (
        <Portal>
          <Snackbar
            open
            onClose={handleCloseAppended}
            anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
            message={`${appended.toString()} 件のメディアを追加しました`}
            autoHideDuration={3000}
          />
        </Portal>
      ) : null}
    </Stack>
  )
}

interface ReplicaItem {
  total: number
  current: Replica | ReplicaCreate
  currentIndex: number
  currentTotal: number
  removing: boolean
}

export interface ReplicaCreate {
  tempid: string
  name: string
  size: number
  width?: number
  height?: number
  lastModified: Date
  blob: Blob
}

type ReplicaMetadataResult = {
  status: 'succeeded'
  value: ReplicaCreate
} | {
  status: 'failed'
  file: File
}

export interface MediumItemImageEditProps {
  className?: string
  gap?: number
  replicas: (Replica | ReplicaCreate)[]
  setReplicas: (setReplicas: (replicas: (Replica | ReplicaCreate)[]) => (Replica | ReplicaCreate)[]) => void
  removingReplicas?: Replica[]
  removeReplica: (replica: Replica | ReplicaCreate) => void
  restoreReplica?: (replica: Replica) => void
}

export default MediumItemImageEdit
