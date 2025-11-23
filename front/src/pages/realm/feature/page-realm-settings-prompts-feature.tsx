import { useDeletePrompt, useGetPrompts } from '@/api/prompt.api'
import { RouterParams } from '@/routes/router'
import { useParams, useNavigate } from 'react-router'
import PageRealmSettingsPrompts from '../ui/page-realm-settings-prompts'
import { toast } from 'sonner'

export default function PageRealmSettingsPromptsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data: responseGetPrompts, isLoading } = useGetPrompts({ realm: realm_name })
  const { mutate: deletePrompt } = useDeletePrompt()

  const prompts = responseGetPrompts?.data ?? []

  const handleDeletePrompt = (promptId: string) => {
    if (!realm_name) return

    if (confirm('Are you sure you want to delete this prompt?')) {
      deletePrompt(
        {
          path: {
            realm_name,
            prompt_id: promptId,
          },
        },
        {
          onSuccess: () => {
            toast.success('Prompt deleted successfully')
          },
          onError: (error) => {
            toast.error('Failed to delete prompt: ' + error.message)
          },
        }
      )
    }
  }

  const handleEditPrompt = (promptId: string) => {
    navigate(`/realms/${realm_name}/realm-settings/prompts/${promptId}/edit`)
  }

  if (isLoading) {
    return <div>Loading...</div>
  }

  return (
    <PageRealmSettingsPrompts
      prompts={prompts}
      handleDeletePrompt={handleDeletePrompt}
      handleEditPrompt={handleEditPrompt}
    />
  )
}
