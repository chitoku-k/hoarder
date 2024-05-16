'use client'

import type { ComponentPropsWithoutRef, ComponentType, FunctionComponent, KeyboardEvent, SyntheticEvent } from 'react'
import { useCallback, useMemo, useState, useTransition } from 'react'
import { useFilter } from '@react-aria/i18n'
import type { FilterOptionsState } from '@mui/material'
import type { AutocompleteProps } from '@mui/material/Autocomplete'
import Autocomplete from '@mui/material/Autocomplete'
import CircularProgress from '@mui/material/CircularProgress'
import type { SvgIconProps } from '@mui/material/SvgIcon'
import type { TextFieldVariants } from '@mui/material/TextField'
import TextField from '@mui/material/TextField'
import debounce from '@mui/material/utils/debounce'

import { ObjectKind, useObjects, useObjectsSkip } from '@/hooks'

import styles from './styles.module.scss'

const AutocompleteContainerBody: FunctionComponent<AutocompleteContainerBodyProps> = ({
  focus,
  label,
  placeholder,
  variant,
  icon: Icon,
  onChange: onChangeContainer,
  ...props
}) => {
  const [ open, setOpen ] = useState(false)
  const [ value, setValue ] = useState('')
  const [ inputValue, setInputValue ] = useState('')
  const [ highlight, setHighlight ] = useState<string | null>(null)

  const [ loading, startTransition ] = useTransition()
  const { contains } = useFilter({
    usage: 'search',
    sensitivity: 'accent',
  })

  const ref = useCallback((input: HTMLInputElement | null) => {
    if (!focus) {
      return
    }
    requestAnimationFrame(() => {
      input?.focus()
    })
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

  const handleHighlightChange = useCallback((_e: SyntheticEvent, option: string | null) => {
    setHighlight(option)
  }, [])

  const handleInputValueChange = useMemo(
    () => debounce(
      (value: string) => {
        startTransition(() => {
          setValue(value)
        })
      },
      300,
    ),
    [],
  )

  const handleInputChange = useCallback((_e: SyntheticEvent, value: string) => {
    setInputValue(value)
    handleInputValueChange(value.substring(0, value.lastIndexOf('/')))
    onChangeContainer?.(value)
  }, [ handleInputValueChange, onChangeContainer ])

  const handleOnKeyDown = useCallback((e: KeyboardEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    if (highlight && e.key === 'ArrowRight' && e.currentTarget.selectionEnd === e.currentTarget.value.length) {
      setInputValue(`${highlight}/`)
      startTransition(() => {
        setValue(highlight)
      })
    }
  }, [ highlight ])

  const handleChange = useCallback((_e: SyntheticEvent, value: string | null) => {
    onChangeContainer?.(value)
  }, [ onChangeContainer ])

  const renderOption = useCallback(({ key, ...props }: ComponentPropsWithoutRef<'li'>, option: string) => (
    <li key={key} {...props}>
      {Icon ? (
        <Icon className={styles.icon} fontSize="small" />
      ) : null}
      {option}/
    </li>
  ), [ Icon ])

  const filterOptions = useCallback((options: string[], state: FilterOptionsState<string>): string[] => {
    const value = state.inputValue.substring(state.inputValue.lastIndexOf('/') + 1)
    if (!value.length) {
      return options
    }

    return options.filter(option => contains(option.substring(option.lastIndexOf('/') + 1), value))
  }, [ contains ])

  const containers = open || value.length
    ? useObjects({ prefix: `/${value}`, kind: ObjectKind.Container })
    : useObjectsSkip()

  return (
    <Autocomplete
      {...props}
      freeSolo
      getOptionLabel={option => option}
      renderOption={renderOption}
      filterOptions={filterOptions}
      options={containers.map(({ name, url }) => url
        ? decodeURIComponent(url.substring(url.indexOf('file:///') + 'file:///'.length))
        : value ? `${value}/${name}` : name)}
      inputValue={inputValue}
      loading={loading}
      open={open}
      onOpen={handleOpen}
      onClose={handleClose}
      onHighlightChange={handleHighlightChange}
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
            onKeyDown: handleOnKeyDown,
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

export interface AutocompleteContainerBodyProps extends Omit<AutocompleteProps<string, false, boolean | undefined, true>, 'onChange' | 'options' | 'renderInput'> {
  focus?: boolean
  label?: string
  placeholder?: string
  variant?: TextFieldVariants
  icon?: ComponentType<SvgIconProps>,
  onChange?: (container: string | null) => void
}

export default AutocompleteContainerBody
