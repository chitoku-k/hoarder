import { ApolloError } from '@apollo/client'
import { GraphQLError } from 'graphql'

import type { ExternalServiceNotFound, EXTERNAL_SERVICE_NOT_FOUND } from './ExternalServiceNotFound'
import type { ExternalServiceSlugDuplicate, EXTERNAL_SERVICE_SLUG_DUPLICATE } from './ExternalServiceSlugDuplicate'
import type { MediumNotFound, MEDIUM_NOT_FOUND } from './MediumNotFound'
import type { MediumReplicaDecodeFailed, MEDIUM_REPLICA_DECODE_FAILED } from './MediumReplicaDecodeFailed'
import type { MediumReplicaEncodeFailed, MEDIUM_REPLICA_ENCODE_FAILED } from './MediumReplicaEncodeFailed'
import type { MediumReplicaReadFailed, MEDIUM_REPLICA_READ_FAILED } from './MediumReplicaReadFailed'
import type { MediumReplicaUnsupported, MEDIUM_REPLICA_UNSUPPORTED } from './MediumReplicaUnsupported'
import type { MediumReplicasNotMatch, MEDIUM_REPLICAS_NOT_MATCH } from './MediumReplicasNotMatch'
import type { MediumSourceNotFound, MEDIUM_SOURCE_NOT_FOUND } from './MediumSourceNotFound'
import type { MediumTagNotFound, MEDIUM_TAG_NOT_FOUND } from './MediumTagNotFound'
import type { ObjectAlreadyExists, OBJECT_ALREADY_EXISTS } from './ObjectAlreadyExists'
import type { ObjectDeleteFailed, OBJECT_DELETE_FAILED } from './ObjectDeleteFailed'
import type { ObjectGetFailed, OBJECT_GET_FAILED } from './ObjectGetFailed'
import type { ObjectListFailed, OBJECT_LIST_FAILED } from './ObjectListFailed'
import type { ObjectNotFound, OBJECT_NOT_FOUND } from './ObjectNotFound'
import type { ObjectPathInvalid, OBJECT_PATH_INVALID } from './ObjectPathInvalid'
import type { ObjectPutFailed, OBJECT_PUT_FAILED } from './ObjectPutFailed'
import type { ObjectUrlInvalid, OBJECT_URL_INVALID } from './ObjectUrlInvalid'
import type { ObjectUrlUnsupported, OBJECT_URL_UNSUPPORTED } from './ObjectUrlUnsupported'
import type { ReplicaNotFound, REPLICA_NOT_FOUND } from './ReplicaNotFound'
import type { ReplicaNotFoundByUrl, REPLICA_NOT_FOUND_BY_URL } from './ReplicaNotFoundByUrl'
import type { ReplicaOriginalUrlDuplicate, REPLICA_ORIGINAL_URL_DUPLICATE } from './ReplicaOriginalUrlDuplicate'
import type { SourceMetadataDuplicate, SOURCE_METADATA_DUPLICATE } from './SourceMetadataDuplicate'
import type { SourceMetadataInvalid, SOURCE_METADATA_INVALID } from './SourceMetadataInvalid'
import type { SourceMetadataNotMatch, SOURCE_METADATA_NOT_MATCH } from './SourceMetadataNotMatch'
import type { SourceNotFound, SOURCE_NOT_FOUND } from './SourceNotFound'
import type { TagAttachingToDescendant, TAG_ATTACHING_TO_DESCENDANT } from './TagAttachingToDescendant'
import type { TagAttachingToItself, TAG_ATTACHING_TO_ITSELF } from './TagAttachingToItself'
import type { TagChildrenExist, TAG_CHILDREN_EXIST } from './TagChildrenExist'
import type { TagNotFound, TAG_NOT_FOUND } from './TagNotFound'
import type { TagTypeNotFound, TAG_TYPE_NOT_FOUND } from './TagTypeNotFound'
import type { TagTypeSlugDuplicate, TAG_TYPE_SLUG_DUPLICATE } from './TagTypeSlugDuplicate'
import type { ThumbnailNotFound, THUMBNAIL_NOT_FOUND } from './ThumbnailNotFound'

export * from './ExternalServiceNotFound'
export * from './ExternalServiceSlugDuplicate'
export * from './MediumNotFound'
export * from './MediumReplicaDecodeFailed'
export * from './MediumReplicaEncodeFailed'
export * from './MediumReplicaReadFailed'
export * from './MediumReplicaUnsupported'
export * from './MediumReplicasNotMatch'
export * from './MediumSourceNotFound'
export * from './MediumTagNotFound'
export * from './ObjectAlreadyExists'
export * from './ObjectDeleteFailed'
export * from './ObjectGetFailed'
export * from './ObjectListFailed'
export * from './ObjectNotFound'
export * from './ObjectPathInvalid'
export * from './ObjectPutFailed'
export * from './ObjectUrlInvalid'
export * from './ObjectUrlUnsupported'
export * from './ReplicaNotFound'
export * from './ReplicaNotFoundByUrl'
export * from './ReplicaOriginalUrlDuplicate'
export * from './SourceMetadataDuplicate'
export * from './SourceMetadataInvalid'
export * from './SourceMetadataNotMatch'
export * from './SourceNotFound'
export * from './TagAttachingToDescendant'
export * from './TagAttachingToItself'
export * from './TagChildrenExist'
export * from './TagNotFound'
export * from './TagTypeNotFound'
export * from './TagTypeSlugDuplicate'
export * from './ThumbnailNotFound'

