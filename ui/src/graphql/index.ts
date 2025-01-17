import type { AxiosRequestConfig } from 'axios'
import axios from 'axios'
import createUploadLink from 'apollo-upload-client/createUploadLink.mjs'
import { buildAxiosFetch } from '@lifeomic/axios-fetch'
import { disableFragmentWarnings } from '@apollo/client'
import { relayStylePagination } from '@apollo/client/utilities'
import { ApolloClient, InMemoryCache } from '@apollo/experimental-nextjs-app-support'

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
  link: createUploadLink({
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
})
