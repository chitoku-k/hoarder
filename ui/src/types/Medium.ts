import type { Replica, Source, Tag, TagType } from '@/types'

export interface TagTagType {
  tag: Tag
  type: TagType
}

export interface Medium {
  id: string
  replicas?: Replica[]
  sources?: Source[]
  tags?: TagTagType[]
  createdAt: string
  updatedAt: string
}
