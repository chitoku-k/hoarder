import type { FunctionComponent } from 'react'
import type { Metadata } from 'next'

import { query } from '@/graphql/client'
import { SearchDocument } from '@/graphql/Search'
import Content from '@/components/Content'
import MediumList from '@/components/MediumList'
import SearchQueryList from '@/components/SearchQueryList'
import type { Source, Tag, TagType } from '@/types'

const searchParamsToArray = <T extends Partial<Record<string, string | string[]>>>(
  searchParams: Promise<T>,
): Promise<Partial<Record<string, string[]>>> => searchParams.then(
  params => Object.entries(params).reduce(
    (obj, [ k, v ]) => ({
      ...obj,
      [ k ]: Array.isArray(v) ? v : [ v ],
    }),
   {},
  ),
  () => ({}),
)

const displayURL = (url: string): string => url.replace(/^https?:\/\/(?:www\.)?/, '')

const fetchSearchQuery = async (sourceIDs: string[], tagIDs: string[]): Promise<{ sources?: Source[], tagTagTypes?: { tag: Tag, type: TagType }[] }> => {
  const tagTagTypeIDs = tagIDs
    .map(tag => tag.split(':'))
    .flatMap(([ tagTypeID, tagID ]) => tagTypeID && tagID
      ? [ { tagTypeID, tagID } ]
      : [])

  const { data, error } = await query({
    query: SearchDocument,
    variables: {
      sourceIDs,
      tagIDs: tagTagTypeIDs.map(({ tagID }) => tagID),
      tagTypeIDs: tagTagTypeIDs.map(({ tagTypeID }) => tagTypeID),
    },
  })
  if (!data) {
    throw new Error('invalid data', { cause: error })
  }
  switch (true) {
    case data.sources.length > 0: {
      return {
        sources: data.sources,
      }
    }
    case data.tags.length > 0: {
      return {
        tagTagTypes: tagTagTypeIDs.flatMap(({ tagTypeID, tagID }) => {
          const type = data.tagTypes.find(({ id }) => id === tagTypeID)
          const tag = data.tags.find(({ id }) => id === tagID)
          return type && tag
            ? [ { type, tag } ]
            : []
        }),
      }
    }
    default: {
      return {}
    }
  }
}

export const generateMetadata = async (
  { searchParams }: PageProps,
): Promise<Metadata> => {
  const { source, tag } = await searchParamsToArray(searchParams)
  try {
    const { sources, tagTagTypes } = await fetchSearchQuery(source ?? [], tag ?? [])

    switch (true) {
      case sources && sources.length > 0: {
        const query = sources.map(source => `[${source.url ? displayURL(source.url) : 'ソース'}]`).join(' ')
        return {
          title: `Hoarder: ${query}`,
        }
      }
      case tagTagTypes && tagTagTypes.length > 0: {
        const query = tagTagTypes.map(({ type, tag }) => `[${type.name}:${tag.name}]`).join(' ')
        return {
          title: `Hoarder: ${query}`,
        }
      }
      default: {
        return {}
      }
    }
  } catch (e) {
    return {}
  }
}

const Page: FunctionComponent<PageProps> = async ({
  searchParams,
}) => {
  const { source, tag } = await searchParamsToArray(searchParams)
  try {
    const { sources, tagTagTypes } = await fetchSearchQuery(source ?? [], tag ?? [])
    return (
      <Content>
        <SearchQueryList sources={sources} tagTagTypes={tagTagTypes} />
        <MediumList number={48} sources={sources} tagTagTypes={tagTagTypes} />
      </Content>
    )
  } catch {
    return (
      <Content>
        <SearchQueryList />
      </Content>
    )
  }
}

export interface SearchParams extends Partial<Record<string, string | string[]>> {
  source?: string | string[]
  tag?: string | string[]
}

export interface PageProps {
  searchParams: Promise<SearchParams>
}

export default Page
