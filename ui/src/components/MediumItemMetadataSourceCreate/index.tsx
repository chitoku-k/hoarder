'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import deepEqual from 'deep-equal'
import Stack from '@mui/material/Stack'
import FolderSpecialIcon from '@mui/icons-material/FolderSpecial'

import AutocompleteExternalService from '@/components/AutocompleteExternalService'
import type { SourceCreate } from '@/components/AutocompleteSourceBody'
import { isSource } from '@/components/AutocompleteSourceBody'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import MediumItemMetadataSourceGroupEdit from '@/components/MediumItemMetadataSourceGroupEdit'
import { SOURCE_METADATA_DUPLICATE, useCreateSource, useError } from '@/hooks'
import type { ExternalMetadataInput } from '@/hooks/types.generated'
import type { ExternalService, Source } from '@/types'

const MediumItemMetadataSourceCreate: FunctionComponent<MediumItemMetadataSourceCreateProps> = ({
  loading,
  setResolveSourceIDs,
}) => {
  const [ createSource ] = useCreateSource()
  const { graphQLError } = useError()

  const [ focusedExternalService, setFocusedExternalService ] = useState<ExternalService | null>(null)
  const [ newExternalService, setNewExternalService ] = useState<ExternalService | null>(null)

  const [ addingExternalServices, setAddingExternalServices ] = useState<ExternalService[]>([])
  const [ addingSources, setAddingSources ] = useState(new Map<ExternalServiceID, (Source | SourceCreate)[]>())

  const handleChangeNewExternalService = useCallback((type: ExternalService | null) => {
    if (!type) {
      return
    }

    setNewExternalService(null)
    setFocusedExternalService(type)
    setAddingExternalServices(addingExternalServices => [
      ...addingExternalServices,
      type,
    ])
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
  }, [])

  const restoreExternalService = useCallback(() => {}, [])

  const restoreSource = useCallback(() => {}, [])

  const resolveSourceIDs = useCallback((addingSources: Map<ExternalServiceID, (Source | SourceCreate)[]>) => async () => {
    const addingSourceIDs: string[] = []
    const createSources: Promise<void>[] = []
    for (const sources of addingSources.values()) {
      for (const source of sources) {
        if (isSource(source)) {
          addingSourceIDs.push(source.id)
          continue
        }
        createSources.push(
          createSource({
            externalServiceID: source.externalService.id,
            externalMetadata: source.externalMetadata as ExternalMetadataInput,
          }).then(
            newSource => {
              addingSourceIDs.push(newSource.id)
            },
            e => {
              const sourceMetadataDuplicate = graphQLError(e, SOURCE_METADATA_DUPLICATE)
              if (!sourceMetadataDuplicate?.extensions.details.data.id) {
                throw e
              }
              addingSourceIDs.push(sourceMetadataDuplicate.extensions.details.data.id)
            },
          )
        )
      }
    }

    await Promise.all(createSources)
    return addingSourceIDs
  }, [ createSource, graphQLError ])

  const addSource = useCallback((externalService: ExternalService, source: Source | SourceCreate) => {
    setFocusedExternalService(externalService)

    setAddingSources(addingSources => {
      const newSources = addingSources.get(externalService.id) ?? []
      if (newSources.some(newSource => deepEqual(newSource.externalMetadata, source.externalMetadata))) {
        return addingSources
      }

      newSources.push(source)

      const newAddingSources = new Map(addingSources.set(externalService.id, newSources))
      setResolveSourceIDs(() => resolveSourceIDs(newAddingSources))
      return newAddingSources
    })
  }, [ setResolveSourceIDs, resolveSourceIDs ])

  const removeSource = useCallback((externalService: ExternalService, source: Source | SourceCreate) => {
    setFocusedExternalService(null)

    setAddingSources(addingSources => {
      const newSources = addingSources.get(externalService.id) ?? []
      const idx = newSources.findIndex(newSource => deepEqual(newSource.externalMetadata, source.externalMetadata))
      if (idx < 0) {
        return addingSources
      }

      const newAddingSources = new Map(addingSources.set(externalService.id, newSources.toSpliced(idx, 1)))
      setResolveSourceIDs(() => resolveSourceIDs(newAddingSources))
      return newAddingSources
    })
  }, [ setResolveSourceIDs, resolveSourceIDs ])

  return (
    <Stack>
      <MediumItemMetadataHeader title="ソース" />
      <Stack spacing={4}>
        {addingExternalServices.map(externalService => (
          <MediumItemMetadataSourceGroupEdit
            key={`${externalService.id}-${addingSources.get(externalService.id)?.length ?? 0}`}
            loading={loading}
            externalService={externalService}
            sources={[]}
            focus={focusedExternalService?.id === externalService.id}
            removingExternalService={false}
            removeExternalService={removeExternalService}
            restoreExternalService={restoreExternalService}
            addingSources={addingSources.get(externalService.id) ?? []}
            removingSources={[]}
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
            includeInputInList
            placeholder="サービスの追加..."
            disabled={loading}
            value={newExternalService}
            getOptionDisabled={({ id }) => addingExternalServices.some(externalService => externalService.id === id)}
            icon={({ ...props }) => <FolderSpecialIcon {...props} />}
            onChange={handleChangeNewExternalService}
          />
        </Stack>
      </Stack>
    </Stack>
  )
}

export interface MediumItemMetadataSourceCreateProps {
  loading: boolean
  setResolveSourceIDs: (setResolveSourceIDs: () => () => Promise<string[]>) => void
}

type ExternalServiceID = string

export default MediumItemMetadataSourceCreate
