import { Route, Routes } from 'react-router'
import PageFilesOverviewFeature from './feature/page-files-overview-feature'
import { FILE_OVERVIEW_URL } from '@/routes/sub-router/file.router'

export default function PageFile() {
  return (
    <Routes>
      <Route path={FILE_OVERVIEW_URL} element={<PageFilesOverviewFeature />} />
      <Route path='/' element={<PageFilesOverviewFeature />} />
    </Routes>
  )
}
