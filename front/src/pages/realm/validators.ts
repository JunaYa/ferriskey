import { SigningAlgorithm } from '@/api/core.interface'
import { z } from 'zod'

export const updateRealmValidator = z.object({
  name: z.string().min(1),
  default_signing_algorithm: z.nativeEnum(SigningAlgorithm),
})

export const createWebhookValidator = z.object({
  name: z.string(),
  description: z.string().optional(),
  endpoint: z.string().url().optional(),
  subscribers: z.array(z.string()),
})

export const createPromptValidator = z.object({
  name: z.string().min(1, 'Name is required'),
  description: z.string().min(1, 'Description is required'),
  template: z.string().min(1, 'Template is required'),
  version: z.string().min(1, 'Version is required'),
})

export const updatePromptValidator = z.object({
  name: z.string().min(1).optional(),
  description: z.string().min(1).optional(),
  template: z.string().min(1).optional(),
  version: z.string().min(1).optional(),
  is_active: z.boolean().optional(),
})

export type UpdateRealmSchema = z.infer<typeof updateRealmValidator>
export type CreateWebhookSchema = z.infer<typeof createWebhookValidator>
export type CreatePromptSchema = z.infer<typeof createPromptValidator>
export type UpdatePromptSchema = z.infer<typeof updatePromptValidator>
