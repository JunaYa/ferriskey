import { DataTable } from '@/components/ui/data-table'
import { columns } from '../columns/list-prompts.column'
import { useNavigate } from 'react-router'
import { Prompt } from '@/api/prompt.api'
import { Edit, Trash2 } from 'lucide-react'

export interface PageRealmSettingsPromptsProps {
  prompts: Prompt[];
  handleDeletePrompt: (promptId: string) => void;
  handleEditPrompt: (promptId: string) => void;
}

export default function PageRealmSettingsPrompts({
  prompts,
  handleDeletePrompt,
  handleEditPrompt,
}: PageRealmSettingsPromptsProps) {
  const navigate = useNavigate()
  return (
    <div>
      <DataTable
        data={prompts}
        columns={columns}
        searchPlaceholder='Find a prompt...'
        searchKeys={['name', 'description']}
        createData={{
          label: 'Create Prompt',
          onClick: () => {
            navigate('create')
          },
        }}
        rowActions={[
          {
            label: 'Edit',
            icon: <Edit className='h-4 w-4' />,
            onClick: (prompt) => handleEditPrompt(prompt.id),
          },
          {
            label: 'Delete',
            icon: <Trash2 className='h-4 w-4' />,
            variant: 'destructive',
            onClick: (prompt) => handleDeletePrompt(prompt.id),
          },
        ]}
      />
    </div>
  )
}
