import { invoke } from '@tauri-apps/api/core'

export interface PythonWorkerInfo {
  protocolVersion: number
  workerVersion: string
  pid: number
}

export interface PythonWorkerDescription extends PythonWorkerInfo {
  pythonVersion: string
  methods: string[]
}

export async function startPythonWorker(): Promise<PythonWorkerInfo> {
  return invoke<PythonWorkerInfo>('start_python_worker')
}

export async function requestPythonWorker<Result>(
  method: string,
  params: Record<string, unknown> = {},
): Promise<Result> {
  return invoke<Result>('python_worker_request', { method, params })
}

export async function describePythonWorker(): Promise<PythonWorkerDescription> {
  return requestPythonWorker<PythonWorkerDescription>('describe')
}

export async function stopPythonWorker(): Promise<void> {
  return invoke('stop_python_worker')
}
