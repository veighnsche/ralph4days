import type { RemoteStatus } from '@/types/generated'

export function requiresRemoteConnection(isBackendMobile: boolean | undefined): boolean {
  return isBackendMobile === true
}

export function canQueryProjectLock(
  isBackendMobile: boolean | undefined,
  remoteStatus: Pick<RemoteStatus, 'connected'> | undefined
): boolean {
  if (!requiresRemoteConnection(isBackendMobile)) return true
  return remoteStatus?.connected === true
}
