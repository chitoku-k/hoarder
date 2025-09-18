'use client'

import type { FunctionComponent, ReactNode } from 'react'
import { createContext, useCallback, useContext, useMemo } from 'react'

import { useRouter, useSearchParams } from 'next/navigation'

const SearchContext = createContext<SearchState>({
  appendQuery() {
    throw new Error('appendQuery is not provided')
  },
  removeQuery() {
    throw new Error('removeQuery is not provided')
  },
  clearQuery() {
    throw new Error('clearQuery is not provided')
  },
})

export const useSearch = (): SearchState => useContext(SearchContext)

export const SearchProvider: FunctionComponent<SearchProviderProps> = ({
  children,
}) => {
  const router = useRouter()
  const searchParams = useSearchParams()

  const appendQuery = useCallback((appending: SearchQuery) => {
    const newSearchParams = new URLSearchParams(searchParams)

    if (appending.sourceID) {
      newSearchParams.delete('tag')
      newSearchParams.set('source', appending.sourceID)
    }

    if (appending.tagTagTypeIDs) {
      newSearchParams.delete('source')
      newSearchParams.delete('tag')

      const tags = new Set<string>(searchParams.getAll('tag'))
      for (const { typeID, tagID } of appending.tagTagTypeIDs) {
        tags.add(`${typeID}:${tagID}`)
      }

      for (const tag of tags.values()) {
        newSearchParams.append('tag', tag)
      }
    }

    router.push(`/?${newSearchParams}`)
  }, [ router, searchParams ])

  const removeQuery = useCallback((removing: SearchQuery) => {
    const newSearchParams = new URLSearchParams(searchParams)

    if (removing.sourceID) {
      newSearchParams.delete('source', removing.sourceID)
    }

    if (removing.tagTagTypeIDs) {
      for (const { typeID, tagID } of removing.tagTagTypeIDs) {
        newSearchParams.delete('tag', `${typeID}:${tagID}`)
      }
    }

    router.push(`/?${newSearchParams}`)
  }, [ router, searchParams ])

  const clearQuery = useCallback(() => {
    const newSearchParams = new URLSearchParams(searchParams)
    newSearchParams.delete('source')
    newSearchParams.delete('tag')

    router.push(`/?${newSearchParams}`)
  }, [ router, searchParams ])

  const state = useMemo(() => ({ appendQuery, removeQuery, clearQuery }), [ appendQuery, removeQuery, clearQuery ])

  return (
    <SearchContext value={state}>
      {children}
    </SearchContext>
  )
}

export interface SearchQuery {
  sourceID?: string
  tagTagTypeIDs?: {
    tagID: string
    typeID: string
  }[]
}

export interface SearchState {
  appendQuery: (query: SearchQuery) => void
  removeQuery: (query: SearchQuery) => void
  clearQuery: () => void
}

export interface SearchProviderProps {
  children: ReactNode
}
