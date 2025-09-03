'use client'

import type { ComponentPropsWithoutRef, FunctionComponent } from 'react'
import { forwardRef, useCallback, useMemo, useState } from 'react'
import type { TableComponents } from 'react-virtuoso'
import { TableVirtuoso } from 'react-virtuoso'
import strictUriEncode from 'strict-uri-encode'
import { AxiosError, isAxiosError } from 'axios'
import { filter, from, mergeMap } from 'rxjs'
import { Observable } from '@apollo/client'
import Button from '@mui/material/Button'
import Card from '@mui/material/Card'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import Paper from '@mui/material/Paper'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Typography from '@mui/material/Typography'
import FolderIcon from '@mui/icons-material/Folder'

import AutocompleteContainer from '@/components/AutocompleteContainer'
import MediumItemFileOverwriteDialog from '@/components/MediumItemFileOverwriteDialog'
import MediumItemFileUploadDialogBodyItem from '@/components/MediumItemFileUploadDialogBodyItem'
import type { ReplicaCreate } from '@/components/MediumItemImageEdit'
import { isReplica } from '@/components/MediumItemImageEdit'
import { ReplicaPhase } from '@/graphql/types.generated'
import type { ObjectAlreadyExists, ReplicaOriginalUrlDuplicate } from '@/hooks'
import { OBJECT_ALREADY_EXISTS, REPLICA_ORIGINAL_URL_DUPLICATE, useCreateReplica, useError, useWatchMedium } from '@/hooks'
import type { Medium, Replica } from '@/types'

import styles from './styles.module.scss'

const isValidName = (name: string) => name.length > 0
const isUniqueName = (name: string, replicas: (Replica | ReplicaCreate)[]) => replicas.reduce((total, replica) => total + Number(!isReplica(replica) && replica.name === name), 0) === 1

const extractEntry = (e: ReplicaOriginalUrlDuplicate | ObjectAlreadyExists): ReplicaUploadOverwritingFile | null => {
  const entry = e.extensions.details.data.entry
  if (!entry) {
    return null
  }

  const name = entry.name
  const size = entry.metadata?.size ?? null
  const lastModified = entry.metadata?.updatedAt ? new Date(entry.metadata.updatedAt) : null
  const url = new URL('/objects', location.href)
  url.searchParams.set('url', entry.url)

  return {
    name,
    size,
    lastModified,
    url: url.toString(),
  }
}

