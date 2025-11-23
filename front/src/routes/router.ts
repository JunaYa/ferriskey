export const REALM_URL = (realmName = ':realmName') => `/realms/${realmName}`
export const REALM_OVERVIEW_URL = '/overview'
export const REALM_SETTINGS_URL = (realmName = ':realmName') => `${REALM_URL(realmName)}/realm-settings`
export const PROMPTS_URL = (realmName: string) => `${REALM_URL(realmName)}/realm-settings/prompts`


export type RouterParams = {
  realm_name: string
  client_id?: string
  user_id?: string
  role_id?: string
  prompt_id?: string
}
