'use client'

import type { ComponentType, FunctionComponent } from 'react'
import clsx from 'clsx'
import Link from '@mui/material/Link'
import Stack from '@mui/material/Stack'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import LaunchIcon from '@mui/icons-material/Launch'
import LinkIcon from '@mui/icons-material/Link'

import { buildURL } from './builder'
import styles from './styles.module.scss'

import type { ExternalService, Source } from '@/types'

const extractProps = (props: SourceURLProps): [ ExternalService, unknown ] | [ null, null ] => {
  if ('source' in props) {
    return [ props.source.externalService, props.source.externalMetadata ]
  }

  if ('externalService' in props && 'externalMetadata' in props) {
    return [ props.externalService, props.externalMetadata ]
  }

  return [ null, null ]
}

const displayURL = (url: string): string => url.replace(/^https?:\/\/(?:www\.)?/, '')

const SourceURL: FunctionComponent<SourceURLProps> = ({
  className,
  icon,
  noLink,
  noLaunch,
  ...props
}) => {
  const [ externalService, externalMetadata ] = extractProps(props)
  if (!externalService || !externalMetadata) {
    throw new Error('source or externalService and externalMetadata is required')
  }

  const Icon = icon ?? LinkIcon
  const url = buildURL(externalService, externalMetadata)

  return (
    <Stack className={className} direction="row" alignItems="start">
      <Icon className={styles.icon} fontSize="small" />
      {noLink && url ? (
        <span className={clsx(styles.item, styles.noLink)}>
          <span className={styles.text}>{displayURL(url)}</span>
          {!noLaunch ? (
            <Link href={url} target="_blank" rel="noopener noreferrer" underline="none">
              <LaunchIcon className={styles.launch} fontSize="inherit" />
            </Link>
          ) : null}
        </span>
      ) : url ? (
        <Link className={styles.item} href={url} target="_blank" rel="noopener noreferrer" underline="none">
          <span className={styles.text}>{displayURL(url)}</span>
          {!noLaunch ? (
            <LaunchIcon className={styles.launch} fontSize="inherit" />
          ) : null}
        </Link>
      ) : typeof externalMetadata === 'object' && 'custom' in externalMetadata ? (
        <span className={clsx(styles.item, styles.noLink)}>
          <span className={styles.text}>
            {typeof externalMetadata.custom === 'string' ? externalMetadata.custom : JSON.stringify(externalMetadata)}
          </span>
        </span>
      ) : JSON.stringify(externalMetadata)}
    </Stack>
  )
}

interface SourceURLPropsBase {
  className?: string
  icon?: ComponentType<SvgIconProps>
  noLink?: boolean
  noLaunch?: boolean
}

interface SourceURLPropsBySource extends SourceURLPropsBase {
  source: Source
}

interface SourceURLPropsByExternalMetadata extends SourceURLPropsBase {
  externalService: ExternalService
  externalMetadata: unknown
}

export type SourceURLProps = SourceURLPropsBySource | SourceURLPropsByExternalMetadata

export default SourceURL
