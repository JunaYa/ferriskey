import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import { useListFiles, useDeleteFile } from '../../../api/file.api'
import PageFilesOverview from '../ui/page-files-overview'
import { useState } from 'react'
import { Schemas } from '@/api/api.client.ts'
import { fetcher } from '@/api'

type StoredObject = Schemas.StoredObject

export default function PageFilesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const [offset, setOffset] = useState(0)
  const [limit] = useState(20)
  const [openUploadModal, setOpenUploadModal] = useState(false)

  const { data: responseListFiles, isLoading } = useListFiles({
    realm: realm_name ?? 'master',
    offset,
    limit,
  })

  const { mutate: deleteFile } = useDeleteFile()

  const files = responseListFiles?.items || []

  const handleDeleteSelected = (items: StoredObject[]) => {
    if (!realm_name) return

    items.forEach((file) => {
      deleteFile(
        {
          path: {
            realm_name,
            file_id: file.id,
          },
        },
        {
          onSuccess: () => {
            toast.success(`File ${file.original_name} deleted`)
          },
          onError: (error) => {
            toast.error(`Failed to delete file: ${error.message}`)
          },
        }
      )
    })
  }

  const handleDownload = async (file: StoredObject) => {
    if (!realm_name) return

    try {
      const url = `/realms/${realm_name}/files/${file.id}/download`
      const response = await fetcher('get', url, {})
      const downloadResponse = await response.json() as Schemas.PresignedUrl

      if (downloadResponse && 'url' in downloadResponse) {
        // Open download URL in new tab
        window.open(downloadResponse.url, '_blank')
        toast.success('Download started')
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to get download URL'
      toast.error(errorMessage)
    }
  }

  return (
    <PageFilesOverview
      data={files}
      isLoading={isLoading}
      realmName={realm_name ?? 'master'}
      handleDeleteSelected={handleDeleteSelected}
      handleDownload={handleDownload}
      openUploadModal={openUploadModal}
      setOffset={setOffset}
      setOpenUploadModal={setOpenUploadModal}
    />
  )
}
