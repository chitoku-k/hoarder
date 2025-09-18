'use client'

import type { ComponentPropsWithoutRef, FunctionComponent, MouseEvent, SyntheticEvent } from 'react'
import { useCallback, useState } from 'react'
import clsx from 'clsx'
import type { AutocompleteInputChangeReason } from '@mui/material/Autocomplete'
import IconButton from '@mui/material/IconButton'
import List from '@mui/material/List'
import Stack from '@mui/material/Stack'
import AddIcon from '@mui/icons-material/Add'
import DeleteOutlinedIcon from '@mui/icons-material/DeleteOutlined'
import EditOutlinedIcon from '@mui/icons-material/EditOutlined'
import FolderSpecialIcon from '@mui/icons-material/FolderSpecial'
import SearchIcon from '@mui/icons-material/Search'

import AutocompleteExternalService from '@/components/AutocompleteExternalService'
import ExternalServiceListColumnBodyListItem from '@/components/ExternalServiceListColumnBodyListItem'
import { useAllExternalServices } from '@/hooks'
import type { ExternalService } from '@/types'

import styles from './styles.module.scss'

const ExternalServiceListColumnBodyList: FunctionComponent<ExternalServiceListColumnBodyListProps> = ({
  creating,
  editing,
  active,
  hit,
  hitInput,
  readonly,
  dense,
  disabled: disabledExternalService,
  onSelect: onSelectExternalService,
  onHit: onHitExternalService,
  show: showExternalService,
  create: createExternalService,
  edit: editExternalService,
  delete: deleteExternalService,
  setColumn,
}) => {
  const allExternalServices = useAllExternalServices()

  const [ scrollTop, setScrollTop ] = useState(0)
  const ref = useCallback((node: HTMLElement | null) => {
    if (!node) {
      return
    }
    if (creating) {
      setScrollTop(node.scrollTop)
      node.scrollTo({
        top: node.scrollHeight,
        behavior: 'smooth',
      })
    } else {
      node.scrollTo({
        top: scrollTop,
        behavior: 'smooth',
      })
    }
  }, [ creating, scrollTop ])

  const handleClickExternalService = (externalService: ExternalService) => {
    onSelectExternalService?.(externalService)
    onHitExternalService?.(externalService)
    showExternalService(externalService)
  }

  const handleHitExternalService = useCallback((externalService: ExternalService | null) => {
    onHitExternalService?.(externalService)
  }, [ onHitExternalService ])

  const handleInputHitExternalService = useCallback((_e: SyntheticEvent, value: string, reason: AutocompleteInputChangeReason) => {
    if (!value && reason === 'input') {
      onHitExternalService?.(null)
    }
    setColumn({
      creating,
      editing,
      active,
      hit,
      hitInput: value,
    })
  }, [ onHitExternalService, setColumn, creating, editing, active, hit ])

  const handleClickCreateExternalService = useCallback(() => {
    createExternalService()
  }, [ createExternalService ])

  const handleClickEditExternalService = (e: MouseEvent<HTMLButtonElement>, externalService: ExternalService) => {
    editExternalService(externalService)
    e.stopPropagation()
  }

  const handleClickDeleteExternalService = (e: MouseEvent<HTMLButtonElement>, externalService: ExternalService) => {
    deleteExternalService(externalService)
    e.stopPropagation()
  }

  const handleMouseDownEditExternalService = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  const handleMouseDownDeleteExternalService = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
  }, [])

  const renderExternalServiceOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: ExternalService) => (
    <li key={key} {...props}>
      <Stack direction="row" spacing={0.5} alignItems="start">
        <FolderSpecialIcon className={styles.externalServiceSearchIcon} fontSize="small" />
        <span className={styles.externalServiceSearchText}>{option.name}</span>
      </Stack>
    </li>
  ), [])

  if (!allExternalServices) {
    throw new Error('unreachable')
  }

  return (
    <Stack className={styles.container}>
      <Stack className={clsx(styles.title, !readonly && styles.buttons)}>
        <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
          <AutocompleteExternalService
            className={styles.externalServiceSearch}
            size="small"
            variant="standard"
            fullWidth
            autoHighlight
            blurOnSelect
            clearOnBlur={false}
            clearOnEscape
            includeInputInList
            forcePopupIcon={false}
            placeholder="検索"
            renderOption={renderExternalServiceOption}
            value={hit}
            inputValue={hitInput}
            icon={({ ...props }) => <SearchIcon fontSize="small" {...props} />}
            onChange={handleHitExternalService}
            onInputChange={handleInputHitExternalService}
            slotProps={{
              popper: {
                className: styles.externalServiceSearchPopper,
                placement: 'bottom-start',
              },
            }}
          />
          {!readonly ? (
            <IconButton size="small" onClick={handleClickCreateExternalService}>
              <AddIcon />
            </IconButton>
          ) : null}
        </Stack>
      </Stack>
      <List ref={ref} dense={dense} className={styles.externalServices}>
        {allExternalServices.map(externalService => (
          <ExternalServiceListColumnBodyListItem
            key={externalService.id}
            className={styles.externalService}
            dense={dense}
            disabled={Boolean(disabledExternalService?.(externalService))}
            selected={!creating && (editing ?? active)?.id === externalService.id}
            primary={externalService.name}
            onClick={() => handleClickExternalService(externalService)}
          >
            {!readonly ? (
              <>
                <IconButton
                  className={styles.externalServiceButton}
                  size="small"
                  onMouseDown={handleMouseDownEditExternalService}
                  onClick={e => handleClickEditExternalService(e, externalService)}
                >
                  <EditOutlinedIcon fontSize={dense ? 'small' : 'medium'} />
                </IconButton>
                <IconButton
                  className={styles.externalServiceButton}
                  size="small"
                  onMouseDown={handleMouseDownDeleteExternalService}
                  onClick={e => handleClickDeleteExternalService(e, externalService)}
                >
                  <DeleteOutlinedIcon fontSize={dense ? 'small' : 'medium'} />
                </IconButton>
              </>
            ) : null}
          </ExternalServiceListColumnBodyListItem>
        ))}
        {creating ? (
          <ExternalServiceListColumnBodyListItem
            className={styles.externalService}
            dense={dense}
            selected
            primary="新しいサービス"
          />
        ) : null}
      </List>
    </Stack>
  )
}

export interface ExternalServiceColumn {
  readonly creating: boolean
  readonly editing: ExternalService | null
  readonly active: ExternalService | null
  readonly hit: ExternalService | null
  readonly hitInput: string
}

export interface ExternalServiceListColumnBodyListProps extends ExternalServiceColumn {
  readonly readonly: boolean
  readonly dense: boolean
  readonly disabled?: (externalService: ExternalService) => boolean
  readonly onHit?: (externalService: ExternalService | null) => void
  readonly onSelect?: (externalService: ExternalService) => void
  readonly create: () => void
  readonly show: (externalService: ExternalService) => void
  readonly edit: (externalService: ExternalService) => void
  readonly delete: (externalService: ExternalService) => void
  readonly setColumn: (column: ExternalServiceColumn) => void
}

export default ExternalServiceListColumnBodyList
