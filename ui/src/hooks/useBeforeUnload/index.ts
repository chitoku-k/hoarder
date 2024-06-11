import { useEffect } from 'react'

const handleBeforeUnload = (e: BeforeUnloadEvent) => {
  e.preventDefault()
}

export function useBeforeUnload(enabled: boolean) {
  useEffect(() => {
    if (!enabled) {
      return
    }

    window.addEventListener('beforeunload', handleBeforeUnload)

    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload)
    }
  }, [ enabled ])
}
