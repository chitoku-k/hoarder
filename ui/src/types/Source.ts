import { ExternalService } from '@/types'

export interface Source {
  readonly id: string
  readonly externalService: ExternalService
  readonly externalMetadata: unknown
  readonly url?: string | null
  readonly createdAt: string
  readonly updatedAt: string
}

export interface ExternalMetadataBluesky {
  readonly bluesky: {
    readonly id: string
    readonly creatorId: string
  }
}

export interface ExternalMetadataFantia {
  readonly fantia: {
    readonly id: string
  }
}

export interface ExternalMetadataMastodon {
  readonly mastodon: {
    readonly id: string
    readonly creatorId: string
  }
}

export interface ExternalMetadataMisskey {
  readonly misskey: {
    readonly id: string
  }
}

export interface ExternalMetadataNijie {
  readonly nijie: {
    readonly id: string
  }
}

export interface ExternalMetadataPixiv {
  readonly pixiv: {
    readonly id: string
  }
}

export interface ExternalMetadataPixivFanbox {
  readonly pixiv_fanbox: {
    readonly id: string
    readonly creatorId: string
  }
}

export interface ExternalMetadataPleroma {
  readonly pleroma: {
    readonly id: string
  }
}

export interface ExternalMetadataSeiga {
  readonly seiga: {
    readonly id: string
  }
}

export interface ExternalMetadataSkeb {
  readonly skeb: {
    readonly id: string
    readonly creatorId: string
  }
}

export interface ExternalMetadataThreads {
  readonly threads: {
    readonly id: string
    readonly creatorId?: string | null
  }
}

export interface ExternalMetadataWebsite {
  readonly website: {
    readonly url: string
  }
}

export interface ExternalMetadataX {
  readonly x: {
    readonly id: string
    readonly creatorId?: string | null
  }
}

export interface ExternalMetadataXfolio {
  readonly xfolio: {
    readonly id: string
    readonly creatorId: string
  }
}

export interface ExternalMetadataCustom {
  readonly custom: unknown
}
