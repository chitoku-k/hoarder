import type {
  ExternalMetadata,
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

const builders: Builder[] = [
  {
    kind: 'bluesky',
    build: (_externalService, params) => {
      const { bluesky } = params as ExternalMetadataBluesky
      return `https://bsky.app/profile/${bluesky.creatorId}/post/${bluesky.id}`
    },
  },
  {
    kind: 'fantia',
    build: (_externalService, params) => {
      const { fantia } = params as ExternalMetadataFantia
      return `https://fantia.jp/posts/${fantia.id}`
    },
  },
  {
    kind: 'mastodon',
    build: (externalService, params) => {
      if (!externalService.baseUrl) {
        return null
      }
      const { mastodon } = params as ExternalMetadataMastodon
      return `${externalService.baseUrl}/@${mastodon.creatorId}/${mastodon.id}`
    },
  },
  {
    kind: 'misskey',
    build: (externalService, params) => {
      if (!externalService.baseUrl) {
        return null
      }
      const { misskey } = params as ExternalMetadataMisskey
      return `${externalService.baseUrl}/notes/${misskey.id}`
    },
  },
  {
    kind: 'nijie',
    build: (_externalService, params) => {
      const { nijie } = params as ExternalMetadataNijie
      return `https://nijie.info/view.php?id=${nijie.id}`
    },
  },
  {
    kind: 'pixiv',
    build: (_externalService, params) => {
      const { pixiv } = params as ExternalMetadataPixiv
      return `https://www.pixiv.net/artworks/${pixiv.id}`
    },
  },
  {
    kind: 'pixiv_fanbox',
    build: (_externalService, params) => {
      const { pixiv_fanbox } = params as ExternalMetadataPixivFanbox
      return `https://${pixiv_fanbox.creatorId}.fanbox.cc/posts/${pixiv_fanbox.id}`
    },
  },
  {
    kind: 'pleroma',
    build: (externalService, params) => {
      if (!externalService.baseUrl) {
        return null
      }
      const { pleroma } = params as ExternalMetadataPleroma
      return `${externalService.baseUrl}/notice/${pleroma.id}`
    },
  },
  {
    kind: 'seiga',
    build: (_externalService, params) => {
      const { seiga } = params as ExternalMetadataSeiga
      return `https://seiga.nicovideo.jp/seiga/im${seiga.id}`
    },
  },
  {
    kind: 'skeb',
    build: (_externalService, params) => {
      const { skeb } = params as ExternalMetadataSkeb
      return `https://skeb.jp/@${skeb.creatorId}/works/${skeb.id}`
    },
  },
  {
    kind: 'threads',
    build: (_externalService, params) => {
      const { threads } = params as ExternalMetadataThreads
      return `https://www.threads.net/@${threads.creatorId ?? ''}/post/${threads.id}`
    },
  },
  {
    kind: 'website',
    build: (_externalService, params) => {
      const { website } = params as ExternalMetadataWebsite
      return website.url
    },
  },
  {
    kind: 'x',
    build: (_externalService, params) => {
      const { x } = params as ExternalMetadataX
      return `https://x.com/${x.creatorId ?? 'i'}/status/${x.id}`
    },
  },
  {
    kind: 'xfolio',
    build: (_externalService, params) => {
      const { xfolio } = params as ExternalMetadataXfolio
      return `https://xfolio.jp/portfolio/@${xfolio.creatorId}/works/${xfolio.id}`
    },
  },
]

export const buildURL = (externalService: ExternalService, externalMetadata: ExternalMetadata): string | null => {
  for (const { kind, build } of builders) {
    if (externalService.kind === kind) {
      return build(externalService, externalMetadata)
    }
  }

  return null
}

interface Builder {
  kind: string
  build: (externalService: ExternalService, params: unknown) => string | null,
}
