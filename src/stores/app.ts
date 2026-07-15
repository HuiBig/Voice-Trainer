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

export interface TrainingProject {
  id: number
  name: string
  materialDirectory: string
  trainingStatus: 'not_started' | 'preparing' | 'training' | 'completed' | 'failed'
  createdAt: number
  updatedAt: number
  lastOpenedAt: number | null
}

export interface MaterialScanReport {
  scannedAt: number
  directory: string
  audioFileCount: number
  totalAudioBytes: number
  unsupportedFileCount: number
  issues: string[]
}

function errorMessage(error: unknown, fallback = '操作失败，请查看应用日志') {
  if (error instanceof Error) return error.message
  return typeof error === 'string' ? error : fallback
}

export const useAppStore = defineStore('app', {
  state: () => ({
    environmentReport: null as EnvironmentReport | null,
    environmentError: null as string | null,
    isCheckingEnvironment: false,
    activeProjectId: null as string | null,
    activeTaskId: null as string | null,
    projects: [] as TrainingProject[],
    projectsLoaded: false,
    isLoadingProjects: false,
    projectError: null as string | null,
    materialScanReport: null as MaterialScanReport | null,
    isScanningMaterials: false,
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
        this.environmentError = errorMessage(error, '环境检测失败，请查看应用日志')
        throw error
      } finally {
        this.isCheckingEnvironment = false
      }
    },
    async loadProjects() {
      this.isLoadingProjects = true
      this.projectError = null
      try {
        this.projects = await invoke<TrainingProject[]>('list_projects')
        this.projectsLoaded = true
        return this.projects
      } catch (error) {
        this.projectError = errorMessage(error, '无法读取项目列表')
        throw error
      } finally {
        this.isLoadingProjects = false
      }
    },
    async createProject(input: { name: string; materialDirectory: string }) {
      const project = await invoke<TrainingProject>('create_project', { input })
      this.projects = [project, ...this.projects]
      this.activeProjectId = String(project.id)
      return project
    },
    async openProject(id: number) {
      const project = await invoke<TrainingProject>('open_project', { id })
      this.activeProjectId = String(project.id)
      this.projects = [project, ...this.projects.filter((item) => item.id !== id)]
      return project
    },
    async renameProject(id: number, name: string) {
      const project = await invoke<TrainingProject>('rename_project', { id, name })
      this.projects = this.projects.map((item) => item.id === id ? project : item)
      return project
    },
    async deleteProject(id: number) {
      await invoke('delete_project', { id })
      this.projects = this.projects.filter((item) => item.id !== id)
      if (this.activeProjectId === String(id)) this.activeProjectId = null
    },
    async scanMaterialDirectory(directory: string) {
      this.isScanningMaterials = true
      try {
        this.materialScanReport = await invoke<MaterialScanReport>('scan_material_directory', { directory })
        return this.materialScanReport
      } finally {
        this.isScanningMaterials = false
      }
    },
  },
})
