'use client'

import type { ComponentType, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useMemo, useState, useTransition } from 'react'
import clsx from 'clsx'
import { skipToken } from '@apollo/client/react'
import type { FilterOptionsState } from '@mui/material'
import type { AutocompleteInputChangeReason, AutocompleteProps } from '@mui/material/Autocomplete'
import Autocomplete, { createFilterOptions } from '@mui/material/Autocomplete'
import CircularProgress from '@mui/material/CircularProgress'
import IconButton from '@mui/material/IconButton'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import type { TextFieldVariants } from '@mui/material/TextField'
import TextField from '@mui/material/TextField'
import ArrowOutwardIcon from '@mui/icons-material/ArrowOutward'
import { debounce } from '@mui/material/utils'

import TagSelectDialog from '@/components/TagSelectDialog'
import type { MetadataLike } from '@/hooks'
import { useAllTagTypes, useMetadataLike } from '@/hooks'
import type { Source, Tag, TagType } from '@/types'

import styles from './styles.module.scss'

export const isMetadataSource = (option: Metadata): option is MetadataSource => 'source' in option
export const isMetadataTag = (option: Metadata): option is MetadataTag => 'tag' in option
export const isMetadataTagType = (option: Metadata): option is MetadataTagType => 'tagType' in option

function* useMetadata(
  sources: MetadataLike['sources'] | null | undefined,
  tags: MetadataLike['tags'] | null | undefined,
  tagTypes: readonly TagType[] | null | undefined,
  options: {
    readonly noSources?: boolean
    readonly noTags?: boolean
    readonly noTagTypes?: boolean
  },
): Generator<Metadata> {
  if (sources && !options.noSources) {
    for (const source of sources.id) {
      yield { source }
    }

    for (const source of sources.url) {
      yield { source }
    }
  }

  if (tags && !options.noTags) {
    for (const tag of tags) {
      yield { tag }
    }
  }

  if (tagTypes && !options.noTagTypes) {
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

  const tagTypeFilterOptions = useMemo(() => createFilterOptions<MetadataTagType>({
    stringify: option => `${option.tagType.name} ${option.tagType.kana}`,
  }), [])

  const filterOptions = useCallback((options: readonly Metadata[], state: FilterOptionsState<Metadata>) => {
    const result: Metadata[] = []
    for (const option of options) {
      if (isMetadataTagType(option)) {
        result.push(...tagTypeFilterOptions([ option ], state))
      } else {
        result.push(option)
      }
    }
    return result
  }, [ tagTypeFilterOptions ])

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

  if (typeof inputValue === 'string' && value !== inputValue) {
    if (inputValue === '') {
      setValue(inputValue)
    } else {
      updateInputValue(inputValue)
    }
  }

  const handleInputChange = useCallback((e: SyntheticEvent, value: string, reason: AutocompleteInputChangeReason) => {
    if (onInputChange) {
      onInputChange(e, value, reason)
    } else {
      updateInputValue(value)
    }
  }, [ onInputChange, updateInputValue ])

  const handleChange = useCallback((_e: SyntheticEvent, metadata: readonly Metadata[]) => {
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
  const { sources, tags } = useMetadataLike(value.length ? { like: value } : skipToken)

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
  readonly source: Source
}

interface MetadataTag {
  readonly tag: Tag
}

interface MetadataTagType {
  readonly tagType: TagType
}

export type Metadata = MetadataSource | MetadataTag | MetadataTagType

export interface AutocompleteMetadataBodyProps extends Omit<AutocompleteProps<Metadata, true, boolean | undefined, false>, 'onChange' | 'options' | 'renderInput'> {
  readonly focus?: boolean
  readonly label?: string
  readonly placeholder?: string
  readonly variant?: TextFieldVariants
  readonly icon?: ComponentType<SvgIconProps>
  readonly onChange?: (metadata: readonly Metadata[]) => void
  readonly noSources?: boolean
  readonly noTags?: boolean
  readonly noTagTypes?: boolean
}

export default AutocompleteMetadataBody
