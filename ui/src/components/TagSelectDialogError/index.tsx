import type { FunctionComponent } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'

const TagSelectDialogError: FunctionComponent<TagSelectDialogErrorProps> = ({
  close,
}) => (
  <>
    <DialogContent>
      <DialogContentText>タグを取得できませんでした</DialogContentText>
    </DialogContent>
    <DialogActions>
      <Button onClick={close} autoFocus>閉じる</Button>
    </DialogActions>
  </>
)

export interface TagSelectDialogErrorProps {
  readonly close: () => void
}

export default TagSelectDialogError
