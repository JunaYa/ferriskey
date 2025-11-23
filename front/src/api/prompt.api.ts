import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'

export interface PromptsQuery extends BaseQuery {
  name?: string
  description?: string
  include_deleted?: boolean
  limit?: number
  offset?: number
}

export interface PromptQuery extends BaseQuery {
  promptId: string
}

// Get all prompts for a realm
export const useGetPrompts = ({ realm, name, description, include_deleted, limit, offset }: PromptsQuery) => {
  return useQuery({
    ...window.tanstackApi
      .get('/realms/{realm_name}/prompts', {
        path: {
          realm_name: realm!,
        },
        query: {
          name: name || undefined,
          description: description || undefined,
          include_deleted: include_deleted || undefined,
          limit: limit || 10,
          offset: offset || 0,
        },
      })
      .queryOptions,
    enabled: !!realm,
  })
}

// Get a single prompt by ID
export const useGetPrompt = ({ realm, promptId }: PromptQuery, options?: { enabled?: boolean }) => {
  return useQuery({
    ...window.tanstackApi
      .get('/realms/{realm_name}/prompts/{prompt_id}', {
        path: {
          realm_name: realm!,
          prompt_id: promptId,
        },
      })
      .queryOptions,
    enabled: options?.enabled !== undefined ? options.enabled : !!(realm && promptId),
  })
}

// Create a new prompt
export const useCreatePrompt = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi
      .mutation('post', '/realms/{realm_name}/prompts', async (response) => {
        const data = await response.json()
        return data
      })
      .mutationOptions,
    onSuccess: async (_data, variables) => {
      // Invalidate the prompts list for this realm
      const queryKey = window.tanstackApi
        .get('/realms/{realm_name}/prompts', {
          path: {
            realm_name: variables.path.realm_name,
          },
          query: {
            include_deleted: false,
            limit: 10,
            offset: 0,
          },
        })
        .queryKey

      await queryClient.invalidateQueries({
        queryKey,
      })
    },
  })
}

// Update an existing prompt
export const useUpdatePrompt = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi
      .mutation('put', '/realms/{realm_name}/prompts/{prompt_id}', async (response) => {
        const data = await response.json()
        return data
      })
      .mutationOptions,
    onSuccess: async (_data, variables) => {
      // Invalidate the prompts list
      const listQueryKey = window.tanstackApi
        .get('/realms/{realm_name}/prompts', {
          path: {
            realm_name: variables.path.realm_name,
          },
          query: {
            include_deleted: false,
            limit: 10,
            offset: 0,
          },
        })
        .queryKey

      // Invalidate the specific prompt detail
      const detailQueryKey = window.tanstackApi
        .get('/realms/{realm_name}/prompts/{prompt_id}', {
          path: {
            realm_name: variables.path.realm_name,
            prompt_id: variables.path.prompt_id,
          },
        })
        .queryKey

      await queryClient.invalidateQueries({ queryKey: listQueryKey })
      await queryClient.invalidateQueries({ queryKey: detailQueryKey })
    },
  })
}

// Delete a prompt (soft delete)
export const useDeletePrompt = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi
      .mutation('delete', '/realms/{realm_name}/prompts/{prompt_id}', async (response) => {
        return response.ok
      })
      .mutationOptions,
    onSuccess: async (_data, variables) => {
      // Invalidate the prompts list for this realm
      const queryKey = window.tanstackApi
        .get('/realms/{realm_name}/prompts', {
          path: {
            realm_name: variables.path.realm_name,
          },
          query: {
            include_deleted: false,
            limit: 10,
            offset: 0,
          },
        })
        .queryKey

      await queryClient.invalidateQueries({
        queryKey,
      })
    },
  })
}
