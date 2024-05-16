'use client'

import type { FunctionComponent, MouseEvent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import IconButton from '@mui/material/IconButton'
import List from '@mui/material/List'
import Stack from '@mui/material/Stack'
import DeleteOutlinedIcon from '@mui/icons-material/DeleteOutlined'
import EditOutlinedIcon from '@mui/icons-material/EditOutlined'

import ExternalServiceListColumnBodyListItem from '@/components/ExternalServiceListColumnBodyListItem'
import { useAllExternalServices } from '@/hooks'
import type { ExternalService } from '@/types'

import styles from './styles.module.scss'

const ExternalServiceListColumnBodyList: FunctionComponent<ExternalServiceListColumnBodyListProps> = ({
  creating,
  editing,
  active,
  readonly,
  dense,
  disabled: disabledExternalService,
  onSelect: onSelectExternalService,
  show: showExternalService,
  create: createExternalService,
  edit: editExternalService,
  delete: deleteExternalService,
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
    showExternalService(externalService)
  }

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

  return (
    <Stack className={styles.container}>
      <Stack className={styles.buttons}>
        {!readonly ? (
          <Button variant="outlined" onClick={handleClickCreateExternalService}>
            新規作成
          </Button>
        ) : null}
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
  creating: boolean
  editing: ExternalService | null
  active: ExternalService | null
}

export interface ExternalServiceListColumnBodyListProps extends ExternalServiceColumn {
  readonly: boolean
  dense: boolean
  disabled?: (externalService: ExternalService) => boolean
  onSelect?: (externalService: ExternalService) => void
  create: () => void
  show: (externalService: ExternalService) => void
  edit: (externalService: ExternalService) => void
  delete: (externalService: ExternalService) => void
}

export default ExternalServiceListColumnBodyList
