import { DataTable } from '@/components/ui/data-table'
import { Heading } from '@/components/ui/heading'
import { Download, Trash2 } from 'lucide-react'
import { Fragment } from 'react/jsx-runtime'
import { columns } from '../columns/list-file.column'
import UploadFileModalFeature from '../feature/upload-file-modal-feature'
import { Dispatch, SetStateAction } from 'react'
import { Schemas } from '@/api/api.client.ts'

type StoredObject = Schemas.StoredObject

export interface PageFilesOverviewProps {
  isLoading?: boolean
  data: StoredObject[]
  realmName: string
  handleDeleteSelected: (items: StoredObject[]) => void
  handleDownload: (file: StoredObject) => void
  openUploadModal: boolean
  setOpenUploadModal: Dispatch<SetStateAction<boolean>>
  setOffset: Dispatch<SetStateAction<number>>
}

export default function PageFilesOverview({
  isLoading,
  data,
  realmName,
  handleDeleteSelected,
  handleDownload,
  openUploadModal,
  setOpenUploadModal,
  setOffset,
}: PageFilesOverviewProps) {
  return (
    <Fragment>
      <div className='flex flex-col gap-4 p-8'>
        <div className='flex items-center justify-between'>
          <div>
            <Heading>Files</Heading>
            <p className='text-muted-foreground'>Manage files in {realmName}</p>
          </div>
        </div>

        <DataTable
          data={data}
          columns={columns}
          isLoading={isLoading}
          searchPlaceholder='Search files...'
          searchKeys={['original_name', 'object_key']}
          enableSelection={true}
          onDeleteSelected={handleDeleteSelected}
          setOffset={setOffset}
          createData={{
            label: 'Upload File',
            onClick: () => {
              setOpenUploadModal(true)
            },
          }}
          rowActions={[
            {
              label: 'Download',
              icon: <Download className='h-4 w-4' />,
              onClick: handleDownload,
            },
            {
              label: 'Delete',
              icon: <Trash2 className='h-4 w-4' />,
              variant: 'destructive',
              onClick: (file) => handleDeleteSelected([file]),
            },
          ]}
        />
      </div>

      <UploadFileModalFeature open={openUploadModal} setOpen={setOpenUploadModal} />
    </Fragment>
  )
}
