import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { useAnalyzeFoodText, useAnalyzeFoodImage } from '@/api/food-analysis.api'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { Loader2, ImageIcon, FileText } from 'lucide-react'
import { Label } from '@/components/ui/label'
import { useGetPrompts } from '@/api/prompt.api'

export default function PageAnalyzeFood() {
  const { realm_name } = useParams<RouterParams>()
  const [selectedPromptId, setSelectedPromptId] = useState<string>('')
  const [textInput, setTextInput] = useState('')
  const [imageFile, setImageFile] = useState<File | null>(null)

  // fetch prompts
  const { data: responseGetPrompts, isLoading } = useGetPrompts({ realm: realm_name })
  const prompts = responseGetPrompts?.data ?? []

  const analyzeFoodText = useAnalyzeFoodText()
  const analyzeFoodImage = useAnalyzeFoodImage()

  const handleTextAnalysis = async () => {
    if (!selectedPromptId || !textInput) {
      toast.error('Please select a prompt and enter text')
      return
    }

    try {
      await analyzeFoodText.mutateAsync({
        path: { realm_name: realm_name! },
        body: {
          prompt_id: selectedPromptId,
          text_input: textInput,
        },
      })
      toast.success('Food analysis completed successfully')
      setTextInput('')
    } catch {
      toast.error('Failed to analyze food')
    }
  }

  const handleImageAnalysis = async () => {
    if (!selectedPromptId || !imageFile) {
      toast.error('Please select a prompt and upload an image')
      return
    }

    try {
      const formData = new FormData()
      formData.append('prompt_id', selectedPromptId)
      formData.append('image', imageFile)

      await analyzeFoodImage.mutateAsync({
        path: { realm_name: realm_name! },
        body: formData,
      })
      toast.success('Image analysis completed successfully')
      setImageFile(null)
    } catch {
      toast.error('Failed to analyze image')
    }
  }

  return (
    <div className='space-y-6'>
      <Card>
        <CardHeader>
          <CardTitle>Select Analysis Prompt</CardTitle>
          <CardDescription>
            Choose the prompt template for food analysis
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className='space-y-2'>
            <Label htmlFor='prompt-select'>Prompt</Label>
            <Select value={selectedPromptId} onValueChange={setSelectedPromptId}>
              <SelectTrigger id='prompt-select'>
                <SelectValue placeholder='Select a prompt...' />
              </SelectTrigger>
              <SelectContent>
                {isLoading ? (
                  <SelectItem value='loading'>Loading...</SelectItem>
                ) : (
                  prompts.map((prompt) => (
                  <SelectItem key={prompt.id} value={prompt.id}>
                      {prompt.name}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      <Tabs defaultValue='text' className='w-full'>
        <TabsList className='grid w-full grid-cols-2'>
          <TabsTrigger value='text' className='flex items-center gap-2'>
            <FileText className='h-4 w-4' />
            Text Analysis
          </TabsTrigger>
          <TabsTrigger value='image' className='flex items-center gap-2'>
            <ImageIcon className='h-4 w-4' />
            Image Analysis
          </TabsTrigger>
        </TabsList>

        <TabsContent value='text'>
          <Card>
            <CardHeader>
              <CardTitle>Analyze Food from Text</CardTitle>
              <CardDescription>
                Enter a description of the food item you want to analyze
              </CardDescription>
            </CardHeader>
            <CardContent className='space-y-4'>
              <div className='space-y-2'>
                <Label htmlFor='text-input'>Food Description</Label>
                <Textarea
                  id='text-input'
                  placeholder='E.g., Spicy chicken curry with rice and vegetables'
                  value={textInput}
                  onChange={(e) => setTextInput(e.target.value)}
                  maxLength={5000}
                  className='min-h-[120px]'
                />
                <p className='text-xs text-muted-foreground'>
                  {textInput.length}/5000 characters
                </p>
              </div>
              <Button
                onClick={handleTextAnalysis}
                disabled={analyzeFoodText.isPending || !selectedPromptId || !textInput}
                className='w-full'
              >
                {analyzeFoodText.isPending && (
                  <Loader2 className='mr-2 h-4 w-4 animate-spin' />
                )}
                Analyze Text
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value='image'>
          <Card>
            <CardHeader>
              <CardTitle>Analyze Food from Image</CardTitle>
              <CardDescription>
                Upload an image of the food item you want to analyze
              </CardDescription>
            </CardHeader>
            <CardContent className='space-y-4'>
              <div className='space-y-2'>
                <Label htmlFor='image-input'>Food Image</Label>
                <Input
                  id='image-input'
                  type='file'
                  accept='image/*'
                  onChange={(e) => setImageFile(e.target.files?.[0] || null)}
                />
                {imageFile && (
                  <p className='text-sm text-muted-foreground'>
                    Selected: {imageFile.name} ({(imageFile.size / 1024 / 1024).toFixed(2)} MB)
                  </p>
                )}
              </div>
              <Button
                onClick={handleImageAnalysis}
                disabled={analyzeFoodImage.isPending || !selectedPromptId || !imageFile}
                className='w-full'
              >
                {analyzeFoodImage.isPending && (
                  <Loader2 className='mr-2 h-4 w-4 animate-spin' />
                )}
                Analyze Image
              </Button>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
