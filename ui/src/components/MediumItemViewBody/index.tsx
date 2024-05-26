'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import * as uuid from 'uuid'
import { useRouter } from 'next/navigation'
import Divider from '@mui/material/Divider'
import Grid from '@mui/material/Unstable_Grid2'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'

import type { ReplicaCreate } from '@/components/MediumItemImageEdit'
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
import { useDeleteReplica, useMedium, useUpdateMedium } from '@/hooks'
import type { TagTagTypeInput } from '@/hooks/types.generated'
import type { Medium, Replica } from '@/types'

import styles from './styles.module.scss'

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
    setUploading(false)
  }, [])

  const handleComplete = useCallback(async (current: Medium, replicas: (Replica | ReplicaCreate)[]) => {
    const processed = (replicas: (Replica | ReplicaCreate)[]): replicas is Replica[] => replicas.every(isReplica)
    if (!processed(replicas)) {
      setReplicas(replicas)
      updateMedium({
        id: current.id,
        replicaOrders: replicas.filter(isReplica).map(({ id }) => id),
        createdAt: current.createdAt,
      }).catch(e => {
        console.error('Error updating medium\n', e)
        setError(e)
      })
      return
    }

    setUploading(false)

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

    const newReplicas: Promise<Replica | void>[] = []
    for (const replica of replicas) {
      if (removingReplicas.some(({ id }) => id === replica.id)) {
        newReplicas.push(deleteReplica({ id: replica.id, deleteObject }).then(
          () => {},
          e => {
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
            replicaOrders: results.filter((r): r is Replica => Boolean(r)).map(({ id }) => id),
            createdAt: current.createdAt,
          })
        },
      ).then(
        newMedium => {
          closeEditSummary()
          setReplicas(newMedium.replicas)
        },
        e => {
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

  return (
    <Grid className={styles.container} container spacing={4}>
      <Grid className={styles.imageContainer} xs={9}>
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
      <Grid className={styles.metadataContainer} xs={3}>
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
          resolveMedium={() => Promise.resolve(medium)}
          replicas={replicas}
          setReplicas={setReplicas}
          close={closeUpload}
          onComplete={handleComplete}
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
