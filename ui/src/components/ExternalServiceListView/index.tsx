'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import clsx from 'clsx'
import Card from '@mui/material/Card'
import Grid from '@mui/material/Grid'

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
    hit: null,
    hitInput: '',
  })

  const [ creating, setCreating ] = useState(false)
  const [ showingExternalService, setShowingExternalService ] = useState<ExternalService | null>(initial ?? null)
  const [ editingExternalService, setEditingExternalService ] = useState<ExternalService | null>(null)
  const [ deletingExternalService, setDeletingExternalService ] = useState<ExternalService | null>(null)

  const closeCreateExternalService = useCallback(() => {
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

  const closeEditExternalService = useCallback(() => {
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

  const closeDeleteExternalService = useCallback(() => {
    setDeletingExternalService(null)
  }, [])

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

  const showExternalService = useCallback((externalService: ExternalService) => {
    setCreating(false)
    setShowingExternalService(externalService)
    setEditingExternalService(null)
    setColumn(column => ({
      ...column,
      creating: false,
      editing: null,
      active: externalService,
      hitInput: '',
    }))
  }, [])

  const deleteExternalService = useCallback((externalService: ExternalService) => {
    setDeletingExternalService(externalService)
  }, [])

  const handleEditExternalService = useCallback((externalService: ExternalService) => {
    if (column.active?.id === externalService.id) {
      setShowingExternalService(externalService)
    }
  }, [ column ])

  const handleDeleteExternalService = useCallback((externalService: ExternalService) => {
    if (showingExternalService?.id == externalService.id) {
      closeShowExternalService()
    }
    if (editingExternalService?.id == externalService.id) {
      closeEditExternalService()
    }
  }, [ showingExternalService, closeEditExternalService, editingExternalService, closeShowExternalService ])

  const handleHitExternalService = useCallback((hit: ExternalService | null) => {
    closeCreateExternalService()
    closeEditExternalService()

    if (hit) {
      showExternalService(hit)
      setColumn(column => ({
        ...column,
        hitInput: hit.name,
      }))
    }
  }, [ closeCreateExternalService, closeEditExternalService, showExternalService ])

  const handleSelectExternalService = useCallback((externalService: ExternalService) => {
    onSelect?.(externalService)
  }, [ onSelect ])

  return (
    <Card className={clsx(styles.container, className)}>
      <Grid className={styles.wrapper} container>
        <ExternalServiceListColumn className={clsx(styles.column, styles.listColumn)} size={{ xs: 4, lg: 3 }}>
          <ExternalServiceListColumnBodyList
            {...column}
            readonly={Boolean(readonly)}
            dense={Boolean(dense)}
            disabled={disabled}
            onHit={handleHitExternalService}
            onSelect={handleSelectExternalService}
            create={createExternalService}
            show={showExternalService}
            edit={editExternalService}
            delete={deleteExternalService}
            setColumn={setColumn}
          />
        </ExternalServiceListColumn>
        <ExternalServiceListColumn key={showingExternalService?.id ?? editingExternalService?.id ?? String(creating)} className={styles.column} size={{ xs: 8, lg: 9 }}>
          {showingExternalService ? (
            <ExternalServiceListColumnBodyShow externalService={showingExternalService} edit={editExternalService} />
          ) : null}
          {creating ? (
            <ExternalServiceListColumnBodyCreate close={closeCreateExternalService} />
          ) : null}
          {editingExternalService ? (
            <ExternalServiceListColumnBodyEdit externalService={editingExternalService} close={closeEditExternalService} onEdit={handleEditExternalService} />
          ) : null}
          {deletingExternalService ? (
            <ExternalServiceDeleteDialog key={deletingExternalService.id} externalService={deletingExternalService} close={closeDeleteExternalService} onDelete={handleDeleteExternalService} />
          ) : null}
        </ExternalServiceListColumn>
      </Grid>
    </Card>
  )
}

export interface ExternalServiceListViewProps {
  readonly className?: string
  readonly initial?: ExternalService
  readonly readonly?: boolean
  readonly dense?: boolean
  readonly disabled?: (externalService: ExternalService) => boolean
  readonly onSelect?: (externalService: ExternalService) => void
}

export default ExternalServiceListView
