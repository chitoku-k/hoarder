'use client'

import type { FunctionComponent } from 'react'
import clsx from 'clsx'
import Link from 'next/link'
import Stack from '@mui/material/Stack'
import LaunchIcon from '@mui/icons-material/Launch'
import LinkIcon from '@mui/icons-material/Link'

import type {
  ExternalMetadata,
  ExternalMetadataBluesky,
  ExternalMetadataFantia,
  ExternalMetadataMastodon,
  ExternalMetadataMisskey,
  ExternalMetadataNijie,
  ExternalMetadataPixiv,
  ExternalMetadataPixivFanbox,
  ExternalMetadataPleroma,
  ExternalMetadataSeiga,
  ExternalMetadataSkeb,
  ExternalMetadataThreads,
  ExternalMetadataWebsite,
  ExternalMetadataX,
  ExternalMetadataXfolio,
  ExternalService,
  Source,
} from '@/types'

import styles from './styles.module.scss'

const builders: Builder[] = [
  {
    kind: 'bluesky',
    build: (_externalService, params) => {
      const { bluesky } = params as ExternalMetadataBluesky
      return `https://bsky.app/profile/${bluesky.creatorId}/post/${bluesky.id}`
    },
  },
  {
    kind: 'fantia',
    build: (_externalService, params) => {
      const { fantia } = params as ExternalMetadataFantia
      return `https://fantia.jp/posts/${fantia.id}`
    },
  },
  {
    kind: 'mastodon',
    build: (externalService, params) => {
      if (!externalService.baseUrl) {
        return null
      }
      const { mastodon } = params as ExternalMetadataMastodon
      return `${externalService.baseUrl}/@${mastodon.creatorId}/${mastodon.id}`
    },
  },
  {
    kind: 'misskey',
    build: (externalService, params) => {
      if (!externalService.baseUrl) {
        return null
      }
      const { misskey } = params as ExternalMetadataMisskey
      return `${externalService.baseUrl}/notes/${misskey.id}`
    },
  },
  {
    kind: 'nijie',
    build: (_externalService, params) => {
      const { nijie } = params as ExternalMetadataNijie
      return `https://nijie.info/view.php?id=${nijie.id}`
    },
  },
  {
    kind: 'pixiv',
    build: (_externalService, params) => {
      const { pixiv } = params as ExternalMetadataPixiv
      return `https://www.pixiv.net/artworks/${pixiv.id}`
    },
  },
  {
    kind: 'pixiv_fanbox',
    build: (_externalService, params) => {
      const { pixiv_fanbox } = params as ExternalMetadataPixivFanbox
      return `https://${pixiv_fanbox.creatorId}.fanbox.cc/posts/${pixiv_fanbox.id}`
    },
  },
  {
    kind: 'pleroma',
    build: (externalService, params) => {
      if (!externalService.baseUrl) {
        return null
      }
      const { pleroma } = params as ExternalMetadataPleroma
      return `${externalService.baseUrl}/notice/${pleroma.id}`
    },
  },
  {
    kind: 'seiga',
    build: (_externalService, params) => {
      const { seiga } = params as ExternalMetadataSeiga
      return `https://seiga.nicovideo.jp/seiga/im${seiga.id}`
    },
  },
  {
    kind: 'skeb',
    build: (_externalService, params) => {
      const { skeb } = params as ExternalMetadataSkeb
      return `https://skeb.jp/@${skeb.creatorId}/works/${skeb.id}`
    },
  },
  {
    kind: 'threads',
    build: (_externalService, params) => {
      const { threads } = params as ExternalMetadataThreads
      return `https://www.threads.net/@${threads.creatorId ?? ''}/post/${threads.id}`
    },
  },
  {
    kind: 'website',
    build: (_externalService, params) => {
      const { website } = params as ExternalMetadataWebsite
      return website.url
    },
  },
  {
    kind: 'x',
    build: (_externalService, params) => {
      const { x } = params as ExternalMetadataX
      return `https://x.com/${x.creatorId ?? 'i'}/status/${x.id}`
    },
  },
  {
    kind: 'xfolio',
    build: (_externalService, params) => {
      const { xfolio } = params as ExternalMetadataXfolio
      return `https://xfolio.jp/portfolio/@${xfolio.creatorId}/works/${xfolio.id}`
    },
  },
]

export const buildURL = (externalService: ExternalService, externalMetadata: ExternalMetadata): string | null => {
  for (const { kind, build } of builders) {
    if (externalService.kind === kind) {
      return build(externalService, externalMetadata)
    }
  }

  return null
}

export const displayURL = (url: string): string => url.replace(/^https?:\/\/(?:www\.)?/, '')

const MediumItemMetadataSourceItem: FunctionComponent<MediumItemMetadataSourceItemProps> = ({
  source,
  noLink,
  noLaunch,
}) => {
  const url = buildURL(source.externalService, source.externalMetadata)

  return (
    <Stack direction="row" alignItems="start">
      <LinkIcon className={styles.icon} fontSize="small" />
      {noLink && url ? (
        <span className={clsx(styles.item, styles.noLink)}>
          <span className={styles.text}>{displayURL(url)}</span>
          {!noLaunch ? (
            <Link className={styles.link} href={url} target="_blank">
              <LaunchIcon className={styles.launch} fontSize="inherit" />
            </Link>
          ) : null}
        </span>
      ) : url ? (
        <Link className={clsx(styles.item, styles.link)} href={url} target="_blank">
          <span className={styles.text}>{displayURL(url)}</span>
          {!noLaunch ? (
            <LaunchIcon className={styles.launch} fontSize="inherit" />
          ) : null}
        </Link>
      ) : source.externalMetadata && typeof source.externalMetadata === 'object' && 'custom' in source.externalMetadata ? (
        <span className={clsx(styles.item, styles.noLink)}>
          <span className={styles.text}>
            {typeof source.externalMetadata.custom === 'string' ? source.externalMetadata.custom : JSON.stringify(source.externalMetadata)}
          </span>
        </span>
      ) : JSON.stringify(source.externalMetadata)}
    </Stack>
  )
}

export interface MediumItemMetadataSourceItemProps {
  source: Source
  noLink?: boolean
  noLaunch?: boolean
}

interface Builder {
  kind: string
  build: (externalService: ExternalService, params: unknown) => string | null,
}

export default MediumItemMetadataSourceItem