const MediumItemFileUploadDialogBody: FunctionComponent<MediumItemFileUploadDialogBodyProps> = ({
  abortSignal,
  resolveMedium,
  replicas,
  setReplicas,
  close,
  onProgress,
  onComplete,
}) => {
  const [ createReplica ] = useCreateReplica()
  const [ watchMedium ] = useWatchMedium()
  const { graphQLError } = useError()

  const [ uploading, setUploading ] = useState(false)
  const [ uploads, setUploads ] = useState(new Map<ReplicaCreateID, ReplicaUpload>())
  const [ overwriting, setOverwriting ] = useState<ReplicaUploadOverwrite[]>([])
  const [ error, setError ] = useState<unknown>(null)

  const defaultContainer = useMemo(() => {
    let current = ''
    for (const replica of replicas) {
      if (!isReplica(replica) || !URL.canParse(replica.originalUrl)) {
        continue
      }

      const pathname = decodeURIComponent(new URL(replica.originalUrl).pathname)
      const dirpart = pathname.substring(1).substring(pathname.indexOf('/'), pathname.lastIndexOf('/') - 1)
      if (current && current !== dirpart) {
        return ''
      }

      current = dirpart
    }

    return current
  }, [ replicas ])
  const [ container, setContainer ] = useState(defaultContainer)

  const handleChangeName = useCallback((replica: ReplicaCreate, name: string) => {
    setReplicas(replicas => {
      const idx = replicas.findIndex(r => !isReplica(r) && r.tempid === replica.tempid)
      if (idx < 0) {
        return replicas
      }

      return replicas.with(idx, {
        ...replica,
        name,
      })
    })
  }, [ setReplicas ])

  const handleChangeContainer = useCallback((container: string | null) => {
    setContainer(container ?? '')
  }, [])

  const handleUploadProgress = useCallback((replica: ReplicaCreate, upload: ReplicaUpload) => {
    setUploads(uploads => {
      const current = uploads.get(replica.tempid)
      if (current?.status === 'aborted' && upload.status === 'error') {
        return uploads
      }

      return new Map(uploads).set(replica.tempid, upload)
    })
  }, [])

  const handleClickCancel = useCallback(() => {
    close()
  }, [ close ])

  const processReplicaUpload = useCallback(async (medium: Medium, replica: ReplicaCreate, observable: Observable<Replica>, overwrite?: boolean): Promise<Replica> => {
    const path = `${container}/${replica.name}`.split('/').filter(c => c.length).map(strictUriEncode).join('/')
    const file = new File([ replica.blob ], `/${path}`)

    try {
      const newReplica = await createReplica(
        {
          mediumID: medium.id,
          file,
          overwrite: Boolean(overwrite),
        },
        {
          signal: abortSignal,
          onUploadProgress: ({ loaded, total }) => {
            if (loaded < total) {
              handleUploadProgress(replica, { status: 'uploading', progress: { loaded, total } })
            } else {
              handleUploadProgress(replica, { status: 'creating' })
            }
          },
        },
      )

      const { promise, resolve, reject } = Promise.withResolvers<null>()
      const subscription = observable
        .pipe(filter(({ id }) => id === newReplica.id))
        .subscribe(replica => {
          switch (replica.status.phase) {
            case ReplicaPhase.Ready: {
              return resolve(null)
            }
            case ReplicaPhase.Error: {
              return reject()
            }
          }
        })

      try {
        await promise
      } finally {
        subscription.unsubscribe()
      }

      handleUploadProgress(replica, { status: 'done' })
      return newReplica
    } catch (e) {
      const replicaOriginalUrlDuplicate = graphQLError(e, REPLICA_ORIGINAL_URL_DUPLICATE)
      if (replicaOriginalUrlDuplicate) {
        const uploading = replica
        const existing = extractEntry(replicaOriginalUrlDuplicate)

        setOverwriting(overwriting => [
          ...overwriting,
          {
            uploading,
            existing,
          },
        ])
      }

      const objectAlreadyExists = graphQLError(e, OBJECT_ALREADY_EXISTS)
      if (objectAlreadyExists && !overwrite) {
        const uploading = replica
        const existing = extractEntry(objectAlreadyExists)

        const { promise: confirm, resolve, reject } = Promise.withResolvers<null>()

        setOverwriting(overwriting => [
          ...overwriting,
          {
            uploading,
            existing,
            onConfirm: () => resolve(null),
            onCancel: () => reject(),
          },
        ])

        try {
          handleUploadProgress(replica, { status: null })
          await confirm
        } catch {
          handleUploadProgress(replica, { status: 'aborted' })
          throw new Error('the uploading file already exists', { cause: e })
        }
        return await processReplicaUpload(medium, replica, observable, true)
      }

      if (isAxiosError(e) && e.code === AxiosError.ERR_CANCELED) {
        handleUploadProgress(replica, { status: 'aborted' })
      } else {
        handleUploadProgress(replica, { status: 'error', error: e })
      }
      throw new Error('error creating replica', { cause: e })
    }
  }, [ container, abortSignal, createReplica, graphQLError, handleUploadProgress, onProgress ])

  const handleClickUpload = useCallback(async () => {
    setUploading(true)

    let medium: Medium
    try {
      medium = await resolveMedium()
    } catch (e) {
      setError(e)
      return
    }

    onProgress?.('uploading')

    const observable = watchMedium({ id: medium.id })
      .pipe(mergeMap(({ data }) => from(data?.medium.replicas ?? [])))

    await Promise.allSettled(
      replicas.map(replica => isReplica(replica)
        ? Promise.resolve(replica)
        : processReplicaUpload(medium, replica, observable)
      ),
    ).then(results => {
      const newReplicas: (Replica | ReplicaCreate)[] = []
      for (const [ idx, result ] of results.entries()) {
        const oldReplica = replicas[idx]
        if (!oldReplica) {
          return
        }

        if (result.status === 'rejected') {
          console.error('Error uploading a file or creating a replica\n', result.reason)
          newReplicas.push(oldReplica)
        } else {
          newReplicas.push(result.value)
        }
      }

      onProgress?.('done')
      onComplete(medium, newReplicas)

      setUploading(false)
    }).finally(() => {
      const subscription = observable.subscribe(() => {
        // This seems to be a redundant subscription, but it is here to ensure that it
        // unsubscribes from the medium when no subscriptions were created.
      })
      subscription.unsubscribe()
    })
  }, [ resolveMedium, watchMedium, replicas, processReplicaUpload, onComplete ])

  const tableComputeItemKey = useCallback((_index: number, replica: ReplicaCreate) => replica.tempid, [])

  const tableComponents: TableComponents<ReplicaCreate> = useMemo(() => ({
    Scroller: forwardRef(({ ...rest }: ComponentPropsWithoutRef<'div'>, ref) => (
      <TableContainer ref={ref} component={Paper} {...rest} />
    )),
    Table: ({ ...rest }) => (
      <Table className={styles.table} {...rest} />
    ),
    TableHead: forwardRef(({ ...rest }: ComponentPropsWithoutRef<'thead'>, ref) => (
      <TableHead ref={ref} {...rest} />
    )),
    TableRow: ({ item, ...rest }) => (
      <TableRow {...rest} />
    ),
    TableBody: forwardRef(({ ...rest }: ComponentPropsWithoutRef<'tbody'>, ref) => (
      <TableBody ref={ref} {...rest} />
    )),
  }), [])

  const tableFixedHeaderContent = useCallback(() => (
    <TableRow>
      <TableCell className={styles.headerCell} variant="head" />
      <TableCell className={styles.headerCell} variant="head">ファイル名</TableCell>
      <TableCell className={styles.headerCell} variant="head">サイズ</TableCell>
      <TableCell className={styles.headerCell} variant="head">ステータス</TableCell>
    </TableRow>
  ), [])

  const tableItemContent = useCallback((_index: number, replica: ReplicaCreate) => {
    const upload = uploads.get(replica.tempid)
    return (
      <MediumItemFileUploadDialogBodyItem
        replica={replica}
        status={upload?.status ?? null}
        progress={upload?.progress ?? null}
        error={upload?.error ?? null}
        nameValidationError={!isValidName(replica.name)
          ? 'ファイル名が入力されていません'
          : !isUniqueName(replica.name, replicas)
            ? '同じファイル名は使用できません'
            : null}
        onChangeName={name => handleChangeName(replica, name)}
      />
    )
  }, [ uploads, replicas, handleChangeName ])

  const currentReplicas = replicas.filter((replica) => 'tempid' in replica)
  const currentOverwrite = overwriting[0]
  const hasValidationErrors = currentReplicas.some(replica => !isValidName(replica.name) || !isUniqueName(replica.name, replicas))

  return (
    <Stack className={styles.container}>
      <DialogContent>
        <DialogContentText>
          メディアのアップロード
        </DialogContentText>
        <TableVirtuoso
          className={styles.uploads}
          data={currentReplicas}
          increaseViewportBy={4096}
          computeItemKey={tableComputeItemKey}
          components={tableComponents}
          fixedHeaderContent={tableFixedHeaderContent}
          itemContent={tableItemContent}
        />
        <Stack spacing={2} direction="row" alignItems="center">
          <Typography flexShrink={0}>アップロード先</Typography>
          <Card className={styles.destination}>
            <AutocompleteContainer
              className={styles.containerComplete}
              variant="standard"
              fullWidth
              disableClearable
              includeInputInList
              focus
              forcePopupIcon
              disabled={uploading}
              value={container}
              icon={({ ...props }) => <FolderIcon {...props} />}
              onChange={handleChangeContainer}
              slotProps={{
                listbox: {
                  sx: {
                    maxHeight: '300px',
                  },
                },
                popper: {
                  className: styles.containerPopper,
                },
              }}
            />
          </Card>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClickCancel} autoFocus>キャンセル</Button>
        <Button onClick={handleClickUpload} loading={uploading} disabled={hasValidationErrors}>アップロード</Button>
      </DialogActions>
      {currentOverwrite ? (
        <MediumItemFileOverwriteDialog
          uploading={currentOverwrite.uploading}
          existing={currentOverwrite.existing}
          close={() => {
            currentOverwrite.onCancel?.()
            setOverwriting(overwriting => overwriting.toSpliced(0, 1))
          }}
          overwrite={currentOverwrite.onConfirm && (() => {
            currentOverwrite.onConfirm?.()
            setOverwriting(overwriting => overwriting.toSpliced(0, 1))
          })}
        />
      ) : null}
      {error ? (
        <Portal>
          <Snackbar
            open
            anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
            message="メディアを保存できませんでした"
          />
        </Portal>
      ) : null}
    </Stack>
  )
}

