'use client'

import type { FunctionComponent, ComponentPropsWithoutRef, SyntheticEvent } from 'react'
import { useMemo, useState } from 'react'
import { useCallback } from 'react'
import clsx from 'clsx'
import type { AutocompleteRenderValueGetItemProps } from '@mui/material/Autocomplete'
import Chip from '@mui/material/Chip'
import Stack from '@mui/material/Stack'
import LabelIcon from '@mui/icons-material/Label'
import SearchIcon from '@mui/icons-material/Search'

import type { Metadata } from '@/components/AutocompleteMetadata'
import AutocompleteMetadata, { isMetadataSource, isMetadataTag, isMetadataTagType } from '@/components/AutocompleteMetadata'
import SourceURL from '@/components/SourceURL'
import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import { useSearch } from '@/contexts'
import type { TagType } from '@/types'

import styles from './styles.module.scss'

const MetadataOption: FunctionComponent<MetadataOptionProps> = ({
  className,
  metadata,
}) => {
  switch (true) {
    case isMetadataSource(metadata): {
      return (
        <SourceURL className={className} source={metadata.source} noLaunch noLink />
      )
    }
    case isMetadataTag(metadata): {
      return (
        <TagBreadcrumbsList className={className} tag={metadata.tag} />
      )
    }
    case isMetadataTagType(metadata): {
      return (
        <Stack className={className} direction="row" alignItems="start">
          <LabelIcon className={styles.tagTypeIcon} fontSize="small" />
          <span className={styles.tagTypeText}>{metadata.tagType.name}</span>
        </Stack>
      )
    }
    default: {
      return null
    }
  }
}

const SearchBar: FunctionComponent<SearchBarProps> = ({
  className,
}) => {
  const [ inputValue, setInputValue ] = useState('')
  const [ tagType, setTagType ] = useState<TagType | null>(null)
  const { appendQuery } = useSearch()

  const renderMetadataOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: Metadata) => (
    <li key={key} {...props}>
      <MetadataOption className={styles.option} metadata={option} />
    </li>
  ), [])

  const renderMetadataValue = useCallback((metadata: readonly Metadata[], getTagProps: AutocompleteRenderValueGetItemProps<true>) => metadata.map((option, index) => {
    const { key, ...props } = getTagProps({ index })
    return (
      <Chip key={key} label={<MetadataOption className={styles.chip} metadata={option} />} {...props} />
    )
  }), [])

  const handleChange = useCallback((metadata: readonly Metadata[]) => {
    setInputValue('')

    const option = metadata[metadata.length - 1]
    if (!option) {
      setTagType(null)
      return
    }

    switch (true) {
      case isMetadataSource(option): {
        appendQuery({
          sourceID: option.source.id,
        })
        break
      }
      case isMetadataTag(option): {
        if (!tagType) {
          break
        }
        setTagType(null)
        appendQuery({
          tagTagTypeIDs: [
            {
              tagID: option.tag.id,
              typeID: tagType.id,
            },
          ],
        })
        break
      }
      case isMetadataTagType(option): {
        setTagType(option.tagType)
        break
      }
    }
  }, [ tagType, appendQuery ])

  const handleInputChange = useCallback((_e: SyntheticEvent, value: string) => {
    setInputValue(value)
  }, [])

  const placeholder = tagType
    ? 'タグを検索...'
    : 'ソースまたはタイプを検索...'

  const value = useMemo(
    () => tagType ? [ { tagType } ] : [],
    [ tagType ],
  )

  return (
    <Stack className={clsx(className, styles.container)} spacing={0.8} alignItems="center" direction="row">
      <AutocompleteMetadata
        className={styles.input}
        variant="filled"
        fullWidth
        autoHighlight
        clearOnBlur={false}
        clearOnEscape
        includeInputInList
        forcePopupIcon={false}
        openOnFocus
        placeholder={placeholder}
        renderOption={renderMetadataOption}
        renderValue={renderMetadataValue}
        value={value}
        inputValue={inputValue}
        icon={({ ...props }) => <SearchIcon fontSize="small" {...props} />}
        onChange={handleChange}
        onInputChange={handleInputChange}
        noSources={tagType !== null}
        noTags={tagType === null}
        noTagTypes={tagType !== null}
        slotProps={{
          popper: {
            className: styles.searchPopper,
            placement: 'bottom-start',
          },
        }}
      />
    </Stack>
  )
}

interface MetadataOptionProps {
  readonly className?: string
  readonly metadata: Metadata
}

export interface SearchBarProps {
  readonly className?: string
}

export default SearchBar
