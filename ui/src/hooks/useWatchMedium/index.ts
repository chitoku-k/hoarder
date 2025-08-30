import type { ApolloLink } from '@apollo/client'
import { Observable } from '@apollo/client'
import { useApolloClient } from '@apollo/client/react'

import type { WatchMediumSubscription, WatchMediumSubscriptionVariables } from '@/graphql/Medium'
import { WatchMediumDocument } from '@/graphql/Medium'
import { useCallback } from 'react'

type WatchMedium = ApolloLink.Result<WatchMediumSubscription>

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
