import { ExternalService } from '@/types'

export interface Source {
  id: string
  externalService: ExternalService
  externalMetadata: unknown
  url?: string | null
  createdAt: string
  updatedAt: string
}

export interface ExternalMetadataBluesky {
  bluesky: {
    id: string
    creatorId: string
  }
}

export interface ExternalMetadataFantia {
  fantia: {
    id: string
  }
}

export interface ExternalMetadataMastodon {
  mastodon: {
    id: string
    creatorId: string
  }
}

export interface ExternalMetadataMisskey {
  misskey: {
    id: string
  }
}

export interface ExternalMetadataNijie {
  nijie: {
    id: string
  }
}

export interface ExternalMetadataPixiv {
  pixiv: {
    id: string
  }
}

export interface ExternalMetadataPixivFanbox {
  pixiv_fanbox: {
    id: string
    creatorId: string
  }
}

export interface ExternalMetadataPleroma {
  pleroma: {
    id: string
  }
}

export interface ExternalMetadataSeiga {
  seiga: {
    id: string
  }
}

export interface ExternalMetadataSkeb {
  skeb: {
    id: string
    creatorId: string
  }
}

export interface ExternalMetadataThreads {
  threads: {
    id: string
    creatorId?: string | null
  }
}

export interface ExternalMetadataWebsite {
  website: {
    url: string
  }
}

export interface ExternalMetadataX {
  x: {
    id: string
    creatorId?: string | null
  }
}

export interface ExternalMetadataXfolio {
  xfolio: {
    id: string
    creatorId: string
  }
}

export interface ExternalMetadataCustom {
  custom: unknown
}
