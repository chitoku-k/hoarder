import { FetchResult, Observable, useApolloClient } from '@apollo/client'

import type { WatchMediumSubscription, WatchMediumSubscriptionVariables } from '@/graphql/Medium'
import { WatchMediumDocument } from '@/graphql/Medium'
import { useCallback } from 'react'

type WatchMedium = FetchResult<WatchMediumSubscription>

export function useWatchMedium(): [
  (variables: WatchMediumSubscriptionVariables) => Observable<WatchMedium>,
] {
  const apolloClient = useApolloClient()
  return [
    useCallback((variables: WatchMediumSubscriptionVariables) => apolloClient.subscribe({
      query: WatchMediumDocument,
      variables,
    }), [ apolloClient ]),
  ]
}