function graphQLError(e: unknown, code: typeof EXTERNAL_SERVICE_NOT_FOUND): ExternalServiceNotFound | undefined;
function graphQLError(e: unknown, code: typeof EXTERNAL_SERVICE_SLUG_DUPLICATE): ExternalServiceSlugDuplicate | undefined;
function graphQLError(e: unknown, code: typeof MEDIUM_NOT_FOUND): MediumNotFound | undefined;
function graphQLError(e: unknown, code: typeof MEDIUM_REPLICA_DECODE_FAILED): MediumReplicaDecodeFailed | undefined;
function graphQLError(e: unknown, code: typeof MEDIUM_REPLICA_ENCODE_FAILED): MediumReplicaEncodeFailed | undefined;
function graphQLError(e: unknown, code: typeof MEDIUM_REPLICA_READ_FAILED): MediumReplicaReadFailed | undefined;
function graphQLError(e: unknown, code: typeof MEDIUM_REPLICA_UNSUPPORTED): MediumReplicaUnsupported | undefined;
function graphQLError(e: unknown, code: typeof MEDIUM_REPLICAS_NOT_MATCH): MediumReplicasNotMatch | undefined;
function graphQLError(e: unknown, code: typeof MEDIUM_SOURCE_NOT_FOUND): MediumSourceNotFound | undefined;
function graphQLError(e: unknown, code: typeof MEDIUM_TAG_NOT_FOUND): MediumTagNotFound | undefined;
function graphQLError(e: unknown, code: typeof OBJECT_ALREADY_EXISTS): ObjectAlreadyExists | undefined;
function graphQLError(e: unknown, code: typeof OBJECT_DELETE_FAILED): ObjectDeleteFailed | undefined;
function graphQLError(e: unknown, code: typeof OBJECT_GET_FAILED): ObjectGetFailed | undefined;
function graphQLError(e: unknown, code: typeof OBJECT_LIST_FAILED): ObjectListFailed | undefined;
function graphQLError(e: unknown, code: typeof OBJECT_NOT_FOUND): ObjectNotFound | undefined;
function graphQLError(e: unknown, code: typeof OBJECT_PATH_INVALID): ObjectPathInvalid | undefined;
function graphQLError(e: unknown, code: typeof OBJECT_PUT_FAILED): ObjectPutFailed | undefined;
function graphQLError(e: unknown, code: typeof OBJECT_URL_INVALID): ObjectUrlInvalid | undefined;
function graphQLError(e: unknown, code: typeof OBJECT_URL_UNSUPPORTED): ObjectUrlUnsupported | undefined;
function graphQLError(e: unknown, code: typeof REPLICA_NOT_FOUND): ReplicaNotFound | undefined;
function graphQLError(e: unknown, code: typeof REPLICA_NOT_FOUND_BY_URL): ReplicaNotFoundByUrl | undefined;
function graphQLError(e: unknown, code: typeof REPLICA_ORIGINAL_URL_DUPLICATE): ReplicaOriginalUrlDuplicate | undefined;
function graphQLError(e: unknown, code: typeof SOURCE_METADATA_DUPLICATE): SourceMetadataDuplicate | undefined;
function graphQLError(e: unknown, code: typeof SOURCE_METADATA_INVALID): SourceMetadataInvalid | undefined;
function graphQLError(e: unknown, code: typeof SOURCE_METADATA_NOT_MATCH): SourceMetadataNotMatch | undefined;
function graphQLError(e: unknown, code: typeof SOURCE_NOT_FOUND): SourceNotFound | undefined;
function graphQLError(e: unknown, code: typeof TAG_ATTACHING_TO_DESCENDANT): TagAttachingToDescendant | undefined;
function graphQLError(e: unknown, code: typeof TAG_ATTACHING_TO_ITSELF): TagAttachingToItself | undefined;
function graphQLError(e: unknown, code: typeof TAG_CHILDREN_EXIST): TagChildrenExist | undefined;
function graphQLError(e: unknown, code: typeof TAG_NOT_FOUND): TagNotFound | undefined;
function graphQLError(e: unknown, code: typeof TAG_TYPE_NOT_FOUND): TagTypeNotFound | undefined;
function graphQLError(e: unknown, code: typeof TAG_TYPE_SLUG_DUPLICATE): TagTypeSlugDuplicate | undefined;
function graphQLError(e: unknown, code: typeof THUMBNAIL_NOT_FOUND): ThumbnailNotFound | undefined;
function graphQLError(e: unknown, code: string): GraphQLError | undefined {
  if (!(e instanceof ApolloError)) {
    return undefined
  }

  return e.graphQLErrors.find(err => err.extensions?.details
    && typeof err.extensions.details === 'object'
    && 'code' in err.extensions.details
    && err.extensions.details.code === code)
}

export function useError() {
  return {
    graphQLError,
  }
}
