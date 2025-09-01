'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import * as uuid from 'uuid'
import { useRouter } from 'next/navigation'
import Divider from '@mui/material/Divider'
import Grid from '@mui/material/Grid'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'

import type { ReplicaCreate } from '@/components/MediumItemImageEdit'
import type { MediumItemFileUploadStatus } from '@/components/MediumItemFileUploadDialog'
import MediumItemFileUploadAbortDialog from '@/components/MediumItemFileUploadAbortDialog'
import MediumItemFileUploadDialog from '@/components/MediumItemFileUploadDialog'
import MediumItemImageEdit, { isReplica } from '@/components/MediumItemImageEdit'
import MediumItemImageList from '@/components/MediumItemImageList'
import MediumItemMetadataSourceEdit from '@/components/MediumItemMetadataSourceEdit'
import MediumItemMetadataSourceList from '@/components/MediumItemMetadataSourceList'
import MediumItemMetadataSummaryEdit from '@/components/MediumItemMetadataSummaryEdit'
import MediumItemMetadataSummaryShow from '@/components/MediumItemMetadataSummaryShow'
import MediumItemMetadataTagEdit from '@/components/MediumItemMetadataTagEdit'
import MediumItemMetadataTagList from '@/components/MediumItemMetadataTagList'
import MediumItemReplicaDeleteDialog from '@/components/MediumItemReplicaDeleteDialog'
import type { TagTagTypeInput } from '@/graphql/types.generated'
import { useBeforeUnload, useDeleteReplica, useMedium, useUpdateMedium } from '@/hooks'
import type { Medium, Replica } from '@/types'

import styles from './styles.module.scss'

const hasChanges = (medium: Medium, replicas: (Replica | ReplicaCreate)[], removingReplicas: Replica[]) => {
  if (medium.replicas?.length !== replicas.length || removingReplicas.length > 0) {
    return true
  }

  for (const [ idx, replica ] of replicas.entries()) {
    if (!isReplica(replica)) {
      return true
    }

    if (medium.replicas[idx]?.id !== replica.id) {
      return true
    }
  }

  return false
}

