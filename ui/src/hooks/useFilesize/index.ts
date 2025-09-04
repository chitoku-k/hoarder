import { useCallback } from 'react'

const units = [ 'B', 'KB', 'MB', 'GB', 'TB' ]

export function useFilesize(k = 1024): (n: number) => string {
  return useCallback((n: number) => {
    const i = n && Math.floor(Math.log(n) / Math.log(k))
    return units[i]
      ? `${(n / k ** i).toFixed(i < 2 ? 0 : 2)} ${units[i]}`
      : `${n.toFixed(0)} B`
  }, [ k ])
}
