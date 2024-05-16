'use client'

import type { ComponentPropsWithoutRef, ComponentType, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useMemo, useState, useTransition } from 'react'
import { useCollator } from '@react-aria/i18n'
import type { AutocompleteProps } from '@mui/material/Autocomplete'
import Autocomplete from '@mui/material/Autocomplete'
import CircularProgress from '@mui/material/CircularProgress'
import IconButton from '@mui/material/IconButton'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import type { TextFieldVariants } from '@mui/material/TextField'
import TextField from '@mui/material/TextField'
import Tooltip from '@mui/material/Tooltip'
import ArrowOutwardIcon from '@mui/icons-material/ArrowOutward'
import debounce from '@mui/material/utils/debounce'

import TagBreadcrumbsList from '@/components/TagBreadcrumbsList'
import TagSelectDialog from '@/components/TagSelectDialog'
import { Tag } from '@/types'
import { useTagsLike, useTagsLikeSkip } from '@/hooks'

import styles from './styles.module.scss'

const AutocompleteTagBody: FunctionComponent<AutocompleteTagBodyProps> = ({
  focus,
  label,
  placeholder,
  variant,
  onChange: onChangeTag,
  disabled,
  icon: Icon,
  ...props
}) => {
  const [ value, setValue ] = useState('')
  const [ selecting, setSelecting ] = useState(false)

  const [ loading, startTransition ] = useTransition()
  const collator = useCollator()

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

  const handleChange = useCallback((_e: SyntheticEvent, tag: Tag | null) => {
    onChangeTag?.(tag)
  }, [ onChangeTag ])

  const handleSelect = useCallback((tag: Tag) => {
    onChangeTag?.(tag)
  }, [ onChangeTag ])

  const openSelectDialog = useCallback(() => {
    setSelecting(true)
  }, [])

  const closeSelectDialog = useCallback(() => {
    setSelecting(false)
  }, [])

  const renderOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: Tag) => (
    <li key={key} {...props}>
      <TagBreadcrumbsList tag={option} />
    </li>
  ), [])

  const allTags = value.length
    ? useTagsLike({ nameOrAliasLike: value })
        .toSorted((a, b) => collator.compare(a.kana, b.kana))
        .flatMap(tag => [ tag, ...tag.children.map(child => ({ ...child, parent: tag })) ])
    : useTagsLikeSkip()

  const tags = []
  const ids = new Set()
  for (const tag of allTags) {
    if (!ids.has(tag.id)) {
      ids.add(tag.id)
      tags.push(tag)
    }
  }

  return (
    <>
      <Autocomplete
        {...props}
        isOptionEqualToValue={(option, value) => option.id === value.id}
        getOptionLabel={option => option.name}
        getOptionKey={option => option.id}
        renderOption={renderOption}
        filterOptions={x => x}
        filterSelectedOptions
        options={tags}
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
            InputProps={{
              ...params.InputProps,
              startAdornment: Icon ? (
                <Icon className={styles.icon} fontSize="small" />
              ) : null,
              endAdornment: (
                <>
                  {loading ? <CircularProgress color="inherit" size={20} /> : null}
                  {params.InputProps.endAdornment}
                  <Tooltip title="参照..." placement="right">
                    <IconButton size="small" disabled={disabled} onClick={openSelectDialog}>
                      <ArrowOutwardIcon fontSize="inherit" />
                    </IconButton>
                  </Tooltip>
                </>
              ),
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

export interface AutocompleteTagBodyProps extends Omit<AutocompleteProps<Tag, false, boolean | undefined, false>, 'onChange' | 'options' | 'renderInput'> {
  focus?: boolean
  label?: string
  placeholder?: string
  variant?: TextFieldVariants
  icon?: ComponentType<SvgIconProps>,
  onChange?: (tag: Tag | null) => void
}

export default AutocompleteTagBody
