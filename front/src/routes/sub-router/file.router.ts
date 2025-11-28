import { REALM_URL } from '../router'

export const FILES_URL = (realmName = ':realmName') => `${REALM_URL(realmName)}/files`
export const FILE_URL = (realmName = ':realmName', fileId = ':fileId') =>
  `${FILES_URL(realmName)}/${fileId}`

export const FILE_OVERVIEW_URL = '/overview'

export type FileRouterParams = {
  realm_name: string
  file_id?: string
  current_view?: 'overview'
}
