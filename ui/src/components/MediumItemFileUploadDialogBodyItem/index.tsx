'use client'

import type { ChangeEvent, FunctionComponent } from 'react'
import { useCallback } from 'react'
import LinearProgress from '@mui/material/LinearProgress'
import Stack from '@mui/material/Stack'
import TableCell from '@mui/material/TableCell'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import CloudDoneOutlinedIcon from '@mui/icons-material/CloudDoneOutlined'
import CloudOffOutlinedIcon from '@mui/icons-material/CloudOffOutlined'
import CloudQueueOutlinedIcon from '@mui/icons-material/CloudQueueOutlined'
import CloudUploadOutlinedIcon from '@mui/icons-material/CloudUploadOutlined'

import Image from '@/components/Image'
import ImageBodyBlob from '@/components/ImageBodyBlob'
import { MEDIUM_REPLICA_DECODE_FAILED, MEDIUM_REPLICA_UNSUPPORTED, REPLICA_ORIGINAL_URL_DUPLICATE, useError, useFilesize } from '@/hooks'
import type { ReplicaUploadProgress, ReplicaUploadStatus } from '@/components/MediumItemFileUploadDialogBody'
import type { ReplicaCreate } from '@/components/MediumItemImageEdit'

import styles from './styles.module.scss'

const MediumItemFileUploadDialogBodyItem: FunctionComponent<MediumItemFileUploadDialogBodyItemProps> = ({
  replica,
  status,
  progress,
  error,
  nameValidationError,
  onChangeName,
}) => {
  const { graphQLError } = useError()

  const filesize = useFilesize()
  const handleChangeName = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    onChangeName?.(e.currentTarget.value)
  }, [ onChangeName ])

  const mediumReplicaDecodeFailed = graphQLError(error, MEDIUM_REPLICA_DECODE_FAILED)
  const mediumReplicaUnsupported = graphQLError(error, MEDIUM_REPLICA_UNSUPPORTED)
  const replicaOriginalUrlDuplicate = graphQLError(error, REPLICA_ORIGINAL_URL_DUPLICATE)

  return (
    <>
      <TableCell className={styles.imageCell}>
        <Image className={styles.imageWrapper}>
          <Stack className={styles.imageWrapper} alignItems="center" justifyContent="center">
            <ImageBodyBlob className={styles.image} src={replica.blob} alt="" />
          </Stack>
        </Image>
      </TableCell>
      <TableCell>
        <TextField
          className={styles.filename}
          fullWidth
          value={replica.name}
          error={Boolean(nameValidationError)}
          helperText={nameValidationError}
          onChange={handleChangeName}
          disabled={Boolean(status) && status !== 'error' && status !== 'aborted'}
        />
      </TableCell>
      <TableCell className={styles.filesizeCell} align="right">
        <Typography>{filesize(replica.size)}</Typography>
      </TableCell>
      <TableCell className={styles.progressCell}>
        <Stack spacing={1}>
          {status === 'uploading' && progress ? (
            <>
              <Stack spacing={0.75} direction="row" alignItems="center">
                <CloudUploadOutlinedIcon fontSize="small" />
                <Typography>アップロード中... ({Math.ceil(progress.loaded / progress.total * 100)}%)</Typography>
              </Stack>
              <LinearProgress variant="determinate" value={progress.loaded / progress.total * 100} />
            </>
          ) : status === 'creating' ? (
            <>
              <Stack spacing={0.75} direction="row" alignItems="center">
                <CloudUploadOutlinedIcon fontSize="small" />
                <Typography>処理中...</Typography>
              </Stack>
              <LinearProgress variant="indeterminate" />
            </>
          ) : status === 'done' ? (
            <>
              <Stack spacing={0.75} direction="row" alignItems="center">
                <CloudDoneOutlinedIcon fontSize="small" />
                <Typography>完了</Typography>
              </Stack>
              <LinearProgress variant="determinate" value={100} />
            </>
          ) : status === 'aborted' ? (
            <>
              <Stack spacing={0.75} direction="row" alignItems="center">
                <CloudOffOutlinedIcon fontSize="small" />
                <Typography>キャンセル</Typography>
              </Stack>
              <LinearProgress className={styles.progress} variant="determinate" color="inherit" value={0} />
            </>
          ) : status === 'error' ? (
            <>
              <Stack spacing={0.75} direction="row" alignItems="center">
                <CloudOffOutlinedIcon fontSize="small" />
                {mediumReplicaDecodeFailed ? (
                  <Typography>エラー: メディアが読み込めません</Typography>
                ) : mediumReplicaUnsupported ? (
                  <Typography>エラー: サポートされていません</Typography>
                ) : replicaOriginalUrlDuplicate ? (
                  <Typography>エラー: 登録済みのメディア</Typography>
                ) : (
                  <Typography>エラー</Typography>
                )}
              </Stack>
              <LinearProgress className={styles.progress} variant="determinate" color="error" value={0} />
            </>
          ) : (
            <>
              <Stack spacing={0.75} direction="row" alignItems="center">
                <CloudQueueOutlinedIcon fontSize="small" />
                <Typography>待機中</Typography>
              </Stack>
              <LinearProgress className={styles.progress} variant="determinate" color="inherit" value={0} />
            </>
          )}
        </Stack>
      </TableCell>
    </>
  )
}

export interface MediumItemFileUploadDialogBodyItemProps {
  replica: ReplicaCreate
  status: ReplicaUploadStatus
  progress: ReplicaUploadProgress | null
  error: unknown
  nameValidationError?: string | null
  onChangeName?: (name: string) => void
}

export default MediumItemFileUploadDialogBodyItem
