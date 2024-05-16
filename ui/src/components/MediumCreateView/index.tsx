'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
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
import MediumItemMetadataSourceCreate from '@/components/MediumItemMetadataSourceCreate'
import MediumItemMetadataSourceEdit from '@/components/MediumItemMetadataSourceEdit'
import MediumItemMetadataSourceList from '@/components/MediumItemMetadataSourceList'
import MediumItemMetadataSummaryCreate from '@/components/MediumItemMetadataSummaryCreate'
import MediumItemMetadataSummaryEdit from '@/components/MediumItemMetadataSummaryEdit'
import MediumItemMetadataSummaryShow from '@/components/MediumItemMetadataSummaryShow'
import MediumItemMetadataTagCreate from '@/components/MediumItemMetadataTagCreate'
import MediumItemMetadataTagEdit from '@/components/MediumItemMetadataTagEdit'
import MediumItemMetadataTagList from '@/components/MediumItemMetadataTagList'
import { useCreateMedium, useDeleteReplica, useUpdateMedium } from '@/hooks'
import type { TagTagTypeInput } from '@/hooks/types.generated'
import type { Medium, Replica } from '@/types'

import styles from './styles.module.scss'

const MediumCreateView: FunctionComponent = () => {
  const router = useRouter()

  const [ medium, setMedium ] = useState<Medium | null>(null)
  const [ resolveMedium, setResolveMedium ] = useState(() => () => Promise.reject<Medium>())
  const [ createMedium, { loading: createLoading } ] = useCreateMedium()
  const [ updateMedium, { loading: updateLoading } ] = useUpdateMedium()
  const [ deleteReplica ] = useDeleteReplica()

  const [ editingSummary, setEditingSummary ] = useState(true)
  const [ editingSources, setEditingSources ] = useState(true)
  const [ editingTags, setEditingTags ] = useState(true)

  const [ resolveSourceIDs, setResolveSourceIDs ] = useState(() => () => Promise.resolve<string[]>([]))
  const [ tagTagTypeIDs, setTagTagTypeIDs ] = useState<TagTagTypeInput[]>([])
  const [ replicas, setReplicas ] = useState<(Replica | ReplicaCreate)[]>([])
  const [ removingReplicas, setRemovingReplicas ] = useState<Replica[]>([])
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

  const closeEditSummary = useCallback((newReplicas?: Replica[]) => {
    setEditingSummary(false)
    setReplicas(() => newReplicas ?? medium?.replicas ?? [])
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

  const handleComplete = useCallback((current: Medium, replicas: (Replica | ReplicaCreate)[]) => {
    const processed = (replicas: (Replica | ReplicaCreate)[]): replicas is Replica[] => replicas.every(isReplica)
    if (!processed(replicas)) {
      setReplicas(replicas)
      updateMedium({
        id: current.id,
        replicaOrders: replicas.filter(isReplica).map(({ id }) => id),
        createdAt: current.createdAt,
      }).then(
        newMedium => {
          setMedium(newMedium)
        },
        e => {
          console.error('Error updating medium\n', e)
          setMedium(current)
          setError(e)
        },
      )
      return
    }

    setUploading(false)

    const newReplicas: Promise<Replica | void>[] = []
    for (const replica of replicas) {
      if (removingReplicas.some(({ id }) => id === replica.id)) {
        newReplicas.push(deleteReplica({ id: replica.id }).then(
          () => {},
          e => {
            throw new Error('error deleting replica', { cause: e })
          },
        ))
      } else {
        newReplicas.push(Promise.resolve(replica))
      }
    }

    Promise.all(newReplicas)
      .then(
        results => {
          return updateMedium({
            id: current.id,
            replicaOrders: results.filter((r): r is Replica => Boolean(r)).map(({ id }) => id),
            createdAt: current.createdAt,
          })
        },
      ).then(
        () => {
          router.refresh()
        },
        e => {
          console.error('Error updating medium\n', e)
          setMedium({
            ...current,
            replicas,
          })
          setError(e)
        },
      )
  }, [ updateMedium, deleteReplica, removingReplicas, router ])

  const save = useCallback(async (current: MediumCreate) => {
    const newResolveMedium = async () => {
      const newMedium = await createMedium({
        sourceIDs: await resolveSourceIDs(),
        tagTagTypeIDs,
        createdAt: current.createdAt,
      })
      history.replaceState(null, '', `/media/${newMedium.id}`)
      setResolveMedium(() => () => Promise.resolve(newMedium))
      return newMedium
    }

    try {
      if (replicas.some(r => !isReplica(r))) {
        setResolveMedium(resolveMedium => medium ? resolveMedium : newResolveMedium)
        setUploading(true)
      } else {
        const newMedium = medium ?? await newResolveMedium()
        handleComplete(newMedium, replicas)
      }
    } catch (e) {
      console.error('Error creating medium\n', e)
      setError(e)
    }
  }, [ medium, resolveSourceIDs, tagTagTypeIDs, replicas, createMedium, handleComplete ])

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

  const loading = createLoading || updateLoading

  return (
    <Grid className={styles.container} container spacing={4}>
      <Grid className={styles.imageContainer} xs={9}>
        {!medium || editingSummary ? (
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
            replicas={medium.replicas ?? []}
          />
        )}
      </Grid>
      <Grid className={styles.metadataContainer} xs={3}>
        <Stack className={styles.metadataList} divider={<Divider />} spacing={4}>
          {!medium ? (
            <MediumItemMetadataSummaryCreate loading={loading} save={save} />
          ) : editingSummary ? (
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
          {!medium ? (
            <MediumItemMetadataTagCreate loading={loading} setTagTagTypeIDs={setTagTagTypeIDs} />
          ) : editingTags ? (
            <MediumItemMetadataTagEdit medium={medium} loading={loading} save={saveTags} close={closeEditTags} />
          ) : (
            <MediumItemMetadataTagList medium={medium} edit={editTags} />
          )}
          {!medium ? (
            <MediumItemMetadataSourceCreate loading={loading} setResolveSourceIDs={setResolveSourceIDs} />
          ) : editingSources ? (
            <MediumItemMetadataSourceEdit medium={medium} loading={loading} save={saveSources} close={closeEditSources} />
          ) : (
            <MediumItemMetadataSourceList medium={medium} edit={editSources} />
          )}
        </Stack>
      </Grid>
      {uploading ? (
        <MediumItemFileUploadDialog
          resolveMedium={resolveMedium}
          replicas={replicas}
          setReplicas={setReplicas}
          close={closeUpload}
          onComplete={handleComplete}
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

export interface MediumCreate {
  createdAt: string | null
}

export default MediumCreateView
