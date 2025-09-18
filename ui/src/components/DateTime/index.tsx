'use client'

import type { ComponentPropsWithoutRef, FunctionComponent } from 'react'
import Typography from '@mui/material/Typography'

import { format } from 'date-fns'
import { ja } from 'date-fns/locale/ja'
import { TZDate } from '@date-fns/tz'

const DateTime: FunctionComponent<DateTimeProps> = ({
  date = new Date(),
  format: formatStr,
}) => (
  <Typography
    component={({ key, ...props }: ComponentPropsWithoutRef<'time'>) => (
      <time key={key} dateTime={date.toISOString()} {...props} />
    )}
  >
    {format(new TZDate(date, 'Asia/Tokyo'), formatStr, { locale: ja })}
  </Typography>
)

export interface DateTimeProps {
  readonly date?: Date
  readonly format: string
}

export default DateTime
