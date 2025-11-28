import { ColumnDef } from '@/components/ui/data-table'
import { Schemas } from '@/api/api.client.ts'
import { File, FileImage, FileText, FileVideo, FileAudio, Archive, FileCode } from 'lucide-react'
import BadgeColor from '@/components/ui/badge-color'
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'

type StoredObject = Schemas.StoredObject

// Format file size from bytes to human-readable format
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`
}

// Get file icon based on MIME type
const getFileIcon = (mimeType: string) => {
  if (mimeType.startsWith('image/')) {
    return <FileImage className='h-4 w-4 text-blue-500' />
  }
  if (mimeType.startsWith('video/')) {
    return <FileVideo className='h-4 w-4 text-purple-500' />
  }
  if (mimeType.startsWith('audio/')) {
    return <FileAudio className='h-4 w-4 text-green-500' />
  }
  if (mimeType.includes('pdf') || mimeType.includes('document') || mimeType.includes('text')) {
    return <FileText className='h-4 w-4 text-red-500' />
  }
  if (mimeType.includes('zip') || mimeType.includes('archive') || mimeType.includes('compressed')) {
    return <Archive className='h-4 w-4 text-orange-500' />
  }
  if (mimeType.includes('json') || mimeType.includes('xml') || mimeType.includes('code')) {
    return <FileCode className='h-4 w-4 text-indigo-500' />
  }
  return <File className='h-4 w-4 text-gray-500' />
}

export const columns: ColumnDef<StoredObject>[] = [
  {
    id: 'name',
    header: 'File Name',
    cell: (file) => (
      <div className='flex items-center gap-3'>
        <div className='flex-shrink-0'>{getFileIcon(file.mime_type)}</div>
        <div>
          <div className='font-medium'>{file.original_name}</div>
          <div className='text-xs text-muted-foreground'>{file.object_key}</div>
        </div>
      </div>
    ),
  },
  {
    id: 'size',
    header: 'Size',
    cell: (file) => (
      <div className='text-sm font-medium'>{formatFileSize(file.size_bytes)}</div>
    ),
  },
  {
    id: 'mime_type',
    header: 'Type',
    cell: (file) => (
      <BadgeColor color={BadgeColorScheme.BLUE}>
        {file.mime_type}
      </BadgeColor>
    ),
  },
  {
    id: 'uploaded_by',
    header: 'Uploaded By',
    cell: (file) => (
      <div className='text-sm text-muted-foreground'>{file.uploaded_by || '-'}</div>
    ),
  },
  {
    id: 'created_at',
    header: 'Created At',
    cell: (file) => (
      <div className='text-sm text-muted-foreground'>
        {new Date(file.created_at).toLocaleString()}
      </div>
    ),
  },
]
