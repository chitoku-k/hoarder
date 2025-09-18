'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback, useState } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import FormControlLabel from '@mui/material/FormControlLabel'
import FormGroup from '@mui/material/FormGroup'
import Radio from '@mui/material/Radio'
import RadioGroup from '@mui/material/RadioGroup'

import { useDeleteMedium } from '@/hooks'
import type { Medium } from '@/types'

import styles from './styles.module.scss'

const MediumDeleteDialogBody: FunctionComponent<MediumDeleteDialogBodyProps> = ({
  medium,
  close,
  onDelete,
}) => {
  const [ deleteMedium, { error, loading } ] = useDeleteMedium()
  const [ deleteObjects, setDeleteObjects ] = useState<boolean | null>(null)

  const handleChangeDeleteObjects = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    setDeleteObjects(e.currentTarget.value === 'true')
  }, [])

  const handleClickDelete = useCallback(async () => {
    try {
      await deleteMedium({ id: medium.id, deleteObjects })
      close()
      onDelete(medium)
    } catch (e) {
      console.error('Error deleting medium\n', e)
    }
  }, [ deleteMedium, medium, deleteObjects, close, onDelete ])

  const hasReplicas = Boolean(medium.replicas?.length)

  return error ? (
    <>
      <DialogContent>
        <DialogContentText>メディアを削除できませんでした</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>閉じる</Button>
      </DialogActions>
    </>
  ) : (
    <>
      <DialogContent>
        <DialogContentText>
          メディアを削除しますか？
        </DialogContentText>
        {hasReplicas ? (
          <FormGroup className={styles.form}>
            <RadioGroup value={deleteObjects} onChange={handleChangeDeleteObjects}>
              <FormControlLabel value={true} className={styles.label} control={<Radio />} label="アップロードされたメディアを削除する" />
              <FormControlLabel value={false} className={styles.label} control={<Radio />} label="アップロードされたメディアを削除しない" />
            </RadioGroup>
          </FormGroup>
        ) : null}
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>キャンセル</Button>
        <Button color="error" onClick={handleClickDelete} loading={loading} disabled={hasReplicas && deleteObjects === null}>削除</Button>
      </DialogActions>
    </>
  )
}

export interface MediumDeleteDialogBodyProps {
  readonly medium: Medium
  readonly close: () => void
  readonly onDelete: (medium: Medium) => void
}

export default MediumDeleteDialogBody
