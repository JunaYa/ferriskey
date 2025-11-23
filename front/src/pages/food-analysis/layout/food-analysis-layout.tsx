import { Heading } from '@/components/ui/heading'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { RouterParams } from '@/routes/router'
import { Outlet, useLocation, useNavigate, useParams } from 'react-router'

export default function FoodAnalysisLayout() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const location = useLocation()

  // Determine active tab based on current path
  const getActiveTab = () => {
    if (location.pathname.includes('/history')) return 'history'
    if (location.pathname.includes('/result')) return 'history'
    return 'analyze'
  }

  const handleTabChange = (value: string) => {
    navigate(`/realms/${realm_name}/food-analysis/${value}`)
  }

  return (
    <div className='p-4'>
      <div className='pb-4 mb-4'>
        <div className='flex flex-col gap-2 mb-4'>
          <div className='flex flex-col gap-2'>
            <Heading size={3}>Food Analysis</Heading>
            <p className='text-sm text-muted-foreground'>
              Analyze food items for IBD/IBS safety using AI-powered analysis
            </p>
          </div>
        </div>

        <div>
          <Tabs value={getActiveTab()} onValueChange={handleTabChange}>
            <TabsList className='flex items-center gap-4'>
              <TabsTrigger value='analyze'>Analyze Food</TabsTrigger>
              <TabsTrigger value='history'>Analysis History</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>

      <Outlet />
    </div>
  )
}
