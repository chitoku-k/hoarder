'use client'

import type { ComponentType, FunctionComponent, SyntheticEvent } from 'react'
import { useCallback, useState, useTransition } from 'react'
import clsx from 'clsx'
import type { AutocompleteProps } from '@mui/material/Autocomplete'
import Autocomplete from '@mui/material/Autocomplete'
import CircularProgress from '@mui/material/CircularProgress'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import type { TextFieldVariants } from '@mui/material/TextField'
import TextField from '@mui/material/TextField'

import { useAllExternalServices, useAllExternalServicesSkip } from '@/hooks'
import { ExternalService } from '@/types'

import styles from './styles.module.scss'

const AutocompleteExternalServiceBody: FunctionComponent<AutocompleteExternalServiceBodyProps> = ({
  className,
  focus,
  loadOnOpen,
  label,
  placeholder,
  variant,
  icon: Icon,
  onChange: onChangeExternalService,
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

  const handleChange = useCallback((_e: SyntheticEvent, type: ExternalService | null) => {
    onChangeExternalService?.(type)
  }, [ onChangeExternalService ])

  const externalServices = open || !loadOnOpen
    ? useAllExternalServices()
    : useAllExternalServicesSkip()

  return (
    <Autocomplete
      {...props}
      className={clsx(className, styles.autocomplete)}
      isOptionEqualToValue={(option, value) => option.id === value.id}
      getOptionLabel={option => option.name}
      getOptionKey={option => option.id}
      options={externalServices}
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

export interface AutocompleteExternalServiceBodyProps extends Omit<AutocompleteProps<ExternalService, false, boolean | undefined, false>, 'onChange' | 'options' | 'renderInput'> {
  focus?: boolean
  loadOnOpen?: boolean
  label?: string
  placeholder?: string
  variant?: TextFieldVariants
  icon?: ComponentType<SvgIconProps>,
  onChange?: (type: ExternalService | null) => void
}

export default AutocompleteExternalServiceBody
