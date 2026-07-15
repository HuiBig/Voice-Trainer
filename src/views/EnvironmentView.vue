<script setup lang="ts">
import { CircleCheck, Refresh, WarningFilled } from '@element-plus/icons-vue'
import { storeToRefs } from 'pinia'
import { computed } from 'vue'
import { useAppStore } from '../stores/app'
import { formatBytes, formatCheckedAt } from '../utils/format'

const appStore = useAppStore()
const { environmentError, environmentReport, environmentStatus, isCheckingEnvironment } = storeToRefs(appStore)

const statusTitle = computed(() => {
  const copy = {
    pending: '尚未检测',
    checking: '正在检测',
    ready: '环境已就绪',
    incomplete: '依赖不完整',
    error: '检测失败',
  }
  return copy[environmentStatus.value]
})

async function runCheck() {
  try {
    await appStore.checkEnvironment()
  } catch {
    // The store exposes the user-facing error state.
  }
}
</script>

<template>
  <section class="environment-view">
    <header class="topbar environment-header">
      <div>
        <span class="eyebrow">设置 / 运行环境</span>
        <h1>训练环境</h1>
        <p>检查本机资源与 Voice Trainer 运行 RVC 管线所需的基础依赖。</p>
      </div>
      <el-button type="primary" :icon="Refresh" :loading="isCheckingEnvironment" @click="runCheck">
        {{ environmentReport ? '重新检测' : '开始检测' }}
      </el-button>
    </header>

    <section class="environment-summary" :class="environmentStatus">
      <div class="summary-state">
        <span class="status-dot" :class="environmentStatus" />
        <div>
          <span class="section-index">当前状态</span>
          <h2>{{ statusTitle }}</h2>
        </div>
      </div>
      <div v-if="environmentReport" class="summary-count">
        <strong>{{ environmentReport.summary.requiredFound }}/{{ environmentReport.summary.requiredTotal }}</strong>
        <span>必要依赖</span>
      </div>
      <p v-else>{{ environmentError ?? '检测不会修改系统环境，也不会自动安装任何依赖。' }}</p>
    </section>

    <template v-if="environmentReport">
      <section class="system-section">
        <div class="section-heading">
          <div>
            <span class="section-index">系统资源</span>
            <h3>{{ environmentReport.platform.hostName }}</h3>
          </div>
          <span>检测于 {{ formatCheckedAt(environmentReport.checkedAt) }}</span>
        </div>
        <div class="system-facts">
          <div>
            <span>操作系统</span>
            <strong>{{ environmentReport.platform.os }} {{ environmentReport.platform.osVersion }}</strong>
          </div>
          <div>
            <span>架构</span>
            <strong>{{ environmentReport.platform.architecture }}</strong>
          </div>
          <div>
            <span>内存</span>
            <strong>{{ formatBytes(environmentReport.resources.availableMemoryBytes) }} / {{ formatBytes(environmentReport.resources.totalMemoryBytes) }}</strong>
          </div>
          <div>
            <span>磁盘可用</span>
            <strong>{{ formatBytes(environmentReport.resources.availableDiskBytes) }}</strong>
            <small>{{ environmentReport.resources.diskMount }}</small>
          </div>
        </div>
      </section>

      <section class="dependency-section">
        <div class="section-heading">
          <div>
            <span class="section-index">运行依赖</span>
            <h3>训练管线</h3>
          </div>
          <span>必要依赖必须全部可用</span>
        </div>
        <div class="dependency-list">
          <div v-for="item in environmentReport.dependencies" :key="item.id" class="dependency-row">
            <el-icon :class="item.available ? 'available' : 'missing'">
              <CircleCheck v-if="item.available" />
              <WarningFilled v-else />
            </el-icon>
            <div class="dependency-name">
              <strong>{{ item.name }}</strong>
              <span>{{ item.required ? '必要' : '可选' }}</span>
            </div>
            <div class="dependency-detail">
              <strong>{{ item.detail }}</strong>
              <span v-if="!item.available">{{ item.hint }}</span>
            </div>
          </div>
        </div>
      </section>

      <section v-if="environmentReport.summary.warnings.length" class="warning-section">
        <span class="section-index">建议</span>
        <ul>
          <li v-for="warning in environmentReport.summary.warnings" :key="warning">{{ warning }}</li>
        </ul>
      </section>
    </template>

    <section v-else class="environment-empty">
      <div class="scan-orbit" aria-hidden="true"><span /></div>
      <h2>建立本机能力基线</h2>
      <p>检测 Python、PyTorch、FFmpeg、FFprobe、CUDA/MPS、内存与训练磁盘空间。</p>
    </section>
  </section>
</template>
