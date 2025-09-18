'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
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
import MediumItemMetadataSourceCreate from '@/components/MediumItemMetadataSourceCreate'
import MediumItemMetadataSourceEdit from '@/components/MediumItemMetadataSourceEdit'
import MediumItemMetadataSourceList from '@/components/MediumItemMetadataSourceList'
import MediumItemMetadataSummaryCreate from '@/components/MediumItemMetadataSummaryCreate'
import MediumItemMetadataSummaryEdit from '@/components/MediumItemMetadataSummaryEdit'
import MediumItemMetadataSummaryShow from '@/components/MediumItemMetadataSummaryShow'
import MediumItemMetadataTagCreate from '@/components/MediumItemMetadataTagCreate'
import MediumItemMetadataTagEdit from '@/components/MediumItemMetadataTagEdit'
import MediumItemMetadataTagList from '@/components/MediumItemMetadataTagList'
import type { TagTagTypeInput } from '@/graphql/types.generated'
import { useBeforeUnload, useCreateMedium, useUpdateMedium } from '@/hooks'
import type { Medium, Replica } from '@/types'

import styles from './styles.module.scss'

const hasChanges = (medium: Medium | null, replicas: (Replica | ReplicaCreate)[]) => {
  if ((medium?.replicas?.length ?? 0) !== replicas.length) {
    return true
  }

  for (const [ idx, replica ] of replicas.entries()) {
    if (!isReplica(replica)) {
      return true
    }

    if (medium?.replicas?.[idx]?.id !== replica.id) {
      return true
    }
  }

  return false
}

const MediumCreateView: FunctionComponent = () => {
  const router = useRouter()

  const [ medium, setMedium ] = useState<Medium | null>(null)
  const [ resolveMedium, setResolveMedium ] = useState<(() => Promise<Medium>) | null>(null)
  const [ createMedium, { loading: createLoading } ] = useCreateMedium()
  const [ updateMedium, { loading: updateLoading } ] = useUpdateMedium()

  const [ editingSummary, setEditingSummary ] = useState(true)
  const [ editingSources, setEditingSources ] = useState(true)
  const [ editingTags, setEditingTags ] = useState(true)

  const [ resolveSourceIDs, setResolveSourceIDs ] = useState(() => () => Promise.resolve<string[]>([]))
  const [ tagTagTypeIDs, setTagTagTypeIDs ] = useState<TagTagTypeInput[]>([])
  const [ replicas, setReplicas ] = useState<(Replica | ReplicaCreate)[]>([])

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
  }, [])

  const editSummary = useCallback(() => {
    setEditingSummary(true)
  }, [])

  const closeEditSummary = useCallback((newReplicas?: Replica[]) => {
    setEditingSummary(false)
    setReplicas(() => newReplicas ?? medium?.replicas ?? [])
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
      try {
        const newMedium = await updateMedium({
          id: current.id,
          replicaOrders: replicas.filter(isReplica).map(({ id }) => id),
          createdAt: current.createdAt,
        })
        setMedium(newMedium)
      } catch (e) {
        console.error('Error updating medium\n', e)
        setMedium(current)
        setError(e)
      }
      return
    }

    setUploading(false)
    setUploadAborting(false)

    try {
      await updateMedium({
        id: current.id,
        replicaOrders: replicas.map(({ id }) => id),
        createdAt: current.createdAt,
      })
      router.refresh()
    } catch (e) {
      console.error('Error updating medium\n', e)
      setMedium({
        ...current,
        replicas,
      })
      setError(e)
    }
  }, [ updateMedium, router ])

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
        await handleComplete(newMedium, replicas)
      }
    } catch (e) {
      console.error('Error creating medium\n', e)
      setError(e)
    }
  }, [ medium, resolveSourceIDs, tagTagTypeIDs, replicas, createMedium, handleComplete ])

  const saveTags = useCallback(async (id: string, addTagTagTypeIDs: TagTagTypeInput[], removeTagTagTypeIDs: TagTagTypeInput[]) => {
    const medium = await updateMedium({
      id,
      addTagTagTypeIDs,
      removeTagTagTypeIDs,
    })
    setMedium(medium)
    return medium
  }, [ updateMedium ])

  const saveSources = useCallback(async (id: string, addSourceIDs: string[], removeSourceIDs: string[]) => {
    const medium = await updateMedium({
      id,
      addSourceIDs,
      removeSourceIDs,
    })
    setMedium(medium)
    return medium
  }, [ updateMedium ])

  const handleDeleteMedium = useCallback(() => {
    router.replace('/')
  }, [ router ])

  const loading = createLoading || updateLoading
  const changed = hasChanges(medium, replicas)
  useBeforeUnload(changed)

  return (
    <Grid className={styles.container} container spacing={4} flexGrow={1}>
      <Grid size={9}>
        {!medium || editingSummary ? (
          <MediumItemImageEdit
            className={styles.imageList}
            gap={32}
            replicas={replicas}
            setReplicas={setReplicas}
            removeReplica={removeReplica}
          />
        ) : (
          <MediumItemImageList
            className={styles.imageList}
            gap={32}
            replicas={medium.replicas ?? []}
          />
        )}
      </Grid>
      <Grid className={styles.metadataContainer} size={3}>
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
      {resolveMedium && uploading ? (
        <MediumItemFileUploadDialog
          abortSignal={uploadAbortController.signal}
          resolveMedium={resolveMedium}
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
