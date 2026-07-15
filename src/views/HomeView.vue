<script setup lang="ts">
import { ArrowRight, CircleCheck, Plus, Refresh, Warning } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'

const setupSteps = [
  { title: '检测训练环境', detail: '确认 FFmpeg、Python、PyTorch 与可用计算设备', state: 'current' },
  { title: '创建第一个项目', detail: '导入已获授权的目标人声音频', state: 'pending' },
  { title: '开始数据预处理', detail: '转码、切片并检查训练素材质量', state: 'pending' },
]

function notifyScaffold(action: string) {
  ElMessage.info(`${action}将在下一阶段接入`)
}
</script>

<template>
  <section class="home-view">
    <header class="topbar">
      <div>
        <span class="eyebrow">PC 训练端</span>
        <h1>工作台</h1>
      </div>
      <el-button type="primary" :icon="Plus" @click="notifyScaffold('项目创建')">新建训练项目</el-button>
    </header>

    <div class="intro-layout">
      <section class="intro-copy">
        <span class="section-index">01 / 准备</span>
        <h2>从干净的人声素材开始。</h2>
        <p>Voice Trainer 会在本机完成数据处理、RVC 模型训练和移动端模型包导出，音频默认不会离开设备。</p>
        <button class="text-action" type="button" @click="notifyScaffold('环境检测')">
          检测训练环境
          <el-icon><ArrowRight /></el-icon>
        </button>
      </section>

      <section class="signal-visual" aria-label="音频训练流程示意">
        <div class="signal-grid" />
        <div class="waveform" aria-hidden="true">
          <span
            v-for="(height, index) in [18, 32, 48, 25, 66, 92, 58, 36, 76, 112, 72, 44, 84, 53, 30, 64, 42, 24, 37, 19]"
            :key="index"
            :style="{ height: `${height}px`, animationDelay: `${index * 35}ms` }"
          />
        </div>
        <div class="signal-caption">
          <span>INPUT SIGNAL</span>
          <strong>40 kHz</strong>
        </div>
      </section>
    </div>

    <section class="status-strip">
      <div class="status-heading">
        <span class="status-dot warning" />
        <div>
          <strong>环境尚未检测</strong>
          <span>开始训练前需要完成一次本机运行环境检查</span>
        </div>
      </div>
      <div class="status-metrics">
        <div><span>活动项目</span><strong>0</strong></div>
        <div><span>训练任务</span><strong>空闲</strong></div>
        <div><span>磁盘缓存</span><strong>—</strong></div>
      </div>
      <el-button plain :icon="Refresh" @click="notifyScaffold('环境检测')">开始检测</el-button>
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
          @click="notifyScaffold(step.title)"
        >
          <span class="step-number">0{{ index + 1 }}</span>
          <span class="step-copy">
            <strong>{{ step.title }}</strong>
            <span>{{ step.detail }}</span>
          </span>
          <el-icon v-if="step.state === 'current'" class="step-state current"><Warning /></el-icon>
          <el-icon v-else class="step-state"><CircleCheck /></el-icon>
        </button>
      </div>
    </section>
  </section>
</template>
