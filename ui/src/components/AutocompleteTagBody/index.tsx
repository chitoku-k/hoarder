'use client'

import type { ComponentType, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useEffect, useMemo, useState, useTransition } from 'react'
import { useCollator } from '@react-aria/i18n'
import clsx from 'clsx'
import type { AutocompleteInputChangeReason, AutocompleteProps } from '@mui/material/Autocomplete'
import Autocomplete from '@mui/material/Autocomplete'
import CircularProgress from '@mui/material/CircularProgress'
import IconButton from '@mui/material/IconButton'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import type { TextFieldVariants } from '@mui/material/TextField'
import TextField from '@mui/material/TextField'
import Tooltip from '@mui/material/Tooltip'
import ArrowOutwardIcon from '@mui/icons-material/ArrowOutward'
import { debounce } from '@mui/material/utils'

import TagSelectDialog from '@/components/TagSelectDialog'
import { useTagsLike, useTagsLikeSkip } from '@/hooks'
import type { Tag } from '@/types'

import styles from './styles.module.scss'

const AutocompleteTagBody: FunctionComponent<AutocompleteTagBodyProps> = ({
  className,
  focus,
  selector,
  label,
  placeholder,
  variant,
  onChange: onChangeTag,
  onInputChange,
  disabled,
  icon: Icon,
  inputValue,
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
    onInputChange?.(e, value, reason)
    updateInputValue(value)
  }, [ onInputChange, updateInputValue ])

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

  const allTags = value.length
    ? useTagsLike({ nameOrAliasLike: value })
        .toSorted((a, b) => collator.compare(a.kana, b.kana))
        .flatMap(tag => [ tag, ...tag.children.map(child => ({ ...child, parent: tag })) ])
    : useTagsLikeSkip()

  const tags: Tag[] = []
  const ids = new Set<string>()
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
        className={clsx(className, styles.autocomplete)}
        isOptionEqualToValue={(option, value) => option.id === value.id}
        getOptionLabel={option => option.name}
        getOptionKey={option => option.id}
        filterOptions={x => x}
        filterSelectedOptions
        options={tags}
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
                startAdornment: Icon ? (
                  <Icon className={styles.icon} fontSize="small" />
                ) : null,
                endAdornment: (
                  <>
                    {loading ? <CircularProgress color="inherit" size={20} /> : null}
                    {params.InputProps.endAdornment}
                    {selector ? (
                      <Tooltip title="参照..." placement="right">
                        <IconButton size="small" disabled={disabled} onClick={openSelectDialog}>
                          <ArrowOutwardIcon fontSize="inherit" />
                        </IconButton>
                      </Tooltip>
                    ) : null}
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

export interface AutocompleteTagBodyProps extends Omit<AutocompleteProps<Tag, false, boolean | undefined, false>, 'onChange' | 'options' | 'renderInput'> {
  focus?: boolean
  selector?: boolean
  label?: string
  placeholder?: string
  variant?: TextFieldVariants
  icon?: ComponentType<SvgIconProps>,
  onChange?: (tag: Tag | null) => void
}

export default AutocompleteTagBody
