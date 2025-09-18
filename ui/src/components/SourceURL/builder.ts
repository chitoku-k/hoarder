import type {
  ExternalMetadataBluesky,
  ExternalMetadataFantia,
  ExternalMetadataMastodon,
  ExternalMetadataMisskey,
  ExternalMetadataNijie,
  ExternalMetadataPixiv,
  ExternalMetadataPixivFanbox,
  ExternalMetadataPleroma,
  ExternalMetadataSeiga,
  ExternalMetadataSkeb,
  ExternalMetadataThreads,
  ExternalMetadataWebsite,
  ExternalMetadataX,
  ExternalMetadataXfolio,
  ExternalService,
} from '@/types'

const builders = [
  {
    kind: 'bluesky',
    build: (_externalService, { id, creatorId }) => {
      if (typeof id !== 'string' || typeof creatorId !== 'string') {
        return null
      }
      return `https://bsky.app/profile/${creatorId}/post/${id}`
    },
  } satisfies Builder<'bluesky', ExternalMetadataBluesky>,
  {
    kind: 'fantia',
    build: (_externalService, { id }) => {
      if (typeof id !== 'string') {
        return null
      }
      return `https://fantia.jp/posts/${id}`
    },
  } satisfies Builder<'fantia', ExternalMetadataFantia>,
  {
    kind: 'mastodon',
    build: (externalService, { id, creatorId }) => {
      if (!externalService.baseUrl || typeof id !== 'string' || typeof creatorId !== 'string') {
        return null
      }
      return `${externalService.baseUrl}/@${creatorId}/${id}`
    },
  } satisfies Builder<'mastodon', ExternalMetadataMastodon>,
  {
    kind: 'misskey',
    build: (externalService, { id }) => {
      if (!externalService.baseUrl || typeof id !== 'string') {
        return null
      }
      return `${externalService.baseUrl}/notes/${id}`
    },
  } satisfies Builder<'misskey', ExternalMetadataMisskey>,
  {
    kind: 'nijie',
    build: (_externalService, { id }) => {
      if (typeof id !== 'string') {
        return null
      }
      return `https://nijie.info/view.php?id=${id}`
    },
  } satisfies Builder<'nijie', ExternalMetadataNijie>,
  {
    kind: 'pixiv',
    build: (_externalService, { id }) => {
      if (typeof id !== 'string') {
        return null
      }
      return `https://www.pixiv.net/artworks/${id}`
    },
  } satisfies Builder<'pixiv', ExternalMetadataPixiv>,
  {
    kind: 'pixiv_fanbox',
    build: (_externalService, { id, creatorId }) => {
      if (typeof id !== 'string' || typeof creatorId !== 'string') {
        return null
      }
      return `https://${creatorId}.fanbox.cc/posts/${id}`
    },
  } satisfies Builder<'pixiv_fanbox', ExternalMetadataPixivFanbox>,
  {
    kind: 'pleroma',
    build: (externalService, { id }) => {
      if (!externalService.baseUrl || typeof id !== 'string') {
        return null
      }
      return `${externalService.baseUrl}/notice/${id}`
    },
  } satisfies Builder<'pleroma', ExternalMetadataPleroma>,
  {
    kind: 'seiga',
    build: (_externalService, { id }) => {
      if (typeof id !== 'string') {
        return null
      }
      return `https://seiga.nicovideo.jp/seiga/im${id}`
    },
  } satisfies Builder<'seiga', ExternalMetadataSeiga>,
  {
    kind: 'skeb',
    build: (_externalService, { id, creatorId }) => {
      if (typeof id !== 'string' || typeof creatorId !== 'string') {
        return null
      }
      return `https://skeb.jp/@${creatorId}/works/${id}`
    },
  } satisfies Builder<'skeb', ExternalMetadataSkeb>,
  {
    kind: 'threads',
    build: (_externalService, { id, creatorId }) => {
      if (typeof id !== 'string') {
        return null
      }
      return `https://www.threads.net/@${creatorId ?? ''}/post/${id}`
    },
  } satisfies Builder<'threads', ExternalMetadataThreads>,
  {
    kind: 'website',
    build: (_externalService, { url }) => {
      if (typeof url !== 'string') {
        return null
      }
      return url
    },
  } satisfies Builder<'website', ExternalMetadataWebsite>,
  {
    kind: 'x',
    build: (_externalService, { id, creatorId }) => {
      if (typeof id !== 'string') {
        return null
      }
      return `https://x.com/${creatorId ?? 'i'}/status/${id}`
    },
  } satisfies Builder<'x', ExternalMetadataX>,
  {
    kind: 'xfolio',
    build: (_externalService, { id, creatorId }) => {
      if (typeof id !== 'string' || typeof creatorId !== 'string') {
        return null
      }
      return `https://xfolio.jp/portfolio/@${creatorId}/works/${id}`
    },
  } satisfies Builder<'xfolio', ExternalMetadataXfolio>,
]

export const buildURL = (externalService: ExternalService, externalMetadata: Record<string, Record<string, unknown>>): string | null => {
  for (const { kind, build } of builders) {
    if (externalService.kind === kind) {
      return build(externalService, externalMetadata)
    }
  }

  return null
}

interface Builder<Kind extends string, Metadata extends Record<Kind, unknown>> {
  readonly kind: Kind
  readonly build: (externalService: ExternalService, params: Partial<Metadata[Kind]>) => string | null
}
