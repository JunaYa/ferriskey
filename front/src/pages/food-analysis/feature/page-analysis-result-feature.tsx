import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useGetAnalysisResult } from '@/api/food-analysis.api'
import PageAnalysisResult from '../ui/page-analysis-result'
import LoadingPage from '@/components/ui/loading-page'

export default function PageAnalysisResultFeature() {
  const { realm_name, requestId } = useParams<RouterParams & { requestId: string }>()

  const { data, isLoading } = useGetAnalysisResult({
    realm: realm_name,
    requestId: requestId!,
  })

  if (!realm_name || !requestId) {
    return <LoadingPage />
  }

  return <PageAnalysisResult result={data?.data || null} isLoading={isLoading} />
}
