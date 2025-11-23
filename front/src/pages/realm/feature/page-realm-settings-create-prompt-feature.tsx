import { Form } from '@/components/ui/form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import PageRealmSettingsCreatePrompt from '../ui/page-realm-settings-create-prompt'
import { CreatePromptSchema, createPromptValidator, updatePromptValidator, UpdatePromptSchema } from '../validators'
import { useCreatePrompt, useGetPrompt, useUpdatePrompt } from '@/api/prompt.api'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'

export default function PageRealmSettingsCreatePromptFeature() {
  const { realm_name, prompt_id } = useParams<RouterParams & { prompt_id?: string }>()
  const navigate = useNavigate()
  const isEdit = !!prompt_id

  const { mutate: createPrompt } = useCreatePrompt()
  const { mutate: updatePrompt } = useUpdatePrompt()
  const { data: responseGetPrompt, isLoading } = useGetPrompt(
    { realm: realm_name!, promptId: prompt_id! },
    { enabled: isEdit }
  )

  const existingPrompt = responseGetPrompt

  const form = useForm<CreatePromptSchema | UpdatePromptSchema>({
    resolver: zodResolver(isEdit ? updatePromptValidator : createPromptValidator),
    mode: 'all',
    values: isEdit && existingPrompt
      ? {
          name: existingPrompt.name,
          description: existingPrompt.description,
          template: existingPrompt.template,
          version: existingPrompt.version,
          is_active: existingPrompt.is_active,
        }
      : {
          name: '',
          description: '',
          template: '',
          version: '',
        },
  })

  const onSubmit = form.handleSubmit((data) => {
    if (!realm_name) return

    if (isEdit && prompt_id) {
      updatePrompt(
        {
          path: {
            realm_name,
            prompt_id,
          },
          body: data,
        },
        {
          onSuccess: () => {
            toast.success('Prompt updated successfully')
            navigate(`/realms/${realm_name}/realm-settings/prompts`)
          },
          onError: (error) => {
            toast.error('Failed to update prompt: ' + error.message)
          },
        }
      )
    } else {
      createPrompt(
        {
          path: {
            realm_name,
          },
          body: data,
        },
        {
          onSuccess: () => {
            toast.success('Prompt created successfully')
            navigate(`/realms/${realm_name}/realm-settings/prompts`)
          },
          onError: (error) => {
            toast.error('Failed to create prompt: ' + error.message)
          },
        }
      )
    }
  })

  const handleBack = () => {
    navigate(`/realms/${realm_name}/realm-settings/prompts`)
  }

  if (isEdit && isLoading) {
    return <div>Loading...</div>
  }

  return (
    <Form {...form}>
      <PageRealmSettingsCreatePrompt onSubmit={onSubmit} handleBack={handleBack} isEdit={isEdit} />
    </Form>
  )
}
