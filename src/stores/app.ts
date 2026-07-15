import { defineStore } from 'pinia'

export type EnvironmentStatus = 'pending' | 'ready' | 'unavailable'

export const useAppStore = defineStore('app', {
  state: () => ({
    environmentStatus: 'pending' as EnvironmentStatus,
    activeProjectId: null as string | null,
    activeTaskId: null as string | null,
  }),
})
