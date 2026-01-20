'use client'

import type { FunctionComponent } from 'react'
import { useCallback, useRef } from 'react'
import Dialog from '@mui/material/Dialog'

import type { MediumItemFileUploadDialogBodyProps, MediumItemFileUploadDialogBodyRef, UploadStatus } from '@/components/MediumItemFileUploadDialogBody'
import MediumItemFileUploadDialogBody from '@/components/MediumItemFileUploadDialogBody'

import styles from './styles.module.scss'

const MediumItemFileUploadDialog: FunctionComponent<MediumItemFileUploadDialogProps> = ({
  close,
  ...props
}) => {
  const ref = useRef<MediumItemFileUploadDialogBodyRef>(null)
  const handleClose = useCallback(() => {
    ref.current?.close()
  }, [ ref ])

  return (
    <Dialog className={styles.dialog} open onClose={handleClose}>
      <MediumItemFileUploadDialogBody ref={ref} close={close} {...props} />
    </Dialog>
  )
}

export type MediumItemFileUploadDialogProps = MediumItemFileUploadDialogBodyProps
export type MediumItemFileUploadStatus = UploadStatus

export default MediumItemFileUploadDialog
