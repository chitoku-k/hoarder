'use client'

import type { ComponentType, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useEffect, useMemo, useState, useTransition } from 'react'
import { useCollator } from '@react-aria/i18n'
import clsx from 'clsx'
import type { AutocompleteInputChangeReason, AutocompleteProps } from '@mui/material/Autocomplete'
import Autocomplete, { createFilterOptions } from '@mui/material/Autocomplete'
import CircularProgress from '@mui/material/CircularProgress'
import IconButton from '@mui/material/IconButton'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import type { TextFieldVariants } from '@mui/material/TextField'
import TextField from '@mui/material/TextField'
import ArrowOutwardIcon from '@mui/icons-material/ArrowOutward'
import debounce from '@mui/material/utils/debounce'

import TagSelectDialog from '@/components/TagSelectDialog'
import { useMetadataLike, useMetadataLikeSkip, useAllTagTypes } from '@/hooks'
import type { MetadataLike } from '@/hooks'
import type { Source, Tag, TagType } from '@/types'

import styles from './styles.module.scss'

export const isMetadataSource = (option: Metadata): option is MetadataSource => 'source' in option
export const isMetadataTag = (option: Metadata): option is MetadataTag => 'tag' in option
export const isMetadataTagType = (option: Metadata): option is MetadataTagType => 'tagType' in option

function* useMetadata(
  sources: MetadataLike['sources'],
  tags: MetadataLike['tags'],
  tagTypes: TagType[],
  options?: {
    noSources?: boolean,
    noTags?: boolean,
    noTagTypes?: boolean,
  },
): Generator<Metadata> {
  const collator = useCollator()

  if (sources && !options?.noSources) {
    for (const source of sources.id) {
      yield { source }
    }

    for (const source of sources.url) {
      yield { source }
    }
  }

  if (tags && !options?.noTags) {
    const allTags = tags
      .toSorted((a, b) => collator.compare(a.kana, b.kana))
      .flatMap(tag => [ tag, ...tag.children.map(child => ({ ...child, parent: tag })) ])

    const ids = new Set<string>()
    for (const tag of allTags) {
      if (!ids.has(tag.id)) {
        ids.add(tag.id)
        yield { tag }
      }
    }
  }

  if (tagTypes && !options?.noTagTypes) {
    for (const tagType of tagTypes) {
      yield { tagType }
    }
  }
}

