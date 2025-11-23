import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useGetAnalysisHistory } from '@/api/food-analysis.api'
import PageAnalysisHistory from '../ui/page-analysis-history'
import LoadingPage from '@/components/ui/loading-page'

export default function PageAnalysisHistoryFeature() {
  const { realm_name } = useParams<RouterParams>()

  const { data, isLoading } = useGetAnalysisHistory({
    realm: realm_name,
    offset: 0,
    limit: 50,
  })

  if (!realm_name) {
    return <LoadingPage />
  }

  const requests = data?.data || []

  return <PageAnalysisHistory requests={requests} isLoading={isLoading} />
}
