'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import deepEqual from 'deep-equal'
import IconButton from '@mui/material/IconButton'
import Stack from '@mui/material/Stack'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'
import RemoveCircleOutlineIcon from '@mui/icons-material/RemoveCircleOutline'
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline'
import AddLinkIcon from '@mui/icons-material/AddLink'

import type { SourceCreate } from '@/components/AutocompleteSourceBody'
import { isSource } from '@/components/AutocompleteSourceBody'
import AutocompleteSource from '@/components/AutocompleteSource'
import MediumItemMetadataSourceItem from '@/components/MediumItemMetadataSourceItem'
import MediumItemMetadataSourceItemNew from '@/components/MediumItemMetadataSourceItemNew'
import type { ExternalService, Source } from '@/types'

import styles from './styles.module.scss'

const MetadataSourceGroupEdit: FunctionComponent<MetadataSourceGroupEditProps> = ({
  loading,
  externalService,
  sources,
  focus,
  removingExternalService,
  removeExternalService,
  restoreExternalService,
  addingSources,
  removingSources,
  addSource,
  removeSource,
  restoreSource,
}) => {
  const [ newSource, setNewSource ] = useState<Source | SourceCreate | null>(null)

  const handleChangeNewSource = useCallback((source: Source | SourceCreate | null) => {
    if (!source) {
      return
    }

    setNewSource(null)
    restoreExternalService(externalService)
    addSource(externalService, source)
  }, [ restoreExternalService, externalService, addSource ])

  const handleClickRemoveExternalService = useCallback(() => {
    removeExternalService(externalService)

    for (const source of sources) {
      removeSource(externalService, source)
    }

    for (const source of addingSources) {
      removeSource(externalService, source)
    }
  }, [ removeExternalService, externalService, removeSource, sources, addingSources ])

  const handleClickRestoreExternalService = useCallback(() => {
    restoreExternalService(externalService)

    for (const source of sources) {
      restoreSource(externalService, source)
    }
  }, [ restoreExternalService, restoreSource, externalService, sources ])

  const handleClickRemoveSource = (source: Source | SourceCreate) => {
    removeSource(externalService, source)
  }

  const handleClickRestoreSource = (tag: Source) => {
    restoreExternalService(externalService)
    restoreSource(externalService, tag)
  }

  const allSources = [ ...sources, ...addingSources ]

  return (
    <Stack>
      <Stack className={styles.header} direction="row" alignItems="center" justifyContent="space-between">
        {removingExternalService || (removingSources.length > 0 && removingSources.length === sources.length && addingSources.length === 0) ? (
          <>
            <Typography className={styles.title} variant="h4">
              <del>{externalService.name}</del>
            </Typography>
            <Stack direction="row" alignItems="center">
              <Tooltip title="元に戻す" placement="right">
                <IconButton size="small" disabled={loading} onClick={handleClickRestoreExternalService}>
                  <AddCircleOutlineIcon fontSize="inherit" />
                </IconButton>
              </Tooltip>
            </Stack>
          </>
        ) : (
          <>
            <Typography className={styles.title} variant="h4">
              {externalService.name}
            </Typography>
            <Stack direction="row" alignItems="center">
              <Tooltip title="削除" placement="right">
                <IconButton size="small" disabled={loading} onClick={handleClickRemoveExternalService}>
                  <RemoveCircleOutlineIcon fontSize="inherit" />
                </IconButton>
              </Tooltip>
            </Stack>
          </>
        )}
      </Stack>
      <Stack spacing={0.5}>
        {allSources.map((source, i) => (
          <Stack key={isSource(source) ? source.id : i} direction="row" alignItems="center" justifyContent="space-between">
            {isSource(source) && removingSources.some(({ id }) => id === source.id) ? (
              <>
                <del><MediumItemMetadataSourceItem source={source} noLink /></del>
                <Tooltip title="元に戻す" placement="right">
                  <IconButton size="small" disabled={loading} onClick={() => handleClickRestoreSource(source)}>
                    <AddCircleOutlineIcon fontSize="inherit" />
                  </IconButton>
                </Tooltip>
              </>
            ) : isSource(source) ? (
              <>
                <MediumItemMetadataSourceItem source={source} noLink />
                <Tooltip title="削除" placement="right">
                  <IconButton size="small" disabled={loading} onClick={() => handleClickRemoveSource(source)} >
                    <RemoveCircleOutlineIcon fontSize="inherit" />
                  </IconButton>
                </Tooltip>
              </>
            ) : (
              <>
                <MediumItemMetadataSourceItemNew {...source} />
                <Tooltip title="削除" placement="right">
                  <IconButton size="small" disabled={loading} onClick={() => handleClickRemoveSource(source)} >
                    <RemoveCircleOutlineIcon fontSize="inherit" />
                  </IconButton>
                </Tooltip>
              </>
            )}
          </Stack>
        ))}
        <Stack flexGrow={1} direction="row">
          <AutocompleteSource
            className={styles.newSource}
            size="small"
            variant="standard"
            fullWidth
            blurOnSelect
            clearOnBlur={false}
            clearOnEscape
            includeInputInList
            focus={focus}
            forcePopupIcon={false}
            placeholder="ソースの追加..."
            externalService={externalService}
            disabled={loading}
            value={newSource}
            getOptionDisabled={option => isSource(option)
              ? sources.some(source => source.id === option.id)
              : addingSources.some(source => deepEqual(source.externalMetadata, option.externalMetadata))}
            icon={({ ...props }) => <AddLinkIcon {...props} />}
            onChange={handleChangeNewSource}
            slotProps={{
              popper: {
                className: styles.newSourcePopper,
                placement: 'bottom-start',
              },
            }}
          />
        </Stack>
      </Stack>
    </Stack>
  )
}

export interface MetadataSourceGroupEditProps {
  loading: boolean
  externalService: ExternalService
  sources: Source[]
  focus?: boolean
  removingExternalService: boolean
  removeExternalService: (externalService: ExternalService) => void
  restoreExternalService: (externalService: ExternalService) => void
  addingSources: (Source | SourceCreate)[]
  removingSources: Source[]
  addSource: (externalService: ExternalService, source: Source | SourceCreate) => void
  removeSource: (externalService: ExternalService, source: Source | SourceCreate) => void
  restoreSource: (externalService: ExternalService, source: Source) => void
}

export default MetadataSourceGroupEdit