export interface MediumItemFileUploadDialogBodyProps {
  abortSignal?: AbortSignal
  resolveMedium: () => Promise<Medium>
  replicas: (Replica | ReplicaCreate)[]
  setReplicas: (setReplicas: (replicas: (Replica | ReplicaCreate)[]) => (Replica | ReplicaCreate)[]) => void
  close: () => void
  onProgress?: (status: UploadStatus) => void
  onComplete: (medium: Medium, replicas: (Replica | ReplicaCreate)[]) => void
}

type ReplicaCreateID = string

interface ReplicaUpload {
  status: ReplicaUploadStatus
  progress?: ReplicaUploadProgress
  error?: unknown
}

interface ReplicaUploadOverwrite {
  uploading: ReplicaCreate
  existing: ReplicaUploadOverwritingFile | null
  onConfirm?: () => void
  onCancel?: () => void
}

interface ReplicaUploadOverwritingFile {
  name: string
  size: number | null
  lastModified: Date | null
  url: string
}

export interface ReplicaUploadProgress {
  loaded: number
  total: number
}

export type UploadStatus = 'uploading' | 'done'
export type ReplicaUploadStatus = null | 'uploading' | 'creating' | 'done' | 'aborted' | 'error'

export default MediumItemFileUploadDialogBody
