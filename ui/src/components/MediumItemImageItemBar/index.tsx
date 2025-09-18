'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { memo, useCallback } from 'react'
import IconButton from '@mui/material/IconButton'
import ImageListItemBar from '@mui/material/ImageListItemBar'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline'
import ArrowCircleDownIcon from '@mui/icons-material/ArrowCircleDown'
import ArrowCircleUpIcon from '@mui/icons-material/ArrowCircleUp'
import RemoveCircleOutlineIcon from '@mui/icons-material/RemoveCircleOutline'

import type { ReplicaCreate } from '@/components/MediumItemImageEdit'
import { isReplica } from '@/components/MediumItemImageEdit'
import type { Replica } from '@/types'

import styles from './styles.module.scss'

const MediumItemImageItemBar: FunctionComponent<MediumItemImageItemBarProps> = ({
  index,
  total,
  currentIndex,
  currentTotal,
  removing,
  replica,
  name,
  onChangeName,
  onMoveUp,
  onMoveDown,
  onRemove,
  onRestore,
}) => {
  const handleChangeName = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    onChangeName?.(index, e.currentTarget.value)
  }, [ onChangeName, index ])

  const handleClickMoveUp = useCallback(() => {
    onMoveUp(index)
  }, [ onMoveUp, index ])

  const handleClickMoveDown = useCallback(() => {
    onMoveDown(index)
  }, [ onMoveDown, index ])

  const handleClickRemove = useCallback(() => {
    onRemove(replica)
  }, [ onRemove, replica ])

  const handleClickRestore = useCallback(() => {
    if (isReplica(replica)) {
      onRestore?.(replica)
    }
  }, [ onRestore, replica ])

  return (
    <ImageListItemBar
      className={styles.bar}
      title={
        <Stack spacing={1.5} direction="row" alignItems="center">
          {currentIndex >= 0 ? (
            <Typography className={styles.title}>
              {currentIndex + 1}
              <span className={styles.all}>/{currentTotal}</span>
            </Typography>
          ) : (
            <Typography className={styles.title}>
              −
              <span className={styles.all}>/−</span>
            </Typography>
          )}
          {typeof name === 'string' ? (
            <TextField className={styles.name} variant="standard" value={name} onChange={handleChangeName} />
          ) : null}
        </Stack>
      }
      actionIcon={
        <Stack className={styles.actions} spacing={0.5} direction="row">
          {total > 1 ? (
            <>
              <IconButton className={styles.icon} onClick={handleClickMoveUp} disabled={index === 0}>
                <ArrowCircleUpIcon />
              </IconButton>
              <IconButton className={styles.icon} onClick={handleClickMoveDown} disabled={index === total - 1}>
                <ArrowCircleDownIcon />
              </IconButton>
            </>
          ) : null}
          {removing ? (
            <IconButton className={styles.icon} onClick={handleClickRestore}>
              <AddCircleOutlineIcon />
            </IconButton>
          ) : (
            <IconButton className={styles.icon} onClick={handleClickRemove}>
              <RemoveCircleOutlineIcon />
            </IconButton>
          )}
        </Stack>
      }
    />
  )
}

export interface MediumItemImageItemBarProps {
  readonly index: number
  readonly total: number
  readonly currentIndex: number
  readonly currentTotal: number
  readonly removing: boolean
  readonly replica: Replica | ReplicaCreate
  readonly name?: string
  readonly onChangeName?: (index: number, name: string) => void
  readonly onMoveUp: (index: number) => void
  readonly onMoveDown: (index: number) => void
  readonly onRemove: (replica: Replica | ReplicaCreate) => void
  readonly onRestore?: (replica: Replica) => void
}

export default memo(MediumItemImageItemBar)
