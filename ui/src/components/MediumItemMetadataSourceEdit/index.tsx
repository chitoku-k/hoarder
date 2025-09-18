'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import deepEqual from 'deep-equal'
import Button from '@mui/material/Button'
import Portal from '@mui/material/Portal'
import Snackbar from '@mui/material/Snackbar'
import Stack from '@mui/material/Stack'
import FolderSpecialIcon from '@mui/icons-material/FolderSpecial'

import AutocompleteExternalService from '@/components/AutocompleteExternalService'
import type { SourceCreate } from '@/components/AutocompleteSourceBody'
import { isSource } from '@/components/AutocompleteSourceBody'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import MediumItemMetadataSourceGroupEdit from '@/components/MediumItemMetadataSourceGroupEdit'
import type { ExternalMetadataInput } from '@/graphql/types.generated'
import { SOURCE_METADATA_DUPLICATE, useBeforeUnload, useCreateSource, useError } from '@/hooks'
import type { ExternalService, Medium, Source } from '@/types'

import styles from './styles.module.scss'

const hasChanges = (addingExternalServices: ExternalService[], removingExternalServices: ExternalService[], addingSources: Map<ExternalServiceID, (Source | SourceCreate)[]>, removingSources: Map<ExternalServiceID, Source[]>) => {
  if (addingExternalServices.length > 0 || removingExternalServices.length > 0) {
    return true
  }

  for (const tags of addingSources.values()) {
    if (tags.length > 0) {
      return true
    }
  }

  for (const tags of removingSources.values()) {
    if (tags.length > 0) {
      return true
    }
  }

  return false
}

