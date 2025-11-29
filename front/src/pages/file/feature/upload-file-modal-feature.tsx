import { Dispatch, SetStateAction, useState, useCallback } from 'react'
import { toast } from 'sonner'
import { useInitiateUpload, useCompleteUpload, useUploadFile } from '../../../api/file.api'
import { Schemas } from '../../../api/api.client'
import UploadFileModal from '../ui/upload-file-modal'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { authStore } from '@/store/auth.store'

type Props = {
  open: boolean
  setOpen: Dispatch<SetStateAction<boolean>>
}

// Calculate SHA-256 checksum of a file
const calculateSHA256 = async (file: File): Promise<string> => {
  const arrayBuffer = await file.arrayBuffer()
  const hashBuffer = await crypto.subtle.digest('SHA-256', arrayBuffer)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('')
  return hashHex
}

export default function UploadFileModalFeature({ open, setOpen }: Props) {
  const { realm_name } = useParams<RouterParams>()
  const { mutate: uploadFile, isPending: isUploadingDirect } = useUploadFile()
  const { mutate: initiateUpload, isPending: isInitiating } = useInitiateUpload()
  const { mutate: completeUpload, isPending: isCompleting } = useCompleteUpload()
  const [selectedFile, setSelectedFile] = useState<File | null>(null)
  const [uploadProgress, setUploadProgress] = useState(0)
  const [isUploading, setIsUploading] = useState(false)
  const useDirectUpload = true // Default to direct upload (simpler approach)

  const handleFileSelect = useCallback((file: File | null) => {
    setSelectedFile(file)
    setUploadProgress(0)
  }, [])

  const handleUpload = useCallback(async () => {
    if (!selectedFile || !realm_name) {
      toast.error('Please select a file')
      return
    }

    setIsUploading(true)
    setUploadProgress(0)

    // Use direct upload (simpler, recommended for smaller files)
    if (useDirectUpload) {
      const formData = new FormData()
      formData.append('file', selectedFile)

      uploadFile(
        {
          path: {
            realm_name,
          },
          body: formData,
          onUploadProgress: (progressEvent) => {
            if (progressEvent.total) {
              const percentComplete = (progressEvent.loaded / progressEvent.total) * 100
              setUploadProgress(percentComplete)
            }
          },
        },
        {
          onSuccess: () => {
            toast.success('File uploaded successfully')
            setSelectedFile(null)
            setUploadProgress(0)
            setOpen(false)
            setIsUploading(false)
          },
          onError: (error) => {
            toast.error(`Failed to upload file: ${error.message}`)
            setIsUploading(false)
          },
        }
      )
      return
    }

    // Use multi-step upload (for large files with progress tracking)
    try {
      // Step 1: Calculate checksum
      toast.info('Calculating file checksum...')
      const checksum = await calculateSHA256(selectedFile)

      // Step 2: Initiate upload
      initiateUpload(
        {
          path: {
            realm_name,
          },
          body: {
            filename: selectedFile.name,
            size_bytes: selectedFile.size,
            mime_type: selectedFile.type || 'application/octet-stream',
            checksum_sha256: checksum,
            use_presigned: true,
          },
        },
        {
          onSuccess: async (negotiation: Schemas.UploadNegotiation) => {
            try {
              // Step 3: Upload file
              let uploadUrl: string
              let uploadMethod: 'PUT' | 'POST' = 'PUT'

              if (negotiation.type === 'presigned') {
                uploadUrl = negotiation.presigned_url.url
                uploadMethod = 'PUT'
              } else {
                uploadUrl = negotiation.upload_url
                uploadMethod = 'PUT'
              }

              toast.info('Uploading file...')

              // Upload with progress tracking
              const xhr = new XMLHttpRequest()

              xhr.upload.addEventListener('progress', (e) => {
                if (e.lengthComputable) {
                  const percentComplete = (e.loaded / e.total) * 100
                  setUploadProgress(percentComplete)
                }
              })

              await new Promise<void>((resolve, reject) => {
                xhr.addEventListener('load', () => {
                  if (xhr.status >= 200 && xhr.status < 300) {
                    resolve()
                  } else {
                    reject(new Error(`Upload failed with status ${xhr.status}`))
                  }
                })

                xhr.addEventListener('error', () => {
                  reject(new Error('Upload failed'))
                })

                xhr.open(uploadMethod, uploadUrl)
                xhr.setRequestHeader('Content-Type', selectedFile.type || 'application/octet-stream')

                if (negotiation.type === 'presigned') {
                  // For presigned URLs, send the file directly
                  xhr.send(selectedFile)
                } else {
                  // For direct uploads, might need authentication headers
                  const accessToken = authStore.getState().accessToken
                  if (accessToken) {
                    xhr.setRequestHeader('Authorization', `Bearer ${accessToken}`)
                  }
                  xhr.send(selectedFile)
                }
              })

              // Step 4: Complete upload
              completeUpload(
                {
                  path: {
                    realm_name,
                    file_id: negotiation.object_id,
                  },
                },
                {
                  onSuccess: () => {
                    toast.success('File uploaded successfully')
                    setSelectedFile(null)
                    setUploadProgress(0)
                    setOpen(false)
                    setIsUploading(false)
                  },
                  onError: (error) => {
                    toast.error(`Failed to complete upload: ${error.message}`)
                    setIsUploading(false)
                  },
                }
              )
            } catch (error) {
              const errorMessage = error instanceof Error ? error.message : 'Upload failed'
              toast.error(`Upload error: ${errorMessage}`)
              setIsUploading(false)
            }
          },
          onError: (error) => {
            toast.error(`Failed to initiate upload: ${error.message}`)
            setIsUploading(false)
          },
        }
      )
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to process file'
      toast.error(errorMessage)
      setIsUploading(false)
    }
  }, [selectedFile, realm_name, useDirectUpload, uploadFile, initiateUpload, completeUpload, setOpen])

  const handleClose = useCallback(() => {
    if (!isUploading && !isInitiating && !isCompleting && !isUploadingDirect) {
      setSelectedFile(null)
      setUploadProgress(0)
      setOpen(false)
    }
  }, [isUploading, isInitiating, isCompleting, isUploadingDirect, setOpen])

  return (
    <UploadFileModal
      open={open}
      setOpen={handleClose}
      selectedFile={selectedFile}
      onFileSelect={handleFileSelect}
      onUpload={handleUpload}
      uploadProgress={uploadProgress}
      isUploading={isUploading || isInitiating || isCompleting || isUploadingDirect}
    />
  )
}