const MediumItemViewBody: FunctionComponent<MediumItemViewBodyProps> = ({
  id,
}) => {
  if (!uuid.validate(id)) {
    throw new Error(`medium id invalid: ${id}`)
  }

  const router = useRouter()

  const medium = useMedium({ id })
  const [ updateMedium, { loading } ] = useUpdateMedium()
  const [ deleteReplica ] = useDeleteReplica()

  const [ editingSummary, setEditingSummary ] = useState(false)
  const [ editingSources, setEditingSources ] = useState(false)
  const [ editingTags, setEditingTags ] = useState(false)

  const [ replicas, setReplicas ] = useState<(Replica | ReplicaCreate)[]>(medium.replicas)
  const [ removingReplicas, setRemovingReplicas ] = useState<Replica[]>([])
  const [ deletingObjects, setDeletingObjects ] = useState<MediumDeleteObjects | null>(null)

  const [ uploading, setUploading ] = useState(false)
  const [ uploadAborting, setUploadAborting ] = useState(false)
  const [ uploadAbortController, setUploadAbortController ] = useState(new AbortController())
  const [ uploadInProgress, setUploadInProgress ] = useState(false)

  const [ error, setError ] = useState<unknown>(null)

  const removeReplica = useCallback((replica: Replica | ReplicaCreate) => {
    setReplicas(replicas => {
      if (isReplica(replica)) {
        return replicas
      }

      const idx = replicas.findIndex(r => !isReplica(r) && r.tempid === replica.tempid)
      if (idx < 0) {
        return replicas
      }

      return replicas.toSpliced(idx, 1)
    })

    setRemovingReplicas(removingReplicas => {
      if (!isReplica(replica)) {
        return removingReplicas
      }

      const idx = removingReplicas.findIndex(({ id }) => id === replica.id)
      if (idx >= 0) {
        return removingReplicas
      }

      return [
        ...removingReplicas,
        replica,
      ]
    })
  }, [])

  const restoreReplica = useCallback((replica: Replica) => {
    setRemovingReplicas(removingReplicas => {
      const idx = removingReplicas.findIndex(({ id }) => id === replica.id)
      if (idx < 0) {
        return removingReplicas
      }

      return removingReplicas.toSpliced(idx, 1)
    })
  }, [])

  const editSummary = useCallback(() => {
    setEditingSummary(true)
  }, [])

  const closeEditSummary = useCallback(() => {
    setEditingSummary(false)
    setReplicas(medium.replicas)
    setRemovingReplicas([])
  }, [ medium ])

  const editSources = useCallback(() => {
    setEditingSources(true)
  }, [])

  const closeEditSources = useCallback(() => {
    setEditingSources(false)
  }, [])

  const editTags = useCallback(() => {
    setEditingTags(true)
  }, [])

  const closeEditTags = useCallback(() => {
    setEditingTags(false)
  }, [])

  const closeUpload = useCallback(() => {
    if (uploadInProgress && !uploadAborting) {
      setUploadAborting(true)
    } else {
      setUploading(false)
      setUploadAborting(false)
      setUploadAbortController(uploadAbortController => {
        uploadAbortController.abort()
        return new AbortController()
      })
    }
  }, [ uploadAborting, uploadInProgress ])

  const closeUploadAbort = useCallback(() => {
    setUploadAborting(false)
  }, [])

  const handleProgress = useCallback((status: MediumItemFileUploadStatus) => {
    setUploadInProgress(status === 'uploading')

    if (status === 'done') {
      setUploadAbortController(new AbortController())
    }
  }, [])

  const handleComplete = useCallback(async (current: Medium, replicas: (Replica | ReplicaCreate)[]) => {
    const processed = (replicas: (Replica | ReplicaCreate)[]) => replicas.every(isReplica)
    if (!processed(replicas)) {
      setReplicas(replicas)
      updateMedium({
        id: current.id,
        replicaOrders: replicas.filter(isReplica).map(({ id }) => id),
        createdAt: current.createdAt,
      }).catch((e: unknown) => {
        console.error('Error updating medium\n', e)
        setError(e)
      })
      return
    }

    setUploading(false)
    setUploadAborting(false)

    let deleteObject: boolean | null = null
    if (removingReplicas.length) {
      const { promise: confirm, resolve: onConfirm, reject: onCancel } = Promise.withResolvers<boolean>()
      setDeletingObjects({ onConfirm, onCancel })

      try {
        deleteObject = await confirm
      } catch {
        return
      }
    }

    const newReplicas: Promise<Replica | null>[] = []
    for (const replica of replicas) {
      if (removingReplicas.some(({ id }) => id === replica.id)) {
        newReplicas.push(deleteReplica({ id: replica.id, deleteObject }).then(
          () => null,
          (e: unknown) => {
            throw new Error('error deleting replica', { cause: e })
          },
        ))
      } else {
        newReplicas.push(Promise.resolve(replica))
      }
    }

    await Promise.all(newReplicas)
      .then(
        results => {
          return updateMedium({
            id: current.id,
            replicaOrders: results.filter(r => r !== null).map(({ id }) => id),
            createdAt: current.createdAt,
          })
        },
      ).then(
        newMedium => {
          closeEditSummary()
          setReplicas(newMedium.replicas)
        },
        (e: unknown) => {
          console.error('Error updating medium\n', e)
          setError(e)
        },
      )
  }, [ updateMedium, deleteReplica, removingReplicas, closeEditSummary ])

  const save = useCallback((current: Medium) => {
    if (replicas.some(r => !isReplica(r))) {
      setUploading(true)
    } else {
      handleComplete(current, replicas)
    }
  }, [ replicas, handleComplete ])

  const saveTags = useCallback((id: string, addTagTagTypeIDs: TagTagTypeInput[], removeTagTagTypeIDs: TagTagTypeInput[]) => updateMedium({
    id,
    addTagTagTypeIDs,
    removeTagTagTypeIDs,
  }), [ updateMedium ])

  const saveSources = useCallback((id: string, addSourceIDs: string[], removeSourceIDs: string[]) => updateMedium({
    id,
    addSourceIDs,
    removeSourceIDs,
  }), [ updateMedium ])

  const handleDeleteMedium = useCallback(() => {
    router.replace('/')
  }, [ router ])

  const changed = hasChanges(medium, replicas, removingReplicas)
  useBeforeUnload(changed)

  return (
    <Grid className={styles.container} container spacing={4} flexGrow={1}>
      <Grid size={9}>
        {editingSummary ? (
          <MediumItemImageEdit
            className={styles.imageList}
            gap={32}
            replicas={replicas}
            setReplicas={setReplicas}
            removingReplicas={removingReplicas}
            removeReplica={removeReplica}
            restoreReplica={restoreReplica}
          />
        ) : (
          <MediumItemImageList
            className={styles.imageList}
            gap={32}
            replicas={medium.replicas}
          />
        )}
      </Grid>
      <Grid className={styles.metadataContainer} size={3}>
        <Stack className={styles.metadataList} divider={<Divider />} spacing={4}>
          {editingSummary ? (
            <MediumItemMetadataSummaryEdit
              loading={loading}
              medium={medium}
              save={save}
              close={closeEditSummary}
              onDelete={handleDeleteMedium}
            />
          ) : (
            <MediumItemMetadataSummaryShow medium={medium} edit={editSummary} />
          )}
          {editingTags ? (
            <MediumItemMetadataTagEdit medium={medium} focus loading={loading} save={saveTags} close={closeEditTags} />
          ) : (
            <MediumItemMetadataTagList medium={medium} edit={editTags} />
          )}
          {editingSources ? (
            <MediumItemMetadataSourceEdit medium={medium} focus loading={loading} save={saveSources} close={closeEditSources} />
          ) : (
            <MediumItemMetadataSourceList medium={medium} edit={editSources} />
          )}
        </Stack>
      </Grid>
      {uploading ? (
        <MediumItemFileUploadDialog
          abortSignal={uploadAbortController.signal}
          resolveMedium={() => Promise.resolve(medium)}
          replicas={replicas}
          setReplicas={setReplicas}
          close={closeUpload}
          onProgress={handleProgress}
          onComplete={handleComplete}
        />
      ) : null}
      {uploadAborting ? (
        <MediumItemFileUploadAbortDialog
          close={closeUploadAbort}
          abort={closeUpload}
        />
      ) : null}
      {deletingObjects ? (
        <MediumItemReplicaDeleteDialog
          close={() => {
            deletingObjects.onCancel()
            setDeletingObjects(null)
          }}
          save={result => {
            deletingObjects.onConfirm(result)
            setDeletingObjects(null)
          }}
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
    </Grid>
  )
}

export interface MediumItemViewBodyProps {
  id: string
}

interface MediumDeleteObjects {
  onConfirm: (result: boolean) => void
  onCancel: () => void
}

export default MediumItemViewBody
