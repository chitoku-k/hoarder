'use client'

import type { ComponentPropsWithoutRef, FunctionComponent } from 'react'
import { useCallback } from 'react'
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
import SourceURL from '@/components/SourceURL'
import type { ExternalService, Source } from '@/types'

import styles from './styles.module.scss'

const MediumItemMetadataSourceGroupEdit: FunctionComponent<MediumItemMetadataSourceGroupEditProps> = ({
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
  const handleChangeNewSource = useCallback((source: Source | SourceCreate | null) => {
    if (!source) {
      return
    }

    restoreExternalService?.(externalService)
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
    restoreExternalService?.(externalService)

    for (const source of sources) {
      restoreSource?.(externalService, source)
    }
  }, [ restoreExternalService, restoreSource, externalService, sources ])

  const handleClickRemoveSource = (source: Source | SourceCreate) => {
    removeSource(externalService, source)
  }

  const handleClickRestoreSource = (tag: Source) => {
    restoreExternalService?.(externalService)
    restoreSource?.(externalService, tag)
  }

  const renderSourceOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: Source | SourceCreate) => (
    <li key={key} {...props}>
      {isSource(option) ? (
        <SourceURL source={option} noLink noLaunch />
      ) : (
        <SourceURL icon={AddLinkIcon} externalService={option.externalService} externalMetadata={option.externalMetadata} noLink noLaunch />
      )}
    </li>
  ), [])

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
                <del><SourceURL source={source} noLink /></del>
                <Tooltip title="元に戻す" placement="right">
                  <IconButton size="small" disabled={loading} onClick={() => handleClickRestoreSource(source)}>
                    <AddCircleOutlineIcon fontSize="inherit" />
                  </IconButton>
                </Tooltip>
              </>
            ) : isSource(source) ? (
              <>
                <SourceURL source={source} noLink />
                <Tooltip title="削除" placement="right">
                  <IconButton size="small" disabled={loading} onClick={() => handleClickRemoveSource(source)}>
                    <RemoveCircleOutlineIcon fontSize="inherit" />
                  </IconButton>
                </Tooltip>
              </>
            ) : (
              <>
                <SourceURL icon={AddLinkIcon} externalService={source.externalService} externalMetadata={source.externalMetadata} noLink />
                <Tooltip title="削除" placement="right">
                  <IconButton size="small" disabled={loading} onClick={() => handleClickRemoveSource(source)}>
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
            renderOption={renderSourceOption}
            value={null}
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

export interface MediumItemMetadataSourceGroupEditProps {
  loading: boolean
  externalService: ExternalService
  sources: Source[]
  focus?: boolean
  removingExternalService: boolean
  removeExternalService: (externalService: ExternalService) => void
  restoreExternalService?: (externalService: ExternalService) => void
  addingSources: (Source | SourceCreate)[]
  removingSources: Source[]
  addSource: (externalService: ExternalService, source: Source | SourceCreate) => void
  removeSource: (externalService: ExternalService, source: Source | SourceCreate) => void
  restoreSource?: (externalService: ExternalService, source: Source) => void
}

export default MediumItemMetadataSourceGroupEdit
