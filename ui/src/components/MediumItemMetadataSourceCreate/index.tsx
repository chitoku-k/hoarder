'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import deepEqual from 'deep-equal'
import Stack from '@mui/material/Stack'
import FolderSpecialIcon from '@mui/icons-material/FolderSpecial'

import AutocompleteExternalService from '@/components/AutocompleteExternalService'
import type { SourceCreate } from '@/components/AutocompleteSourceBody'
import { isSource } from '@/components/AutocompleteSourceBody'
import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import MediumItemMetadataSourceGroupEdit from '@/components/MediumItemMetadataSourceGroupEdit'
import type { ExternalMetadataInput } from '@/graphql/types.generated'
import { SOURCE_METADATA_DUPLICATE, useBeforeUnload, useCreateSource, useError } from '@/hooks'
import type { ExternalService, Source } from '@/types'

import styles from './styles.module.scss'

const hasChanges = (addingExternalServices: readonly ExternalService[], addingSources: ReadonlyMap<ExternalServiceID, readonly (Source | SourceCreate)[]>) => {
  if (addingExternalServices.length > 0) {
    return true
  }

  for (const tags of addingSources.values()) {
    if (tags.length > 0) {
      return true
    }
  }

  return false
}

const MediumItemMetadataSourceCreate: FunctionComponent<MediumItemMetadataSourceCreateProps> = ({
  loading,
  setResolveSourceIDs,
}) => {
  const [ createSource ] = useCreateSource()
  const { graphQLError } = useError()

  const [ focusedExternalService, setFocusedExternalService ] = useState<ExternalService | null>(null)
  const [ newExternalServiceInput, setNewExternalServiceInput ] = useState('')

  const [ addingExternalServices, setAddingExternalServices ] = useState<readonly ExternalService[]>([])
  const [ addingSources, setAddingSources ] = useState<ReadonlyMap<ExternalServiceID, readonly (Source | SourceCreate)[]>>(new Map())

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
  }, [])

  const resolveSourceIDs = useCallback((addingSources: ReadonlyMap<ExternalServiceID, readonly (Source | SourceCreate)[]>) => async () => {
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

    await Promise.all(createSources)
    return addingSourceIDs
  }, [ createSource, graphQLError ])

  const addSource = useCallback((externalService: ExternalService, source: Source | SourceCreate) => {
    setFocusedExternalService(externalService)

    const newSources = addingSources.get(externalService.id) ?? []
    if (newSources.some(newSource => deepEqual(newSource.externalMetadata, source.externalMetadata))) {
      return
    }

    const newAddingSources = new Map(addingSources).set(externalService.id, [ ...newSources, source ])
    setAddingSources(newAddingSources)
    setResolveSourceIDs(() => resolveSourceIDs(newAddingSources))
  }, [ addingSources, setResolveSourceIDs, resolveSourceIDs ])

  const removeSource = useCallback((externalService: ExternalService, source: Source | SourceCreate) => {
    setFocusedExternalService(null)

    const newSources = addingSources.get(externalService.id) ?? []
    const idx = newSources.findIndex(newSource => deepEqual(newSource.externalMetadata, source.externalMetadata))
    if (idx < 0) {
      return
    }

    const newAddingSources = new Map(addingSources).set(externalService.id, newSources.toSpliced(idx, 1))
    setAddingSources(newAddingSources)
    setResolveSourceIDs(() => resolveSourceIDs(newAddingSources))
  }, [ addingSources, setResolveSourceIDs, resolveSourceIDs ])

  const renderExternalServiceOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: ExternalService) => (
    <li key={key} {...props}>
      <Stack direction="row" spacing={0.5} alignItems="start">
        <FolderSpecialIcon className={styles.externalServiceSearchIcon} fontSize="small" />
        <span className={styles.externalServiceSearchText}>{option.name}</span>
      </Stack>
    </li>
  ), [])

  const changed = hasChanges(addingExternalServices, addingSources)
  useBeforeUnload(changed)

  return (
    <Stack>
      <MediumItemMetadataHeader title="ソース" />
      <Stack spacing={4}>
        {addingExternalServices.map(externalService => (
          <MediumItemMetadataSourceGroupEdit
            key={`${externalService.id}-${String(addingSources.get(externalService.id)?.length ?? 0)}`}
            loading={loading}
            externalService={externalService}
            sources={[]}
            focus={focusedExternalService?.id === externalService.id}
            removingExternalService={false}
            removeExternalService={removeExternalService}
            addingSources={addingSources.get(externalService.id) ?? []}
            removingSources={[]}
            addSource={addSource}
            removeSource={removeSource}
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
            placeholder="サービスの追加..."
            disabled={loading}
            renderOption={renderExternalServiceOption}
            value={null}
            inputValue={newExternalServiceInput}
            getOptionDisabled={({ id }) => addingExternalServices.some(externalService => externalService.id === id)}
            icon={({ ...props }) => <FolderSpecialIcon {...props} />}
            onChange={handleChangeNewExternalService}
            onInputChange={handleChangeNewExternalServiceInput}
          />
        </Stack>
      </Stack>
    </Stack>
  )
}

export interface MediumItemMetadataSourceCreateProps {
  readonly loading: boolean
  readonly setResolveSourceIDs: (setResolveSourceIDs: () => () => Promise<readonly string[]>) => void
}

type ExternalServiceID = string

export default MediumItemMetadataSourceCreate
