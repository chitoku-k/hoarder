import type { AxiosRequestConfig } from 'axios'
import axios from 'axios'
import UploadHttpLink from 'apollo-upload-client/UploadHttpLink.mjs'
import { Kind, OperationTypeNode } from 'graphql'
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

export const makeClient = () => {
  const API_URL = typeof window === 'undefined' ? process.env.API_URL : ''
  if (typeof API_URL === 'undefined') {
    throw new Error('API_URL must be set')
  }

  return new ApolloClient({
    cache: new InMemoryCache({
      typePolicies: {
        Query: {
          fields: {
            allMedia: relayStylePagination([ 'sourceIds', 'tagIds', 'order' ]),
            allTags: relayStylePagination([ 'root' ]),
          },
        },
      },
    }),
    link: ApolloLink.split(
      ({ query }) => {
        const definition = getMainDefinition(query)
        return definition.kind === Kind.OPERATION_DEFINITION && definition.operation === OperationTypeNode.SUBSCRIPTION
      },
      new GraphQLWsLink(createClient({
        url: `${API_URL}/graphql/subscriptions`,
      })),
      new UploadHttpLink({
        uri: `${API_URL}/graphql`,
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
}
