'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import clsx from 'clsx'
import Card from '@mui/material/Card'
import Grid from '@mui/material/Unstable_Grid2'

import type { ExternalServiceColumn } from '@/components/ExternalServiceListColumn'
import ExternalServiceDeleteDialog from '@/components/ExternalServiceDeleteDialog'
import ExternalServiceListColumn from '@/components/ExternalServiceListColumn'
import ExternalServiceListColumnBodyCreate from '@/components/ExternalServiceListColumnBodyCreate'
import ExternalServiceListColumnBodyEdit from '@/components/ExternalServiceListColumnBodyEdit'
import ExternalServiceListColumnBodyList from '@/components/ExternalServiceListColumnBodyList'
import ExternalServiceListColumnBodyShow from '@/components/ExternalServiceListColumnBodyShow'
import type { ExternalService } from '@/types'

import styles from './styles.module.scss'

const ExternalServiceListView: FunctionComponent<ExternalServiceListViewProps> = ({
  className,
  initial,
  readonly,
  dense,
  disabled,
  onSelect,
}) => {
  const [ column, setColumn ] = useState<ExternalServiceColumn>({
    creating: false,
    editing: null,
    active: initial ?? null,
  })

  const [ creating, setCreating ] = useState(false)
  const [ showingExternalService, setShowingExternalService ] = useState<ExternalService | null>(initial ?? null)
  const [ editingExternalService, setEditingExternalService ] = useState<ExternalService | null>(null)
  const [ deletingExternalService, setDeletingExternalService ] = useState<ExternalService | null>(null)

  const createExternalService = useCallback(() => {
    setCreating(true)
    setShowingExternalService(null)
    setEditingExternalService(null)
    setColumn(column => ({
      ...column,
      creating: true,
      editing: null,
    }))
  }, [])

  const closeCreateExternalService = useCallback(() => {
    setCreating(false)
    setShowingExternalService(column.active)
    setEditingExternalService(null)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: null,
    }))
  }, [ column ])

  const showExternalService = useCallback((externalService: ExternalService) => {
    setCreating(false)
    setShowingExternalService(externalService)
    setEditingExternalService(null)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: null,
      active: externalService,
    }))
  }, [])

  const closeShowExternalService = useCallback(() => {
    setCreating(false)
    setShowingExternalService(null)
    setEditingExternalService(null)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: null,
      active: null,
    }))
  }, [])

  const editExternalService = useCallback((externalService: ExternalService) => {
    setCreating(false)
    setShowingExternalService(null)
    setEditingExternalService(externalService)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: externalService,
    }))
  }, [])

  const closeEditExternalService = useCallback(() => {
    setCreating(false)
    setShowingExternalService(column.active)
    setEditingExternalService(null)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: null,
    }))
  }, [ column ])

  const handleEditExternalService = useCallback((externalService: ExternalService) => {
    if (column.active?.id === externalService.id) {
      setShowingExternalService(externalService)
    }
  }, [ column ])

  const deleteExternalService = useCallback((externalService: ExternalService) => {
    setDeletingExternalService(externalService)
  }, [])

  const closeDeleteExternalService = useCallback(() => {
    setDeletingExternalService(null)
  }, [])

  const handleDeleteExternalService = useCallback((externalService: ExternalService) => {
    if (showingExternalService?.id == externalService.id) {
      closeShowExternalService()
    }
    if (editingExternalService?.id == externalService.id) {
      closeEditExternalService()
    }
  }, [ showingExternalService, closeEditExternalService, editingExternalService, closeShowExternalService ])

  return (
    <Card className={clsx(styles.container, className)}>
      <Grid className={styles.wrapper} container>
        <ExternalServiceListColumn className={clsx(styles.column, styles.listColumn)} xs={4} lg={3}>
          <ExternalServiceListColumnBodyList
            {...column}
            readonly={Boolean(readonly)}
            dense={Boolean(dense)}
            disabled={disabled}
            onSelect={onSelect}
            create={createExternalService}
            show={showExternalService}
            edit={editExternalService}
            delete={deleteExternalService}
          />
        </ExternalServiceListColumn>
        <ExternalServiceListColumn key={showingExternalService?.id ?? editingExternalService?.id ?? String(creating)} className={styles.column} xs={8} lg={9}>
          {showingExternalService ? (
            <ExternalServiceListColumnBodyShow externalService={showingExternalService} edit={editExternalService} />
          ) : null}
          {creating ? (
            <ExternalServiceListColumnBodyCreate close={closeCreateExternalService} />
          ) : null}
          {editingExternalService ? (
            <ExternalServiceListColumnBodyEdit externalService={editingExternalService} close={closeEditExternalService} onEdit={handleEditExternalService} />
          ) : null}
        </ExternalServiceListColumn>
      </Grid>
      {deletingExternalService ? (
        <ExternalServiceDeleteDialog key={deletingExternalService.id} externalService={deletingExternalService} close={closeDeleteExternalService} onDelete={handleDeleteExternalService} />
      ) : null}
    </Card>
  )
}

export interface ExternalServiceListViewProps {
  className?: string
  initial?: ExternalService,
  readonly?: boolean
  dense?: boolean
  disabled?: (externalService: ExternalService) => boolean
  onSelect?: (externalService: ExternalService) => void
}

export default ExternalServiceListView
