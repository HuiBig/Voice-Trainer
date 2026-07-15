import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'

export type EnvironmentStatus = 'pending' | 'checking' | 'ready' | 'incomplete' | 'error'

export interface PlatformInfo {
  os: string
  osVersion: string
  architecture: string
  hostName: string
}

export interface ResourceInfo {
  totalMemoryBytes: number
  availableMemoryBytes: number
  availableDiskBytes: number
  diskMount: string
}

export interface DependencyCheck {
  id: string
  name: string
  required: boolean
  available: boolean
  detail: string
  hint: string
}

export interface EnvironmentReport {
  checkedAt: number
  platform: PlatformInfo
  resources: ResourceInfo
  dependencies: DependencyCheck[]
  summary: {
    ready: boolean
    requiredFound: number
    requiredTotal: number
    warnings: string[]
  }
}

function errorMessage(error: unknown) {
  if (error instanceof Error) return error.message
  return typeof error === 'string' ? error : '环境检测失败，请查看应用日志'
}

export const useAppStore = defineStore('app', {
  state: () => ({
    environmentReport: null as EnvironmentReport | null,
    environmentError: null as string | null,
    isCheckingEnvironment: false,
    activeProjectId: null as string | null,
    activeTaskId: null as string | null,
  }),
  getters: {
    environmentStatus(state): EnvironmentStatus {
      if (state.isCheckingEnvironment) return 'checking'
      if (state.environmentError) return 'error'
      if (!state.environmentReport) return 'pending'
      return state.environmentReport.summary.ready ? 'ready' : 'incomplete'
    },
  },
  actions: {
    async checkEnvironment() {
      this.isCheckingEnvironment = true
      this.environmentError = null
      try {
        this.environmentReport = await invoke<EnvironmentReport>('check_environment')
        return this.environmentReport
      } catch (error) {
        this.environmentError = errorMessage(error)
        throw error
      } finally {
        this.isCheckingEnvironment = false
      }
    },
  },
})
