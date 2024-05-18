'use client'

import type { ComponentPropsWithoutRef, ComponentType, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useMemo, useState, useTransition } from 'react'
import type { AutocompleteProps } from '@mui/material/Autocomplete'
import Autocomplete from '@mui/material/Autocomplete'
import CircularProgress from '@mui/material/CircularProgress'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import type { TextFieldVariants } from '@mui/material/TextField'
import TextField from '@mui/material/TextField'
import debounce from '@mui/material/utils/debounce'

import MediumItemMetadataSourceItem from '@/components/MediumItemMetadataSourceItem'
import MediumItemMetadataSourceItemNew from '@/components/MediumItemMetadataSourceItemNew'
import { useSource, useSourceSkip } from '@/hooks'
import type { ExternalMetadataInput } from '@/hooks/types.generated'
import type { ExternalService, Source } from '@/types'

import styles from './styles.module.scss'

export const isSource = (source: Source | SourceCreate): source is Source => 'id' in source

const builders: Builder[] = [
  {
    kind: 'bluesky',
    patterns: [
      /^https?:\/\/bsky\.app\/profile\/(?<creatorId>[^\/?#]+)\/post\/(?<id>[^\/?#]+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  },
  {
    kind: 'fantia',
    patterns: [
      /^(?<id>\d+)$/,
      /^https?:\/\/fantia\.jp\/posts\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  },
  {
    kind: 'mastodon',
    patterns: [
      /^https?:\/\/(?:[^\/]+)\/@(?<creatorId>[^\/?#]+)\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  },
  {
    kind: 'misskey',
    patterns: [
      /^(?<id>[^\/?#]+)$/,
      /^https?:\/\/(?:[^\/]+)\/notes\/(?<id>[^\/?#]+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  },
  {
    kind: 'nijie',
    patterns: [
      /^(?<id>\d+)$/,
      /^https?:\/\/nijie\.info\/view\.php\?id=(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  },
  {
    kind: 'pixiv',
    patterns: [
      /^(?<id>\d+)$/,
      /^https?:\/\/www\.pixiv\.net\/(?:artworks\/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  },
  {
    kind: 'pixiv_fanbox',
    patterns: [
      /^https?:\/\/(?<creatorId>[^.]+)\.fanbox\.cc\/posts\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  },
  {
    kind: 'pleroma',
    patterns: [
      /^(?<id>[^\/?#]+)$/,
      /^https?:\/\/(?:[^\/]+)\/notice\/(?<id>[^\/?#]+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  },
  {
    kind: 'seiga',
    patterns: [
      /^(?:im)?(?<id>\d+)$/,
      /^https?:\/\/seiga\.nicovideo\.jp\/seiga\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id }) => ({ id }),
  },
  {
    kind: 'skeb',
    patterns: [
      /^https?:\/\/skeb\.jp\/@(?<creatorId>[^\/]+)\/works\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  },
  {
    kind: 'threads',
    patterns: [
      /^(?<id>[^\/?#]+)$/,
      /^https?:\/\/(?:www\.threads\.net)\/(?<creatorId>[^\/]+)\/post\/(?<id>[^\/$#]+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  },
  {
    kind: 'website',
    patterns: [
      /^(?<url>https?:\/\/.+)$/,
    ],
    build: ({ url }) => ({ url }),
  },
  {
    kind: 'x',
    patterns: [
      /^(?<id>\d+)$/,
      /^https?:\/\/(?:twitter\.com|x\.com)\/(?<creatorId>[^\/]+)\/status\/(?<id>\d+)(?:[\/?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId: creatorId !== 'i' ? creatorId : null }),
  },
  {
    kind: 'xfolio',
    patterns: [
      /^https?:\/\/xfolio\.jp\/portfolio\/(?<creatorId>[^\/]+)\/works\/(?<id>\d+)(?:[?#].*)?$/,
    ],
    build: ({ id, creatorId }) => ({ id, creatorId }),
  },
]

const buildExternalMetadata = (externalService: ExternalService, value: string): ExternalMetadataInput | null => {
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
          [externalService.kind]: build(match.groups),
        } as ExternalMetadataInput
      }
    }

    return null
  }

  return {
    custom: value,
  }
}

const AutocompleteSourceBody: FunctionComponent<AutocompleteSourceBodyProps> = ({
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

  const renderOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: Source | SourceCreate) => (
    <li key={key} {...props}>
      {isSource(option) ? (
        <MediumItemMetadataSourceItem
          source={option}
          noLink
          noLaunch
        />
      ) : (
        <MediumItemMetadataSourceItemNew
          externalService={option.externalService}
          externalMetadata={option.externalMetadata}
          noLaunch
        />
      )}
    </li>
  ), [])

  const externalMetadata = buildExternalMetadata(externalService, value)
  const source = externalMetadata
    ? useSource({ externalServiceID: externalService.id, externalMetadata })
    : useSourceSkip()

  const options: (Source | SourceCreate)[] = source
    ? [ source ]
    : externalMetadata
      ? [ { externalService, externalMetadata } ]
      : []

  return (
    <Autocomplete
      {...props}
      isOptionEqualToValue={(option, value) => isSource(option) && isSource(value) && option.id === value.id}
      getOptionLabel={option => JSON.stringify(option.externalMetadata)}
      renderOption={renderOption}
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
          InputProps={{
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
          }}
        />
      )}
    />
  )
}

export type SourceCreate = Pick<Source, 'externalService' | 'externalMetadata'>

export interface AutocompleteSourceBodyProps extends Omit<AutocompleteProps<Source | SourceCreate, false, boolean | undefined, false>, 'onChange' | 'options' | 'renderInput'> {
  externalService: ExternalService,
  focus?: boolean
  label?: string
  placeholder?: string
  variant?: TextFieldVariants
  icon?: ComponentType<SvgIconProps>,
  onChange?: (source: Source | SourceCreate | null) => void
}

type BuilderKind<E> = E extends E ? keyof E : never

interface Builder {
  kind: BuilderKind<ExternalMetadataInput>
  patterns: RegExp[]
  build: (params: Record<string, string>) => ExternalMetadataInput[BuilderKind<ExternalMetadataInput>]
}

export default AutocompleteSourceBody
