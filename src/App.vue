<script setup lang="ts">
import {
  Cpu,
  DataAnalysis,
  FolderOpened,
  House,
  Setting,
  UploadFilled,
} from '@element-plus/icons-vue'
import { storeToRefs } from 'pinia'
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from './stores/app'

const route = useRoute()
const router = useRouter()
const appStore = useAppStore()
const { environmentStatus } = storeToRefs(appStore)

const runtimeCopy = computed(() => {
  const copy = {
    pending: '等待环境检测',
    checking: '正在检测环境',
    ready: '训练依赖已就绪',
    incomplete: '训练依赖不完整',
    error: '检测失败',
  }
  return copy[environmentStatus.value]
})

const navigation = [
  { label: '工作台', path: '/', icon: House },
  { label: '训练项目', path: '/projects', icon: FolderOpened },
  { label: '训练任务', path: '/training', icon: DataAnalysis },
  { label: '模型导出', path: '/exports', icon: UploadFilled },
]
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark" aria-hidden="true">
          <span v-for="height in [9, 17, 25, 17, 9]" :key="height" :style="{ height: `${height}px` }" />
        </div>
        <div>
          <strong>Voice Trainer</strong>
          <span>本地人声训练</span>
        </div>
      </div>

      <nav class="navigation" aria-label="主导航">
        <button
          v-for="item in navigation"
          :key="item.path"
          class="nav-item"
          :class="{ active: route.path === item.path }"
          type="button"
          @click="router.push(item.path)"
        >
          <el-icon><component :is="item.icon" /></el-icon>
          <span>{{ item.label }}</span>
        </button>
      </nav>

      <div class="sidebar-footer">
        <div class="runtime-state" role="status">
          <span class="status-dot" :class="environmentStatus" />
          <div>
            <strong>训练引擎</strong>
            <span>{{ runtimeCopy }}</span>
          </div>
          <el-icon><Cpu /></el-icon>
        </div>
        <button class="nav-item" type="button" @click="router.push('/settings')">
          <el-icon><Setting /></el-icon>
          <span>设置</span>
        </button>
      </div>
    </aside>

    <main class="workspace">
      <router-view v-slot="{ Component }">
        <transition name="page" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
  </div>
</template>
