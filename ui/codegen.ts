import type { CodegenConfig } from '@graphql-codegen/cli'

const config = {
  schema: '../schema/hoarder.gql',
  config: {
    scalars: {
      DateTime: 'string',
      JSON: 'unknown',
      Upload: 'File',
      UUID: 'string',
    },
    skipTypename: true,
    strictScalars: true,
  },
  documents: [
    'src/graphql/**/*.gql',
  ],
  ignoreNoDocuments: true,
  generates: {
    'src/graphql/': {
      preset: 'near-operation-file',
      presetConfig: {
        baseTypesPath: 'types.generated.ts',
      },
      plugins: [
        'typescript-operations',
        'typed-document-node',
      ],
      config: {
        immutableTypes: true,
      },
    },
    'src/graphql/types.generated.ts': {
      plugins: [
        'typescript',
      ],
      config: {
        enumsAsConst: true,
      },
    },
  },
} satisfies CodegenConfig

export default config
