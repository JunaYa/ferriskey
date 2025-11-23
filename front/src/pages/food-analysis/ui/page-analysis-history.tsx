import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { Eye, Image, FileText, Loader2 } from 'lucide-react'

interface AnalysisRequest {
  id: string
  input_type: 'image' | 'text'
  input_content: string
  created_at: string
  prompt_id: string
}

interface PageAnalysisHistoryProps {
  requests: AnalysisRequest[]
  isLoading: boolean
}

export default function PageAnalysisHistory({ requests, isLoading }: PageAnalysisHistoryProps) {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()

  const handleViewResult = (requestId: string) => {
    navigate(`/realms/${realm_name}/food-analysis/result/${requestId}`)
  }

  const truncateText = (text: string, maxLength: number) => {
    if (text.length <= maxLength) return text
    return text.substring(0, maxLength) + '...'
  }

  return (
    <div className='space-y-6'>
      <Card>
        <CardHeader>
          <CardTitle>Recent Analyses</CardTitle>
          <CardDescription>
            A list of all your food analysis requests
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className='flex flex-col items-center justify-center py-12 text-muted-foreground'>
              <Loader2 className='h-8 w-8 animate-spin mb-2' />
              <p>Loading analysis history...</p>
            </div>
          ) : requests.length === 0 ? (
            <div className='text-center py-8 text-muted-foreground'>
              No analysis history found. Start by analyzing some food items!
            </div>
          ) : (
            <div className='overflow-x-auto'>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Type</TableHead>
                    <TableHead>Content</TableHead>
                    <TableHead>Date</TableHead>
                    <TableHead className='text-right'>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {requests.map((request) => (
                    <TableRow key={request.id}>
                      <TableCell>
                        <Badge variant='outline' className='flex items-center gap-1 w-fit'>
                          {request.input_type === 'image' ? (
                            <>
                              <Image className='h-3 w-3' />
                              Image
                            </>
                          ) : (
                            <>
                              <FileText className='h-3 w-3' />
                              Text
                            </>
                          )}
                        </Badge>
                      </TableCell>
                      <TableCell className='max-w-md'>
                        <span className='text-sm'>
                          {request.input_type === 'image'
                            ? 'Image analysis'
                            : truncateText(request.input_content, 100)}
                        </span>
                      </TableCell>
                      <TableCell>
                        <span className='text-sm text-muted-foreground'>
                          {new Date(request.created_at).toLocaleString()}
                        </span>
                      </TableCell>
                      <TableCell className='text-right'>
                        <Button
                          variant='ghost'
                          size='sm'
                          onClick={() => handleViewResult(request.id)}
                          className='flex items-center gap-2'
                        >
                          <Eye className='h-4 w-4' />
                          View Result
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
