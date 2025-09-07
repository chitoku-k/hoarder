'use client'

import type { ComponentType, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useMemo, useState, useTransition } from 'react'
import clsx from 'clsx'
import { skipToken } from '@apollo/client/react'
import type { AutocompleteProps } from '@mui/material/Autocomplete'
import Autocomplete, { createFilterOptions } from '@mui/material/Autocomplete'
import CircularProgress from '@mui/material/CircularProgress'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import type { TextFieldVariants } from '@mui/material/TextField'
import TextField from '@mui/material/TextField'

import { useAllTagTypes } from '@/hooks'
import { TagType } from '@/types'

import styles from './styles.module.scss'

const AutocompleteTagTypeBody: FunctionComponent<AutocompleteTagTypeBodyProps> = ({
  className,
  focus,
  loadOnOpen,
  label,
  placeholder,
  variant,
  icon: Icon,
  onChange: onChangeTagType,
  ...props
}) => {
  const [ open, setOpen ] = useState(false)
  const [ loading, startTransition ] = useTransition()

  const ref = useCallback((input: HTMLInputElement | null) => {
    if (!focus) {
      return
    }
    input?.focus()
  }, [ focus ])

  const filterOptions = useMemo(() => createFilterOptions<TagType>({
    stringify: ({ name, kana }) => `${name} ${kana}`,
  }), [])

  const handleOpen = useCallback(() => {
    if (loading) {
      return
    }
    startTransition(() => {
      setOpen(true)
    })
  }, [ loading ])

  const handleClose = useCallback(() => {
    setOpen(false)
  }, [])

  const handleChange = useCallback((_e: SyntheticEvent, type: TagType | null) => {
    onChangeTagType?.(type)
  }, [ onChangeTagType ])

  const tagTypes = useAllTagTypes(open || !loadOnOpen ? null : skipToken)

  return (
    <Autocomplete
      {...props}
      className={clsx(className, styles.autocomplete)}
      isOptionEqualToValue={(option, value) => option.id === value.id}
      getOptionLabel={option => option.name}
      getOptionKey={option => option.id}
      filterOptions={filterOptions}
      options={tagTypes ?? []}
      loading={loading}
      open={open}
      onOpen={handleOpen}
      onClose={handleClose}
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

export interface AutocompleteTagTypeBodyProps extends Omit<AutocompleteProps<TagType, false, boolean | undefined, false>, 'onChange' | 'options' | 'renderInput'> {
  focus?: boolean
  loadOnOpen?: boolean
  label?: string
  placeholder?: string
  variant?: TextFieldVariants
  icon?: ComponentType<SvgIconProps>
  onChange?: (type: TagType | null) => void
}

export default AutocompleteTagTypeBody
