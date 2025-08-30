import type { AxiosRequestConfig } from 'axios'
import axios from 'axios'
import UploadHttpLink from 'apollo-upload-client/UploadHttpLink.mjs'
import { createClient } from 'graphql-ws'
import { buildAxiosFetch } from '@lifeomic/axios-fetch'
import { ApolloLink, disableFragmentWarnings } from '@apollo/client'
import { GraphQLWsLink } from '@apollo/client/link/subscriptions'
import { getMainDefinition, relayStylePagination } from '@apollo/client/utilities'
import { ApolloClient, InMemoryCache } from '@apollo/client-integration-nextjs'

interface ApolloRequestInit extends RequestInit {
  onUploadProgress?: AxiosRequestConfig['onUploadProgress']
}

disableFragmentWarnings()

export const makeClient = () => new ApolloClient({
  cache: new InMemoryCache({
    typePolicies: {
      Query: {
        fields: {
          allMedia: relayStylePagination(['sourceIds', 'tagIds', 'order']),
          allTags: relayStylePagination(['root']),
        },
      },
    },
  }),
  link: ApolloLink.split(
    ({ query }) => {
      const definition = getMainDefinition(query)
      return definition.kind === 'OperationDefinition' && definition.operation === 'subscription'
    },
    new GraphQLWsLink(createClient({
      url: typeof window === 'undefined' ? `${process.env.BASE_URL}/graphql/subscriptions` : '/graphql/subscriptions',
    })),
    new UploadHttpLink({
      uri: typeof window === 'undefined' ? `${process.env.BASE_URL}/graphql` : '/graphql',
      fetch: buildAxiosFetch(axios, (config, _input, init: ApolloRequestInit = {}) => ({
        ...config,
        signal: init.signal,
        onUploadProgress: init.onUploadProgress,
      })) as typeof fetch,
      fetchOptions: {
        cache: 'no-store',
      },
      headers: {
        'Apollo-Require-Preflight': 'true',
      },
    }),
  ),
})
