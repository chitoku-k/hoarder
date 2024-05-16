'use client'

import type { FunctionComponent, ReactNode } from 'react'
import type { AxiosRequestConfig } from 'axios'
import axios from 'axios'
import createUploadLink from 'apollo-upload-client/createUploadLink.mjs'
import { buildAxiosFetch } from '@lifeomic/axios-fetch'
import { relayStylePagination } from '@apollo/client/utilities'
import { ApolloNextAppProvider, NextSSRApolloClient, NextSSRInMemoryCache } from '@apollo/experimental-nextjs-app-support/ssr'

interface ApolloRequestInit extends RequestInit {
  onUploadProgress?: AxiosRequestConfig['onUploadProgress']
}

const makeClient = () => new NextSSRApolloClient({
  ssrMode: true,
  cache: new NextSSRInMemoryCache({
    typePolicies: {
      Query: {
        fields: {
          allMedia: relayStylePagination(),
          allTags: relayStylePagination(),
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

const ApolloWrapper: FunctionComponent<ApolloWrapperProps> = ({
  children,
}) => {
  return (
    <ApolloNextAppProvider makeClient={makeClient}>
      {children}
    </ApolloNextAppProvider>
  )
}

export interface ApolloWrapperProps {
  children: ReactNode
}

export default ApolloWrapper
