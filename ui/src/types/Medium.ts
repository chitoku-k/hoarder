import type { Replica, Source, Tag, TagType } from '@/types'

export interface TagTagType {
  readonly tag: Tag
  readonly type: TagType
}

export interface Medium {
  readonly id: string
  readonly replicas?: readonly Replica[]
  readonly sources?: readonly Source[]
  readonly tags?: readonly TagTagType[]
  readonly createdAt: string
  readonly updatedAt: string
}
