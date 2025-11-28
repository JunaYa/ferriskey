import { useState, useCallback, DragEvent } from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Upload, X, File, Loader2 } from 'lucide-react'
import { Progress } from '@/components/ui/progress'
import { cn } from '@/lib/utils'

interface UploadFileModalProps {
  open: boolean
  setOpen: (open: boolean) => void
  selectedFile: File | null
  onFileSelect: (file: File | null) => void
  onUpload: () => void
  uploadProgress: number
  isUploading: boolean
}

export default function UploadFileModal({
  open,
  setOpen,
  selectedFile,
  onFileSelect,
  uploadProgress,
  isUploading,
  onUpload,
}: UploadFileModalProps) {
  const [isDragging, setIsDragging] = useState(false)

  const handleDragEnter = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)
  }, [])

  const handleDragOver = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()
  }, [])

  const handleDrop = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault()
      e.stopPropagation()
      setIsDragging(false)

      const files = e.dataTransfer.files
      if (files.length > 0) {
        onFileSelect(files[0])
      }
    },
    [onFileSelect]
  )

  const handleFileInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files
      if (files && files.length > 0) {
        onFileSelect(files[0])
      }
    },
    [onFileSelect]
  )

  const handleRemoveFile = useCallback(() => {
    onFileSelect(null)
  }, [onFileSelect])

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className='sm:max-w-[500px]'>
        <DialogHeader>
          <DialogTitle>Upload File</DialogTitle>
          <DialogDescription>Select a file to upload to the storage</DialogDescription>
        </DialogHeader>

        <div className='space-y-4'>
          {/* Drag and Drop Area */}
          <div
            onDragEnter={handleDragEnter}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            className={cn(
              'border-2 border-dashed rounded-lg p-8 text-center transition-colors',
              isDragging
                ? 'border-primary bg-primary/5'
                : 'border-muted-foreground/25 hover:border-muted-foreground/50',
              selectedFile && 'border-primary/50 bg-primary/5'
            )}
          >
            {selectedFile ? (
              <div className='space-y-2'>
                <File className='h-12 w-12 mx-auto text-primary' />
                <div className='space-y-1'>
                  <p className='text-sm font-medium'>{selectedFile.name}</p>
                  <p className='text-xs text-muted-foreground'>
                    {formatFileSize(selectedFile.size)} • {selectedFile.type || 'Unknown type'}
                  </p>
                </div>
                <Button
                  variant='ghost'
                  size='sm'
                  onClick={handleRemoveFile}
                  disabled={isUploading}
                  className='mt-2'
                >
                  <X className='h-4 w-4 mr-1' />
                  Remove
                </Button>
              </div>
            ) : (
              <div className='space-y-2'>
                <Upload className='h-12 w-12 mx-auto text-muted-foreground' />
                <div className='space-y-1'>
                  <p className='text-sm font-medium'>
                    Drag and drop a file here, or click to select
                  </p>
                  <p className='text-xs text-muted-foreground'>
                    Supports any file type
                  </p>
                </div>
                <Label htmlFor='file-upload' className='cursor-pointer'>
                  <Button variant='outline' size='sm' asChild>
                    <span>Select File</span>
                  </Button>
                  <Input
                    id='file-upload'
                    type='file'
                    className='hidden'
                    onChange={handleFileInputChange}
                    disabled={isUploading}
                  />
                </Label>
              </div>
            )}
          </div>

          {/* Upload Progress */}
          {isUploading && (
            <div className='space-y-2'>
              <div className='flex items-center justify-between text-sm'>
                <span className='text-muted-foreground'>Uploading...</span>
                <span className='font-medium'>{Math.round(uploadProgress)}%</span>
              </div>
              <Progress value={uploadProgress} className='h-2' />
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant='outline' onClick={() => setOpen(false)} disabled={isUploading}>
            Cancel
          </Button>
          <Button onClick={onUpload} disabled={!selectedFile || isUploading}>
            {isUploading && <Loader2 className='mr-2 h-4 w-4 animate-spin' />}
            Upload
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
