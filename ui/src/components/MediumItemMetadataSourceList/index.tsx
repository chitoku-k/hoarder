'use client'

import type { FunctionComponent } from 'react'
import { useCallback } from 'react'
import Button from '@mui/material/Button'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

import MediumItemMetadataHeader from '@/components/MediumItemMetadataHeader'
import SourceURL from '@/components/SourceURL'
import type { ExternalService, Medium, Source } from '@/types'

import styles from './styles.module.scss'

const MediumItemMetadataSourceList: FunctionComponent<MediumItemMetadataSourceListProps> = ({
  medium,
  edit,
}) => {
  const handleClickEdit = useCallback(() => {
    edit()
  }, [ edit ])

  const sources = medium.sources ?? []
  const groups = sources.reduce((groups, source) => {
    const group = groups.find(s => s.externalService.id === source.externalService.id)
    if (group) {
      group.sources.push(source)
    } else {
      groups.push({
        externalService: source.externalService,
        sources: [ source ],
      })
    }
    return groups
  }, [] as SourceGroup[])

  return (
    <Stack>
      <MediumItemMetadataHeader title="ソース">
        <Button onClick={handleClickEdit}>編集</Button>
      </MediumItemMetadataHeader>
      <Stack spacing={4}>
        {groups.length ? groups.map(({ externalService, sources }) => (
          <Stack key={externalService.id}>
            <Stack className={styles.header} direction="row" alignItems="center">
              <Typography className={styles.title} variant="h4">{externalService.name}</Typography>
            </Stack>
            <Stack spacing={0.5}>
              {sources.map(source => (
                <Stack key={source.id} direction="row" alignItems="center">
                  <SourceURL source={source} />
                  <Stack className={styles.buttonArea} />
                </Stack>
              ))}
            </Stack>
          </Stack>
        )) : (
          <Stack>
            未設定
          </Stack>
        )}
      </Stack>
    </Stack>
  )
}

export interface MediumItemMetadataSourceListProps {
  medium: Medium
  edit: () => void
}

interface SourceGroup {
  externalService: ExternalService
  sources: Source[]
}

export default MediumItemMetadataSourceList
