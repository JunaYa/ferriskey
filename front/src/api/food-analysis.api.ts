import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'
import { Schemas } from './api.client'

export interface AnalyzeFoodTextMutationData {
  path: {
    realm_name: string
  }
  body: {
    prompt_id: string
    text_input: string
  }
}

export interface AnalyzeFoodImageMutationData {
  path: {
    realm_name: string
  }
  body: FormData
}

export interface GetAnalysisHistoryQuery extends BaseQuery {
  offset?: number
  limit?: number
}

export interface GetAnalysisResultQuery extends BaseQuery {
  requestId: string
}

// Analyze food from text
export const useAnalyzeFoodText = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi
      .mutation('post', '/realms/{realm_name}/food-analysis/text', async (response) => {
        const data = await response.json()
        return data as Schemas.AnalyzeFoodResponse
      })
      .mutationOptions,
    onSuccess: async (_data, variables) => {
      // Invalidate analysis history
      const queryKey = window.tanstackApi
        .get('/realms/{realm_name}/food-analysis', {
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

// Analyze food from image
export const useAnalyzeFoodImage = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi
      .mutation('post', '/realms/{realm_name}/food-analysis/image', async (response) => {
        const data = await response.json()
        return data as Schemas.AnalyzeFoodResponse
      })
      .mutationOptions,
    onSuccess: async (_data, variables) => {
      const queryKey = window.tanstackApi
        .get('/realms/{realm_name}/food-analysis', {
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

// Get analysis history
export const useGetAnalysisHistory = ({ realm, offset, limit }: GetAnalysisHistoryQuery) => {
  return useQuery({
    ...window.tanstackApi
      .get('/realms/{realm_name}/food-analysis', {
        path: {
          realm_name: realm!,
        },
        query: {
          offset: offset || 0,
          limit: limit || 20,
        },
      })
      .queryOptions,
    enabled: !!realm,
  })
}

// Get analysis result
export const useGetAnalysisResult = ({ realm, requestId }: GetAnalysisResultQuery) => {
  return useQuery({
    ...window.tanstackApi
      .get('/realms/{realm_name}/food-analysis/{request_id}/result', {
        path: {
          realm_name: realm!,
          request_id: requestId,
        },
      })
      .queryOptions,
    enabled: !!(realm && requestId),
  })
}
