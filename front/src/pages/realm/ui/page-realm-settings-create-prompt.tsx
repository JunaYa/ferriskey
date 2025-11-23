import BlockContent from '@/components/ui/block-content'
import { Button } from '@/components/ui/button'
import { Heading } from '@/components/ui/heading'
import { InputText } from '@/components/ui/input-text'
import { ArrowLeft } from 'lucide-react'
import { useFormContext } from 'react-hook-form'
import { CreatePromptSchema, UpdatePromptSchema } from '../validators'
import { FormField } from '@/components/ui/form'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { Textarea } from '@/components/ui/textarea'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'

export interface PageRealmSettingsCreatePromptProps {
  onSubmit: () => void
  handleBack: () => void
  isEdit?: boolean
}

export default function PageRealmSettingsCreatePrompt({
  onSubmit,
  handleBack,
  isEdit = false,
}: PageRealmSettingsCreatePromptProps) {
  const form = useFormContext<CreatePromptSchema | UpdatePromptSchema>()
  const formIsValid = form.formState.isValid
  return (
    <div className='flex flex-col p-4 gap-4'>
      <div className='flex items-center gap-3'>
        <Button variant='ghost' size='icon' onClick={handleBack}>
          <ArrowLeft className='h-3 w-3' />
        </Button>
        <span className='text-gray-500 text-sm font-medium'>Back to prompts</span>
      </div>

      <div className='flex flex-col mb-4'>
        <Heading size={3} className='text-gray-800'>
          {isEdit ? 'Edit Prompt' : 'Create Prompt'}
        </Heading>

        <p className='text-sm text-gray-500 mt-1'>
          {isEdit
            ? 'Update the prompt details below.'
            : 'Fill out the form below to create a new prompt.'}
        </p>
      </div>

      <div className='lg:w-2/3'>
        <BlockContent title='Prompt Details'>
          <div className='flex flex-col gap-5'>
            <FormField
              control={form.control}
              name='name'
              render={({ field }) => <InputText label='Prompt Name' {...field} />}
            />

            <FormField
              control={form.control}
              name='version'
              render={({ field }) => <InputText label='Version' {...field} />}
            />

            <FormField
              control={form.control}
              name='description'
              render={({ field }) => (
                <div className='flex flex-col gap-2'>
                  <Label htmlFor='description'>Description</Label>
                  <Textarea
                    id='description'
                    placeholder='Describe what this prompt does...'
                    className='min-h-[100px]'
                    {...field}
                  />
                </div>
              )}
            />

            <FormField
              control={form.control}
              name='template'
              render={({ field }) => (
                <div className='flex flex-col gap-2'>
                  <Label htmlFor='template'>Template</Label>
                  <Textarea
                    id='template'
                    placeholder='Enter your prompt template here. You can use variables like {{variable_name}}...'
                    className='min-h-[200px] font-mono text-sm'
                    {...field}
                  />
                  <p className='text-xs text-muted-foreground'>
                    Use double curly braces for variables
                  </p>
                </div>
              )}
            />

            {isEdit && (
              <FormField
                control={form.control}
                name='is_active'
                render={({ field }) => (
                  <div className='flex items-center space-x-2'>
                    <Switch id='is_active' checked={field.value} onCheckedChange={field.onChange} />
                    <Label htmlFor='is_active'>Active</Label>
                  </div>
                )}
              />
            )}
          </div>
        </BlockContent>
      </div>

      <FloatingActionBar
        actions={[
          {
            label: 'Cancel',
            variant: 'outline',
            onClick: handleBack,
          },
          {
            label: isEdit ? 'Update Prompt' : 'Create Prompt',
            variant: 'default',
            onClick: onSubmit,
          },
        ]} show={formIsValid} title={'Create Prompt'} description={'Create a new prompt for your realm'}      />
    </div>
  )
}
