import type { FunctionComponent } from 'react'
import { Suspense } from 'react'
import { ErrorBoundary } from 'react-error-boundary'
import Dialog from '@mui/material/Dialog'

import type { TagSelectDialogBodyProps } from '@/components/TagSelectDialogBody'
import TagSelectDialogBody from '@/components/TagSelectDialogBody'
import TagSelectDialogError from '@/components/TagSelectDialogError'
import TagSelectDialogLoading from '@/components/TagSelectDialogLoading'

import styles from './styles.module.scss'

const TagSelectDialog: FunctionComponent<TagSelectDialogProps> = ({
  close,
  ...props
}) => (
  <Dialog className={styles.dialog} open onClose={close}>
    <ErrorBoundary fallback={<TagSelectDialogError close={close} />}>
      <Suspense fallback={<TagSelectDialogLoading />}>
        <TagSelectDialogBody close={close} {...props} />
      </Suspense>
    </ErrorBoundary>
  </Dialog>
)

export type TagSelectDialogProps = TagSelectDialogBodyProps

export default TagSelectDialog
