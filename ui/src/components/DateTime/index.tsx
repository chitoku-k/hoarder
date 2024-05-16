'use client'

import type { ComponentPropsWithoutRef, FunctionComponent } from 'react'
import Typography from '@mui/material/Typography'

import { formatInTimeZone } from 'date-fns-tz'
import { ja } from 'date-fns/locale/ja'

const DateTime: FunctionComponent<DateTimeProps> = ({
  date = new Date(),
  format,
}) => (
  <Typography
    component={({ key, ...props }: ComponentPropsWithoutRef<'time'>) => (
      <time key={key} dateTime={date.toISOString()} {...props} />
    )}
  >
    {formatInTimeZone(date, 'Asia/Tokyo', format, { locale: ja })}
  </Typography>
)

export interface DateTimeProps {
  date?: Date
  format: string
}

export default DateTime
