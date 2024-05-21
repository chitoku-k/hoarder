'use client'

import type { FunctionComponent } from 'react'
import Link from '@mui/material/Link'
import Stack from '@mui/material/Stack'
import AddLinkIcon from '@mui/icons-material/AddLink'
import LaunchIcon from '@mui/icons-material/Launch'

import { buildURL, displayURL } from '@/components/MediumItemMetadataSourceItem'
import type { ExternalMetadata, ExternalService } from '@/types'

import styles from './styles.module.scss'

const MediumItemMetadataSourceItemNew: FunctionComponent<MediumItemMetadataSourceItemNewProps> = ({
  externalService,
  externalMetadata,
  noLaunch,
}) => {
  const url = buildURL(externalService, externalMetadata)

  return (
    <Stack direction="row" alignItems="start">
      <AddLinkIcon className={styles.icon} fontSize="small" />
      <span className={styles.item}>
        {url ? (
          <>
            <span className={styles.text}>{displayURL(url)}</span>
            {!noLaunch ? (
              <Link href={url} target="_blank" rel="noopener noreferrer" underline="none">
                <LaunchIcon className={styles.launch} fontSize="inherit" />
              </Link>
            ) : null}
          </>
      ) : externalMetadata && typeof externalMetadata === 'object' && 'custom' in externalMetadata ? (
          <span className={styles.text}>
            {typeof externalMetadata.custom === 'string' ? externalMetadata.custom : JSON.stringify(externalMetadata.custom)}
          </span>
        ) : JSON.stringify(externalMetadata)}
      </span>
    </Stack>
  )
}

export interface MediumItemMetadataSourceItemNewProps {
  externalService: ExternalService
  externalMetadata: ExternalMetadata
  noLaunch?: boolean
}

export default MediumItemMetadataSourceItemNew
