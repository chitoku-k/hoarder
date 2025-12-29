'use client'

import type { FunctionComponent } from 'react'
import Button from '@mui/material/Button'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogContentText from '@mui/material/DialogContentText'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

import DateTime from '@/components/DateTime'
import Image from '@/components/Image'
import ImageBodyBlob from '@/components/ImageBodyBlob'
import ImageBodyNext from '@/components/ImageBodyNext'
import { useFilesize } from '@/hooks'

import styles from './styles.module.scss'

const MediumItemFileOverwriteDialogBody: FunctionComponent<MediumItemFileOverwriteDialogBodyProps> = ({
  uploading,
  existing,
  overwrite,
  close,
}) => {
  const filesize = useFilesize()

  return (
    <>
      <DialogContent>
        {overwrite ? (
          <DialogContentText>
            同じファイル名のメディアを置き換えますか？
          </DialogContentText>
        ) : (
          <DialogContentText>
            同じファイル名のメディアがすでに登録されています
          </DialogContentText>
        )}
        <Stack className={styles.files} spacing={2}>
          {existing ? (
            <Stack spacing={1}>
              <Typography>既にアップロード先にあるファイル:</Typography>
              <Stack spacing={2} direction="row">
                <Image className={styles.imageWrapper}>
                  <Stack className={styles.imageWrapper} alignItems="center" justifyContent="center" flexShrink={0}>
                    <ImageBodyNext className={styles.image} src={existing.url} fill unoptimized alt="" />
                  </Stack>
                </Image>
                <Stack>
                  <Typography component="strong" fontWeight="bold">{existing.name}</Typography>
                  {typeof existing.size === 'number' ? (
                    <Typography className={styles.description}>サイズ: {filesize(existing.size)}</Typography>
                  ) : null}
                  {existing.lastModified ? (
                    <Typography className={styles.description}>
                      更新日時: <DateTime date={existing.lastModified} format="Pp" />
                    </Typography>
                  ) : null}
                </Stack>
              </Stack>
            </Stack>
          ) : null}
          <Stack spacing={1}>
            <Typography>アップロード中のファイル:</Typography>
            <Stack spacing={2} direction="row">
              <Image className={styles.imageWrapper}>
                <Stack className={styles.imageWrapper} alignItems="center" justifyContent="center" flexShrink={0}>
                  <ImageBodyBlob className={styles.image} src={uploading.blob} alt="" />
                </Stack>
              </Image>
              <Stack>
                <Typography component="strong" fontWeight="bold">{uploading.name}</Typography>
                <Typography className={styles.description}>サイズ: {filesize(uploading.size)}</Typography>
                <Typography className={styles.description}>
                  更新日時: <DateTime date={uploading.lastModified} format="Pp" />
                </Typography>
              </Stack>
            </Stack>
          </Stack>
        </Stack>
      </DialogContent>
      {overwrite ? (
        <DialogActions>
          <Button onClick={close} autoFocus>キャンセル</Button>
          <Button onClick={overwrite}>置き換える</Button>
        </DialogActions>
      ) : (
        <DialogActions>
          <Button onClick={close} autoFocus>閉じる</Button>
        </DialogActions>
      )}
    </>
  )
}

export interface MediumItemFileOverwriteDialogBodyProps {
  readonly uploading: {
    readonly name: string
    readonly size: number
    readonly lastModified: Date
    readonly blob: Blob
  }
  readonly existing: {
    readonly name: string
    readonly size: number | null
    readonly lastModified: Date | null
    readonly url: string
  } | null
  readonly overwrite?: () => void
  readonly close: () => void
}

export default MediumItemFileOverwriteDialogBody