const MediumItemMetadataSourceEdit: FunctionComponent<MediumItemMetadataSourceEditProps> = ({
  loading: mediumLoading,
  focus,
  medium,
  save,
  close,
}) => {
  const [ createSource, { loading: sourceLoading, error } ] = useCreateSource()
  const { graphQLError } = useError()

  const [ focusedExternalService, setFocusedExternalService ] = useState<ExternalService | null>(null)
  const [ newExternalServiceInput, setNewExternalServiceInput ] = useState('')

  const [ addingExternalServices, setAddingExternalServices ] = useState<ExternalService[]>([])
  const [ removingExternalServices, setRemovingExternalServices ] = useState<ExternalService[]>([])

  const [ addingSources, setAddingSources ] = useState(new Map<ExternalServiceID, (Source | SourceCreate)[]>())
  const [ removingSources, setRemovingSources ] = useState(new Map<ExternalServiceID, Source[]>())

  const sources = medium.sources ?? []
  const groups = sources.reduce<SourceGroup[]>((groups, source) => {
    const group = groups.find(s => s.externalService.id === source.externalService.id)
    if (group) {
      group.sources.push(source)
    } else {
      groups.push({
        externalService: source.externalService,
        sources: [ source ],
      })
    }
    return groups
  }, [])

  const handleChangeNewExternalService = useCallback((type: ExternalService | null) => {
    if (!type) {
      return
    }

    setNewExternalServiceInput('')

    setFocusedExternalService(type)
    setAddingExternalServices(addingExternalServices => [
      ...addingExternalServices,
      type,
    ])
  }, [])

  const handleChangeNewExternalServiceInput = useCallback((_e: SyntheticEvent, value: string) => {
    setNewExternalServiceInput(value)
  }, [])

  const removeExternalService = useCallback((type: ExternalService) => {
    setFocusedExternalService(null)

    setAddingExternalServices(addingExternalServices => {
      const idx = addingExternalServices.findIndex(({ id }) => id === type.id)
      if (idx < 0) {
        return addingExternalServices
      }

      return addingExternalServices.toSpliced(idx, 1)
    })

    if (!groups.some(group => group.externalService.id === type.id)) {
      return
    }

    setRemovingExternalServices(removingExternalServices => [
      ...removingExternalServices,
      type,
    ])
  }, [ groups ])

  const restoreExternalService = useCallback((type: ExternalService) => {
    setRemovingExternalServices(removingExternalServices => removingExternalServices.filter(({ id }) => id !== type.id))
  }, [])

  const addSource = useCallback((externalService: ExternalService, source: Source | SourceCreate) => {
    setFocusedExternalService(externalService)

    setAddingSources(addingSources => {
      const newSources = addingSources.get(externalService.id) ?? []
      if (newSources.some(newSource => deepEqual(newSource.externalMetadata, source.externalMetadata))) {
        return addingSources
      }
      if (isSource(source) && groups.some(group => group.externalService.id === externalService.id && group.sources.some(({ id }) => id === source.id))) {
        return addingSources
      }

      return new Map(addingSources).set(externalService.id, [ ...newSources, source ])
    })

    setRemovingSources(removingSources => {
      if (!isSource(source)) {
        return removingSources
      }

      const newSources = removingSources.get(externalService.id) ?? []
      const idx = newSources.findIndex(({ id }) => id === source.id)
      if (idx < 0) {
        return removingSources
      }

      return new Map(removingSources).set(externalService.id, newSources.toSpliced(idx, 1))
    })
  }, [ groups ])

  const removeSource = useCallback((externalService: ExternalService, source: Source | SourceCreate) => {
    setFocusedExternalService(null)

    setAddingSources(addingSources => {
      const newSources = addingSources.get(externalService.id) ?? []
      const idx = newSources.findIndex(newSource => deepEqual(newSource.externalMetadata, source.externalMetadata))
      if (idx < 0) {
        return addingSources
      }

      return new Map(addingSources).set(externalService.id, newSources.toSpliced(idx, 1))
    })

    if (!isSource(source) || !groups.some(group => group.externalService.id === externalService.id && group.sources.some(({ id }) => id === source.id))) {
      return
    }

    setRemovingSources(removingSources => {
      const newSources = removingSources.get(externalService.id) ?? []
      if (newSources.some(({ id }) => id === source.id)) {
        return removingSources
      }

      return new Map(removingSources).set(externalService.id, [ ...newSources, source ])
    })
  }, [ groups ])

  const restoreSource = useCallback((externalService: ExternalService, source: Source) => {
    setFocusedExternalService(null)

    setRemovingSources(removingSources => {
      const newSources = removingSources.get(externalService.id) ?? []
      const idx = newSources.findIndex(({ id }) => id === source.id)
      if (idx < 0) {
        return removingSources
      }

      return new Map(removingSources).set(externalService.id, newSources.toSpliced(idx, 1))
    })
  }, [])

  const handleClickCancel = useCallback(() => {
    close?.()
  }, [ close ])

  const handleClickSubmit = useCallback(async () => {
    const addingSourceIDs: string[] = []
    const createSources: Promise<void>[] = []
    for (const sources of addingSources.values()) {
      for (const source of sources) {
        if (isSource(source)) {
          addingSourceIDs.push(source.id)
          continue
        }
        createSources.push((async () => {
          try {
            const newSource = await createSource({
              externalServiceID: source.externalService.id,
              externalMetadata: source.externalMetadata as ExternalMetadataInput,
            })
            addingSourceIDs.push(newSource.id)
          } catch (e) {
            const sourceMetadataDuplicate = graphQLError(e, SOURCE_METADATA_DUPLICATE)
            if (!sourceMetadataDuplicate?.extensions.details.data.id) {
              throw e
            }
            addingSourceIDs.push(sourceMetadataDuplicate.extensions.details.data.id)
          }
        })())
      }
    }

    const removingSourceIDs: string[] = []
    for (const sources of removingSources.values()) {
      removingSourceIDs.push(...sources.map(({ id }) => id))
    }

    try {
      await Promise.all(createSources)
      await save(medium.id, addingSourceIDs, removingSourceIDs)
      close?.()
    } catch (e) {
      console.error('Error updating medium\n', e)
    }
  }, [ createSource, graphQLError, save, medium, addingSources, removingSources, close ])

  const renderExternalServiceOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: ExternalService) => (
    <li key={key} {...props}>
      <Stack direction="row" spacing={0.5} alignItems="start">
        <FolderSpecialIcon className={styles.externalServiceSearchIcon} fontSize="small" />
        <span className={styles.externalServiceSearchText}>{option.name}</span>
      </Stack>
    </li>
  ), [])

  const loading = sourceLoading || mediumLoading
  const changed = hasChanges(addingExternalServices, removingExternalServices, addingSources, removingSources)
  useBeforeUnload(changed)

  return (
    <Stack>
      <MediumItemMetadataHeader title="ソース">
        <Button onClick={handleClickSubmit} loading={loading}>
          保存
        </Button>
        <Button onClick={handleClickCancel}>
          キャンセル
        </Button>
      </MediumItemMetadataHeader>
      <Stack spacing={4}>
        {groups.map(({ externalService, sources }) => (
          <MediumItemMetadataSourceGroupEdit
            key={`${externalService.id}-${String(addingSources.get(externalService.id)?.length ?? 0)}`}
            loading={loading}
            externalService={externalService}
            sources={sources}
            focus={focusedExternalService?.id === externalService.id}
            removingExternalService={removingExternalServices.some(({ id }) => id === externalService.id)}
            removeExternalService={removeExternalService}
            restoreExternalService={restoreExternalService}
            addingSources={addingSources.get(externalService.id) ?? []}
            removingSources={removingSources.get(externalService.id) ?? []}
            addSource={addSource}
            removeSource={removeSource}
            restoreSource={restoreSource}
          />
        ))}
        {addingExternalServices.map(externalService => (
          <MediumItemMetadataSourceGroupEdit
            key={`${externalService.id}-${String(addingSources.get(externalService.id)?.length ?? 0)}`}
            loading={loading}
            externalService={externalService}
            sources={[]}
            focus={focusedExternalService?.id === externalService.id}
            removingExternalService={false}
            removeExternalService={removeExternalService}
            restoreExternalService={restoreExternalService}
            addingSources={addingSources.get(externalService.id) ?? []}
            removingSources={removingSources.get(externalService.id) ?? []}
            addSource={addSource}
            removeSource={removeSource}
            restoreSource={restoreSource}
          />
        ))}
        <Stack spacing={0.5} direction="row" alignItems="center" justifyContent="space-between">
          <AutocompleteExternalService
            fullWidth
            openOnFocus
            autoHighlight
            blurOnSelect
            clearOnBlur={false}
            clearOnEscape
            includeInputInList
            focus={focus && groups.length === 0}
            loadOnOpen
            placeholder="サービスの追加..."
            disabled={loading}
            renderOption={renderExternalServiceOption}
            value={null}
            inputValue={newExternalServiceInput}
            getOptionDisabled={({ id }) => groups.some(group => group.externalService.id === id) || addingExternalServices.some(externalService => externalService.id === id)}
            icon={({ ...props }) => <FolderSpecialIcon {...props} />}
            onChange={handleChangeNewExternalService}
            onInputChange={handleChangeNewExternalServiceInput}
          />
        </Stack>
      </Stack>
      {error ? (
        <Portal>
          <Snackbar
            open
            anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
            message="ソースを作成できませんでした"
          />
        </Portal>
      ) : null}
    </Stack>
  )
}

export interface MediumItemMetadataSourceEditProps {
  loading: boolean
  focus?: boolean
  medium: Medium
  save: (id: string, addSourceIDs: string[], removeSourceIDs: string[]) => Promise<Medium>
  close?: () => void
}

interface SourceGroup {
  externalService: ExternalService
  sources: Source[]
}

type ExternalServiceID = string

export default MediumItemMetadataSourceEdit
