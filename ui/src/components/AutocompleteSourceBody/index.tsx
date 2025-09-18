'use client'

import type { ComponentType, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useMemo, useState, useTransition } from 'react'
import clsx from 'clsx'
import { skipToken } from '@apollo/client/react'
import type { AutocompleteProps } from '@mui/material/Autocomplete'
import Autocomplete from '@mui/material/Autocomplete'
import CircularProgress from '@mui/material/CircularProgress'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import type { TextFieldVariants } from '@mui/material/TextField'
import TextField from '@mui/material/TextField'
import { debounce } from '@mui/material/utils'

import type { ExternalMetadataInput } from '@/graphql/types.generated'
import { useSource } from '@/hooks'
import type { ExternalService, Source } from '@/types'

import styles from './styles.module.scss'

export const isSource = (source: Source | SourceCreate) => 'id' in source

const builders = [
  {
    kind: 'bluesky',
    patterns: [
      /^https?:\/\/bsky\.app\/profile\/(?<creatorId>[^/?#]+)\/post\/(?<id>[^/?#]+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  } satisfies Builder<'bluesky'>,
  {
    kind: 'fantia',
    patterns: [
      /^(?<id>\d+)$/,
      /^https?:\/\/fantia\.jp\/posts\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  } satisfies Builder<'fantia'>,
  {
    kind: 'mastodon',
    patterns: [
      /^https?:\/\/(?:[^/]+)\/@(?<creatorId>[^/?#]+)\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  } satisfies Builder<'mastodon'>,
  {
    kind: 'misskey',
    patterns: [
      /^(?<id>[^/?#]+)$/,
      /^https?:\/\/(?:[^/]+)\/notes\/(?<id>[^/?#]+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  } satisfies Builder<'misskey'>,
  {
    kind: 'nijie',
    patterns: [
      /^(?<id>\d+)$/,
      /^https?:\/\/nijie\.info\/view\.php\?id=(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  } satisfies Builder<'nijie'>,
  {
    kind: 'pixiv',
    patterns: [
      /^(?<id>\d+)$/,
      /^https?:\/\/www\.pixiv\.net\/(?:artworks\/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  } satisfies Builder<'pixiv'>,
  {
    kind: 'pixiv_fanbox',
    patterns: [
      /^https?:\/\/www\.fanbox\.cc\/@(?<creatorId>[^.]+)\/posts\/(?<id>\d+)(?:[?#].*)?$/,
      /^https?:\/\/(?<creatorId>[^.]+)\.fanbox\.cc\/posts\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  } satisfies Builder<'pixiv_fanbox'>,
  {
    kind: 'pleroma',
    patterns: [
      /^(?<id>[^/?#]+)$/,
      /^https?:\/\/(?:[^/]+)\/notice\/(?<id>[^/?#]+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  } satisfies Builder<'pleroma'>,
  {
    kind: 'seiga',
    patterns: [
      /^(?:im)?(?<id>\d+)$/,
      /^https?:\/\/seiga\.nicovideo\.jp\/seiga\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  } satisfies Builder<'seiga'>,
  {
    kind: 'skeb',
    patterns: [
      /^https?:\/\/skeb\.jp\/@(?<creatorId>[^/]+)\/works\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  } satisfies Builder<'skeb'>,
  {
    kind: 'threads',
    patterns: [
      /^(?<id>[^/?#]+)$/,
      /^https?:\/\/(?:www\.threads\.net)\/(?<creatorId>[^/]+)\/post\/(?<id>[^/$#]+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  } satisfies Builder<'threads'>,
  {
    kind: 'website',
    patterns: [
      /^(?<url>https?:\/\/.+)$/,
    ],
    build: ({ url }) => ({ url }),
  } satisfies Builder<'website'>,
  {
    kind: 'x',
    patterns: [
      /^(?<id>\d+)$/,
      /^https?:\/\/(?:twitter\.com|x\.com)\/(?<creatorId>[^/]+)\/status\/(?<id>\d+)(?:[/?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId: creatorId !== 'i' ? creatorId : null }),
  } satisfies Builder<'x'>,
  {
    kind: 'xfolio',
    patterns: [
      /^https?:\/\/xfolio\.jp\/portfolio\/(?<creatorId>[^/]+)\/works\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  } satisfies Builder<'xfolio'>,
]

const buildExternalMetadata = (externalService: ExternalService, value: string): Partial<ExternalMetadataInput> | null => {
  if (!value) {
    return null
  }

  for (const { kind, patterns, build } of builders) {
    if (externalService.kind !== kind) {
      continue
    }

    for (const pattern of patterns) {
      let match: RegExpMatchArray | null
      if ((match = value.match(pattern)) && match.groups) {
        return {
          [kind]: build(match.groups),
        }
      }
    }

    return null
  }

  return {
    custom: value,
  }
}

const AutocompleteSourceBody: FunctionComponent<AutocompleteSourceBodyProps> = ({
  className,
  externalService,
  focus,
  label,
  placeholder,
  variant,
  icon: Icon,
  onChange: onChangeSource,
  ...props
}) => {
  const [ value, setValue ] = useState('')

  const [ loading, startTransition ] = useTransition()

  const ref = useCallback((input: HTMLInputElement | null) => {
    if (!focus) {
      return
    }
    input?.focus()
  }, [ focus ])

  const handleInputChange = useMemo(
    () => debounce(
      (_e: SyntheticEvent, value: string) => {
        startTransition(() => {
          setValue(value)
        })
      },
      100,
    ),
    [],
  )

  const handleChange = useCallback((_e: SyntheticEvent, source: Source | SourceCreate | null) => {
    onChangeSource?.(source)
  }, [ onChangeSource ])

  const externalMetadata = buildExternalMetadata(externalService, value)
  const source = useSource(externalMetadata ? { externalServiceID: externalService.id, externalMetadata } : skipToken)

  const options = source
    ? [ source ] satisfies Source[]
    : externalMetadata
      ? [ { externalService, externalMetadata } ] satisfies SourceCreate[]
      : []

  return (
    <Autocomplete
      {...props}
      className={clsx(className, styles.autocomplete)}
      isOptionEqualToValue={(option, value) => isSource(option) && isSource(value) && option.id === value.id}
      getOptionLabel={option => JSON.stringify(option.externalMetadata)}
      filterOptions={x => x}
      filterSelectedOptions
      options={options}
      loading={loading}
      onInputChange={handleInputChange}
      onChange={handleChange}
      renderInput={params => (
        <TextField
          {...params}
          label={label}
          placeholder={placeholder}
          variant={variant}
          inputRef={ref}
          slotProps={{
            input: {
              ...params.InputProps,
              startAdornment: Icon ? (
                <Icon className={styles.icon} fontSize="small" />
              ) : null,
              endAdornment: (
                <>
                  {loading ? <CircularProgress color="inherit" size={20} /> : null}
                  {params.InputProps.endAdornment}
                </>
              ),
            },
          }}
        />
      )}
    />
  )
}

export type SourceCreate = Pick<Source, 'externalService' | 'externalMetadata'>

export interface AutocompleteSourceBodyProps extends Omit<AutocompleteProps<Source | SourceCreate, false, boolean | undefined, false>, 'onChange' | 'options' | 'renderInput'> {
  readonly externalService: ExternalService
  readonly focus?: boolean
  readonly label?: string
  readonly placeholder?: string
  readonly variant?: TextFieldVariants
  readonly icon?: ComponentType<SvgIconProps>
  readonly onChange?: (source: Source | SourceCreate | null) => void
}

interface Builder<Kind extends keyof ExternalMetadataInput> {
  readonly kind: Kind
  readonly patterns: readonly RegExp[]
  readonly build: (params: Record<string, string>) => Partial<ExternalMetadataInput[Kind]>
}

export default AutocompleteSourceBody
