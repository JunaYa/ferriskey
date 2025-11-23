import { ColumnDef } from '@/components/ui/data-table'
import { Prompt } from '@/api/prompt.api'
import BadgeColor from '@/components/ui/badge-color'
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'
import { Badge } from '@/components/ui/badge'
import { Check, X } from 'lucide-react'

export const columns: ColumnDef<Prompt>[] = [
  {
    id: 'name',
    header: 'Name',
    cell: (prompt) => <div className='font-medium'>{prompt.name}</div>,
  },
  {
    id: 'version',
    header: 'Version',
    cell: (prompt) => <BadgeColor color={BadgeColorScheme.BLUE}>{prompt.version}</BadgeColor>,
  },
  {
    id: 'description',
    header: 'Description',
    cell: (prompt) => <div className='max-w-md truncate text-muted-foreground'>{prompt.description}</div>,
  },
  {
    id: 'is_active',
    header: 'Active',
    cell: (prompt) => (
      <Badge variant={prompt.is_active ? 'default' : 'secondary'}>
        {prompt.is_active ? (
          <>
            <Check className='mr-1 h-3 w-3' />
            Active
          </>
        ) : (
          <>
            <X className='mr-1 h-3 w-3' />
            Inactive
          </>
        )}
      </Badge>
    ),
  },
  {
    id: 'updated_at',
    header: 'Last Updated',
    cell: (prompt) => (
      <div>
        <BadgeColor color={BadgeColorScheme.GRAY}>
          {new Date(prompt.updated_at).toLocaleString()}
        </BadgeColor>
      </div>
    ),
  },
]
