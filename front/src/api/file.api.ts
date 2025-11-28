import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery, fetcher } from '.'
import { Schemas } from './api.client'

export interface ListFilesQuery extends BaseQuery {
  offset?: number
  limit?: number
  mime_type?: string
  uploaded_by?: string
}

export interface InitiateUploadMutationData {
  path: {
    realm_name: string
  }
  body: Schemas.InitiateUploadRequest
}

export interface CompleteUploadMutationData {
  path: {
    realm_name: string
    file_id: string
  }
}

export interface GetDownloadUrlQuery extends BaseQuery {
  file_id: string
}

export interface DeleteFileMutationData {
  path: {
    realm_name: string
    file_id: string
  }
}

// List files with pagination and filters
export const useListFiles = ({ realm, offset, limit, mime_type, uploaded_by }: ListFilesQuery) => {
  return useQuery({
    ...window.tanstackApi
      .get('/realms/{realm_name}/files', {
        path: {
          realm_name: realm!,
        },
        query: {
          offset: offset || 0,
          limit: limit || 20,
          ...(mime_type && { mime_type }),
          ...(uploaded_by && { uploaded_by }),
        },
      })
      .queryOptions,
    enabled: !!realm,
  })
}

// Initiate file upload
export const useInitiateUpload = () => {
  return useMutation({
    ...window.tanstackApi
      .mutation('post', '/realms/{realm_name}/files/uploads', async (response) => {
        const data = await response.json()
        return data as Schemas.UploadNegotiation
      })
      .mutationOptions,
  })
}

// Complete file upload
export const useCompleteUpload = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (variables: CompleteUploadMutationData) => {
      const url = `/realms/${variables.path.realm_name}/files/${variables.path.file_id}/complete`
      const response = await fetcher('post', url, {})
      const data = await response.json()
      return data as Schemas.StoredObject
    },
    onSuccess: async (_data, variables) => {
      // Invalidate file list
      const queryKey = window.tanstackApi
        .get('/realms/{realm_name}/files', {
          path: {
            realm_name: variables.path.realm_name,
          },
          query: {
            offset: 0,
            limit: 20,
          },
        })
        .queryKey

      await queryClient.invalidateQueries({
        queryKey,
      })
    },
  })
}

// Get download URL (presigned URL)
export const useGetDownloadUrl = ({ realm, file_id }: GetDownloadUrlQuery) => {
  return useQuery({
    queryKey: ['file-download', realm, file_id],
    queryFn: async () => {
      const url = `/realms/${realm}/files/${file_id}/download`
      const response = await fetcher('get', url, {})
      const data = await response.json()
      return data as Schemas.PresignedUrl
    },
    enabled: !!realm && !!file_id,
  })
}

// Delete file
export const useDeleteFile = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (variables: DeleteFileMutationData) => {
      const url = `/realms/${variables.path.realm_name}/files/${variables.path.file_id}`
      await fetcher('delete', url, {})
    },
    onSuccess: async (_data, variables) => {
      // Invalidate file list
      const queryKey = window.tanstackApi
        .get('/realms/{realm_name}/files', {
          path: {
            realm_name: variables.path.realm_name,
          },
          query: {
            offset: 0,
            limit: 20,
          },
        })
        .queryKey

      await queryClient.invalidateQueries({
        queryKey,
      })
    },
  })
}
