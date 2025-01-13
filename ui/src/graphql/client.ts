import { registerApolloClient } from '@apollo/experimental-nextjs-app-support'

import { makeClient } from '@/graphql'

export const { query } = registerApolloClient(makeClient)