const AutocompleteMetadataBody: FunctionComponent<AutocompleteMetadataBodyProps> = ({
  className,
  focus,
  label,
  placeholder,
  variant,
  onChange: onChangeMetadata,
  onInputChange,
  noSources,
  noTags,
  noTagTypes,
  disabled,
  icon: Icon,
  inputValue,
  ...props
}) => {
  const [ value, setValue ] = useState('')
  const [ selecting, setSelecting ] = useState(false)

  const [ loading, startTransition ] = useTransition()

  const ref = useCallback((input: HTMLInputElement | null) => {
    if (!focus) {
      return
    }
    input?.focus()
  }, [ focus ])

  const isOptionEqualToValue = useCallback((option: Metadata, value: Metadata) => {
    if (isMetadataSource(option) && isMetadataSource(value)) {
      return option.source.id === value.source.id
    }
    if (isMetadataTag(option) && isMetadataTag(value)) {
      return option.tag.id === value.tag.id
    }
    if (isMetadataTagType(option) && isMetadataTagType(value)) {
      return option.tagType.id === value.tagType.id
    }
    return false
  }, [])

  const getOptionLabel = useCallback((option: Metadata) => {
    if (isMetadataSource(option)) {
      return option.source.url ?? JSON.stringify(option.source.externalMetadata)
    }
    if (isMetadataTag(option)) {
      return option.tag.name
    }
    if (isMetadataTagType(option)) {
      return option.tagType.name
    }
    throw new Error('unknown metadata type')
  }, [])

  const getOptionKey = useCallback((option: Metadata) => {
    if (isMetadataSource(option)) {
      return option.source.id
    }
    if (isMetadataTag(option)) {
      return option.tag.id
    }
    if (isMetadataTagType(option)) {
      return option.tagType.id
    }
    throw new Error('unknown metadata type')
  }, [])

  const groupBy = useCallback((option: Metadata) => {
    if (isMetadataSource(option)) {
      return 'ソース'
    }
    if (isMetadataTag(option)) {
      return 'タグ'
    }
    if (isMetadataTagType(option)) {
      return 'タイプ'
    }
    return ''
  }, [])

  const filterOptions = useCallback(createFilterOptions<Metadata>({
    stringify: option => {
      if (isMetadataSource(option)) {
        return option.source.url ?? ''
      }

      if (isMetadataTag(option)) {
        let str = ''
        for (let tag: Tag | null = option.tag; tag; tag = tag.parent ?? null) {
          str += ` ${tag.name} ${tag.kana} ${tag.aliases.join(' ')}`
        }

        return str
      }

      if (isMetadataTagType(option)) {
        return `${option.tagType.name} ${option.tagType.kana}`
      }

      return ''
    },
  }), [])

  const updateInputValue = useMemo(
    () => debounce(
      (value: string) => {
        startTransition(() => {
          setValue(value)
        })
      },
      100,
    ),
    [],
  )

  useEffect(() => {
    if (typeof inputValue !== 'string') {
      return
    }
    if (!inputValue) {
      setValue(inputValue)
      return
    }
    updateInputValue(inputValue)
  }, [ updateInputValue, inputValue ])

  const handleInputChange = useCallback((e: SyntheticEvent, value: string, reason: AutocompleteInputChangeReason) => {
    if (onInputChange) {
      onInputChange(e, value, reason)
    } else {
      updateInputValue(value)
    }
  }, [ onInputChange, updateInputValue ])

  const handleChange = useCallback((_e: SyntheticEvent, metadata: Metadata[]) => {
    onChangeMetadata?.(metadata)
  }, [ onChangeMetadata ])

  const handleSelect = useCallback((tag: Tag) => {
    onChangeMetadata?.([ { tag } ])
  }, [ onChangeMetadata ])

  const handleMouseDownSelect = useCallback((e: SyntheticEvent) => {
    e.stopPropagation()
  }, [])

  const openSelectDialog = useCallback((e: SyntheticEvent) => {
    setSelecting(true)
    e.stopPropagation()
  }, [])

  const closeSelectDialog = useCallback(() => {
    setSelecting(false)
  }, [])

  const tagTypes = useAllTagTypes()
  const { sources, tags } = value.length
    ? useMetadataLike(value)
    : useMetadataLikeSkip()

  const options = [ ...useMetadata(sources, tags, tagTypes, { noSources, noTags, noTagTypes }) ]

  return (
    <>
      <Autocomplete
        {...props}
        className={clsx(className, styles.autocomplete)}
        isOptionEqualToValue={isOptionEqualToValue}
        getOptionLabel={getOptionLabel}
        getOptionKey={getOptionKey}
        groupBy={groupBy}
        filterOptions={filterOptions}
        filterSelectedOptions
        multiple
        options={options}
        inputValue={inputValue}
        disabled={disabled}
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
                startAdornment: (
                  <>
                    {Icon ? <Icon className={styles.icon} fontSize="small" /> : null}
                    {params.InputProps.startAdornment}
                  </>
                ),
                endAdornment: (
                  <>
                    {loading ? (
                      <CircularProgress color="inherit" size={20} />
                    ) : !noTags ? (
                      <IconButton className={styles.selectButton} size="small" disabled={disabled} onMouseDown={handleMouseDownSelect} onClick={openSelectDialog} title="参照...">
                        <ArrowOutwardIcon fontSize="inherit" />
                      </IconButton>
                    ) : null}
                    {params.InputProps.endAdornment}
                  </>
                ),
              },
            }}
          />
        )}
      />
      {selecting ? (
        <TagSelectDialog close={closeSelectDialog} onSelect={handleSelect} />
      ) : null}
    </>
  )
}

interface MetadataSource {
  source: Source
}

interface MetadataTag {
  tag: Tag
}

interface MetadataTagType {
  tagType: TagType
}

export type Metadata = MetadataSource | MetadataTag | MetadataTagType

export interface AutocompleteMetadataBodyProps extends Omit<AutocompleteProps<Metadata, true, boolean | undefined, false>, 'onChange' | 'options' | 'renderInput'> {
  focus?: boolean
  label?: string
  placeholder?: string
  variant?: TextFieldVariants
  icon?: ComponentType<SvgIconProps>,
  onChange?: (metadata: Metadata[]) => void
  noSources?: boolean
  noTags?: boolean
  noTagTypes?: boolean
}

export default AutocompleteMetadataBody
