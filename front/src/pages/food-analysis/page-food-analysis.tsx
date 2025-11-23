import { Route, Routes } from 'react-router'
import PageAnalyzeFoodFeature from './feature/page-analyze-food-feature'
import PageAnalysisHistoryFeature from './feature/page-analysis-history-feature'
import PageAnalysisResultFeature from './feature/page-analysis-result-feature'
import FoodAnalysisLayout from './layout/food-analysis-layout'

export default function PageFoodAnalysis() {
  return (
    <Routes>
      <Route element={<FoodAnalysisLayout />}>
        <Route path='/analyze' element={<PageAnalyzeFoodFeature />} />
        <Route path='/history' element={<PageAnalysisHistoryFeature />} />
        <Route path='/result/:requestId' element={<PageAnalysisResultFeature />} />
        <Route path='*' element={<PageAnalyzeFoodFeature />} />
      </Route>
    </Routes>
  )
}
