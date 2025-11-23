import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import PageAnalyzeFood from '../ui/page-analyze-food'
import LoadingPage from '@/components/ui/loading-page'

export default function PageAnalyzeFoodFeature() {
  const { realm_name } = useParams<RouterParams>()


  if (!realm_name) {
    return <LoadingPage />
  }

  return <PageAnalyzeFood />
}
