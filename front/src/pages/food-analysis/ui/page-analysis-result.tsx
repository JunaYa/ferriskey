import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { AlertCircle, CheckCircle2, AlertTriangle, Loader2 } from 'lucide-react'
import { Schemas } from '@/api/api.client'

type SafetyLevel = 'safe' | 'caution' | 'unsafe'
interface PageAnalysisResultProps {
  result: Schemas.FoodAnalysisResult | null
  isLoading: boolean
}

export default function PageAnalysisResult({ result, isLoading }: PageAnalysisResultProps) {
  const getSafetyBadge = (level: SafetyLevel) => {
    switch (level.toLowerCase()) {
      case 'safe':
        return (
          <Badge className='bg-green-500 hover:bg-green-600 flex items-center gap-1'>
            <CheckCircle2 className='h-3 w-3' />
            Safe
          </Badge>
        )
      case 'caution':
        return (
          <Badge className='bg-yellow-500 hover:bg-yellow-600 flex items-center gap-1'>
            <AlertTriangle className='h-3 w-3' />
            Caution
          </Badge>
        )
      case 'unsafe':
        return (
          <Badge className='bg-red-500 hover:bg-red-600 flex items-center gap-1'>
            <AlertCircle className='h-3 w-3' />
            Unsafe
          </Badge>
        )
      default:
        return <Badge variant='outline'>{level}</Badge>
    }
  }

  if (isLoading) {
    return (
      <div className='flex flex-col items-center justify-center min-h-[400px] text-muted-foreground'>
        <Loader2 className='h-12 w-12 animate-spin mb-4' />
        <p>Loading analysis result...</p>
      </div>
    )
  }

  if (!result) {
    return (
      <div className='space-y-6'>
        <Card>
          <CardContent className='py-12'>
            <div className='text-center text-muted-foreground'>
              No analysis result found. The analysis may still be processing.
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className='space-y-6'>
      <div className='space-y-6'>
        {result.dishes.map((dish, index) => (
          <Card key={index}>
            <CardHeader>
              <div className='flex items-center justify-between'>
                <CardTitle>{dish.dish_name}</CardTitle>
                {getSafetyBadge(dish.safety_level as SafetyLevel)}
              </div>
              <CardDescription>{dish.reason}</CardDescription>
            </CardHeader>
            <CardContent className='space-y-6'>
              {dish.ingredients.length > 0 && (
                <div>
                  <h4 className='font-semibold mb-2 flex items-center gap-2'>
                    <AlertTriangle className='h-4 w-4 text-yellow-500' />
                    Risk Ingredients
                  </h4>
                  <div className='space-y-2'>
                    {dish.ingredients.map((ingredient, idx) => (
                      <div key={idx} className='bg-yellow-50 border border-yellow-200 rounded-md p-3'>
                        <p className='font-medium text-sm'>{ingredient.ingredient_name}</p>
                        <p className='text-sm text-muted-foreground mt-1'>{ingredient.risk_reason}</p>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {dish.ibd_concerns.length > 0 && (
                <div>
                  <h4 className='font-semibold mb-2'>IBD Concerns</h4>
                  <ul className='list-disc list-inside space-y-1 text-sm text-muted-foreground'>
                    {dish.ibd_concerns.map((concern, idx) => (
                      <li key={idx}>{concern}</li>
                    ))}
                  </ul>
                </div>
              )}

              {dish.ibs_concerns.length > 0 && (
                <div>
                  <h4 className='font-semibold mb-2'>IBS Concerns</h4>
                  <ul className='list-disc list-inside space-y-1 text-sm text-muted-foreground'>
                    {dish.ibs_concerns.map((concern, idx) => (
                      <li key={idx}>{concern}</li>
                    ))}
                  </ul>
                </div>
              )}

              {dish.recommendations.length > 0 && (
                <div>
                  <h4 className='font-semibold mb-2 flex items-center gap-2'>
                    <CheckCircle2 className='h-4 w-4 text-green-500' />
                    Recommendations
                  </h4>
                  <ul className='list-disc list-inside space-y-1 text-sm text-muted-foreground'>
                    <li>{dish.recommendations}</li>
                  </ul>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
