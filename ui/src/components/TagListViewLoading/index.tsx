import type { FunctionComponent } from 'react'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'

const TagListViewLoading: FunctionComponent = () => (
  <Stack flexGrow={1} alignItems="center" justifyContent="center">
    <CircularProgress size={48} />
  </Stack>
)

export default TagListViewLoading
