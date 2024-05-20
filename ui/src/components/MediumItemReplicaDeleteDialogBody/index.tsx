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

import styles from './styles.module.scss'

const MediumItemReplicaDeleteDialogBody: FunctionComponent<MediumItemReplicaDeleteDialogBodyProps> = ({
  save,
  close,
}) => {
  const [ deleteObjects, setDeleteObjects ] = useState<boolean | null>(null)

  const handleChangeDeleteObjects = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    setDeleteObjects(e.currentTarget.value === 'true')
  }, [])

  const handleClickSave = useCallback(() => {
    if (deleteObjects === null) {
      return
    }
    save(deleteObjects)
  }, [ deleteObjects, save ])

  return (
    <>
      <DialogContent>
        <DialogContentText>
          アップロードされたメディアを削除しますか？
        </DialogContentText>
        <FormGroup className={styles.form}>
          <RadioGroup value={deleteObjects} onChange={handleChangeDeleteObjects}>
            <FormControlLabel value={true} className={styles.label} control={<Radio />} label="アップロードされたメディアを削除する" />
            <FormControlLabel value={false} className={styles.label} control={<Radio />} label="アップロードされたメディアを削除しない" />
          </RadioGroup>
        </FormGroup>
      </DialogContent>
      <DialogActions>
        <Button onClick={close} autoFocus>キャンセル</Button>
        <Button color="error" onClick={handleClickSave} disabled={deleteObjects === null}>保存</Button>
      </DialogActions>
    </>
  )
}

export interface MediumItemReplicaDeleteDialogBodyProps {
  save: (deleteObject: boolean) => void
  close: () => void
}

export default MediumItemReplicaDeleteDialogBody
