import type { FunctionComponent } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'

const MediumItemFileUploadAbortDialogBody: FunctionComponent<MediumItemFileUploadAbortDialogBodyProps> = ({
  abort,
  close,
}) => (
  <>
    <DialogContent>
      <DialogContentText>
        アップロードを取り消しますか？
      </DialogContentText>
    </DialogContent>
    <DialogActions>
      <Button onClick={close} autoFocus>キャンセル</Button>
      <Button color="error" onClick={abort}>取り消す</Button>
    </DialogActions>
  </>
)

export interface MediumItemFileUploadAbortDialogBodyProps {
  readonly abort: () => void
  readonly close: () => void
}

export default MediumItemFileUploadAbortDialogBody
