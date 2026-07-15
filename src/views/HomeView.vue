<script setup lang="ts">
import { ArrowRight, CircleCheck, Plus, Refresh, Warning } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { storeToRefs } from 'pinia'
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../stores/app'
import { formatBytes } from '../utils/format'

const appStore = useAppStore()
const router = useRouter()
const { environmentReport, environmentStatus, isCheckingEnvironment } = storeToRefs(appStore)

const environmentCopy = computed(() => {
  if (environmentStatus.value === 'ready') {
    return { title: '训练环境已就绪', detail: '必要依赖均可用，可以创建训练项目' }
  }
  if (environmentStatus.value === 'incomplete') {
    const summary = environmentReport.value?.summary
    return {
      title: '训练环境需要完善',
      detail: `已检测到 ${summary?.requiredFound ?? 0} / ${summary?.requiredTotal ?? 0} 项必要依赖`,
    }
  }
  if (environmentStatus.value === 'error') {
    return { title: '环境检测失败', detail: '请前往设置页查看错误并重试' }
  }
  if (environmentStatus.value === 'checking') {
    return { title: '正在检测训练环境', detail: '正在检查系统资源和本机运行时' }
  }
  return { title: '环境尚未检测', detail: '开始训练前需要完成一次本机运行环境检查' }
})

const setupSteps = computed(() => [
  {
    title: '检测训练环境',
    detail: '确认 FFmpeg、Python、PyTorch 与可用计算设备',
    state: environmentStatus.value === 'ready' ? 'complete' : 'current',
  },
  { title: '创建第一个项目', detail: '导入已获授权的目标人声音频', state: 'pending' },
  { title: '开始数据预处理', detail: '转码、切片并检查训练素材质量', state: 'pending' },
])

async function runEnvironmentCheck() {
  try {
    const report = await appStore.checkEnvironment()
    if (report.summary.ready) {
      ElMessage.success('训练环境检测通过')
    } else {
      ElMessage.warning('部分必要依赖尚未安装')
      await router.push('/settings')
    }
  } catch {
    ElMessage.error('环境检测失败，请在桌面应用中重试')
  }
}

function handleSetupStep(index: number) {
  if (index === 0) return runEnvironmentCheck()
  ElMessage.info(index === 1 ? '项目创建将在下一阶段接入' : '请先创建训练项目')
}
</script>

<template>
  <section class="home-view">
    <header class="topbar">
      <div>
        <span class="eyebrow">PC 训练端</span>
        <h1>工作台</h1>
      </div>
      <el-button type="primary" :icon="Plus" @click="handleSetupStep(1)">新建训练项目</el-button>
    </header>

    <div class="intro-layout">
      <section class="intro-copy">
        <span class="section-index">01 / 准备</span>
        <h2>从干净的人声素材开始。</h2>
        <p>Voice Trainer 会在本机完成数据处理、RVC 模型训练和移动端模型包导出，音频默认不会离开设备。</p>
        <button class="text-action" type="button" :disabled="isCheckingEnvironment" @click="runEnvironmentCheck">
          {{ isCheckingEnvironment ? '正在检测环境' : '检测训练环境' }}
          <el-icon><ArrowRight /></el-icon>
        </button>
      </section>

      <section class="signal-visual" aria-label="音频训练流程示意">
        <div class="signal-grid" />
        <div class="waveform" aria-hidden="true" :class="{ scanning: isCheckingEnvironment }">
          <span
            v-for="(height, index) in [18, 32, 48, 25, 66, 92, 58, 36, 76, 112, 72, 44, 84, 53, 30, 64, 42, 24, 37, 19]"
            :key="index"
            :style="{ height: `${height}px`, animationDelay: `${index * 35}ms` }"
          />
        </div>
        <div class="signal-caption">
          <span>{{ isCheckingEnvironment ? 'SCANNING SYSTEM' : 'INPUT SIGNAL' }}</span>
          <strong>{{ environmentReport?.platform.architecture ?? '40 kHz' }}</strong>
        </div>
      </section>
    </div>

    <section class="status-strip">
      <div class="status-heading">
        <span class="status-dot" :class="environmentStatus" />
        <div>
          <strong>{{ environmentCopy.title }}</strong>
          <span>{{ environmentCopy.detail }}</span>
        </div>
      </div>
      <div class="status-metrics">
        <div><span>活动项目</span><strong>0</strong></div>
        <div><span>可用内存</span><strong>{{ formatBytes(environmentReport?.resources.availableMemoryBytes ?? 0) }}</strong></div>
        <div><span>磁盘可用</span><strong>{{ formatBytes(environmentReport?.resources.availableDiskBytes ?? 0) }}</strong></div>
      </div>
      <el-button plain :icon="Refresh" :loading="isCheckingEnvironment" @click="runEnvironmentCheck">
        {{ environmentReport ? '重新检测' : '开始检测' }}
      </el-button>
    </section>

    <section class="setup-section">
      <div class="section-heading">
        <div>
          <span class="section-index">首次使用</span>
          <h3>开始训练前</h3>
        </div>
        <span>3 个步骤</span>
      </div>

      <div class="setup-list">
        <button
          v-for="(step, index) in setupSteps"
          :key="step.title"
          class="setup-row"
          type="button"
          @click="handleSetupStep(index)"
        >
          <span class="step-number">0{{ index + 1 }}</span>
          <span class="step-copy">
            <strong>{{ step.title }}</strong>
            <span>{{ step.detail }}</span>
          </span>
          <el-icon v-if="step.state === 'current'" class="step-state current"><Warning /></el-icon>
          <el-icon v-else class="step-state" :class="{ complete: step.state === 'complete' }"><CircleCheck /></el-icon>
        </button>
      </div>
    </section>
  </section>
</template>
