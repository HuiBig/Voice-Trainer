<script setup lang="ts">
import { DataAnalysis, Delete, EditPen, FolderOpened, MoreFilled, Plus, Refresh, Search } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { storeToRefs } from 'pinia'
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore, type TrainingProject } from '../stores/app'
import { formatBytes } from '../utils/format'

type DialogMode = 'create' | 'rename' | 'delete' | null

const appStore = useAppStore()
const route = useRoute()
const router = useRouter()
const { activeProjectId, isLoadingProjects, isScanningMaterials, materialScanReport, projectError, projects } = storeToRefs(appStore)
const query = ref('')
const dialogMode = ref<DialogMode>(null)
const selectedProject = ref<TrainingProject | null>(null)
const isSubmitting = ref(false)
const form = reactive({ name: '', materialDirectory: '' })

const visibleProjects = computed(() => {
  const keyword = query.value.trim().toLocaleLowerCase()
  if (!keyword) return projects.value
  return projects.value.filter((project) =>
    `${project.name} ${project.materialDirectory}`.toLocaleLowerCase().includes(keyword),
  )
})

const activeProject = computed(() =>
  projects.value.find((project) => String(project.id) === activeProjectId.value) ?? null,
)

const dialogTitle = computed(() => {
  if (dialogMode.value === 'create') return '新建训练项目'
  if (dialogMode.value === 'rename') return '重命名项目'
  return '删除项目'
})

const statusCopy: Record<string, string> = {
  not_started: '尚未训练', preparing: '准备中', training: '训练中', completed: '已完成', failed: '失败',
}

function formatDate(timestamp: number) {
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
  }).format(new Date(timestamp * 1000))
}

function showCreate() {
  selectedProject.value = null
  form.name = ''
  form.materialDirectory = ''
  dialogMode.value = 'create'
}

function showRename(project: TrainingProject) {
  selectedProject.value = project
  form.name = project.name
  form.materialDirectory = project.materialDirectory
  dialogMode.value = 'rename'
}

function showDelete(project: TrainingProject) {
  selectedProject.value = project
  dialogMode.value = 'delete'
}

function closeDialog() {
  if (!isSubmitting.value) dialogMode.value = null
}

async function submitDialog() {
  isSubmitting.value = true
  try {
    if (dialogMode.value === 'create') {
      const project = await appStore.createProject({ name: form.name, materialDirectory: form.materialDirectory })
      ElMessage.success(`已创建“${project.name}”`)
    } else if (dialogMode.value === 'rename' && selectedProject.value) {
      await appStore.renameProject(selectedProject.value.id, form.name)
      ElMessage.success('项目已重命名')
    } else if (dialogMode.value === 'delete' && selectedProject.value) {
      await appStore.deleteProject(selectedProject.value.id)
      ElMessage.success('项目已删除')
    }
    dialogMode.value = null
  } catch (error) {
    ElMessage.error(typeof error === 'string' ? error : '项目操作失败')
  } finally {
    isSubmitting.value = false
  }
}

async function openProject(project: TrainingProject) {
  try {
    await appStore.openProject(project.id)
    ElMessage.success(`已打开“${project.name}”`)
  } catch (error) {
    ElMessage.error(typeof error === 'string' ? error : '无法打开项目')
  }
}

async function scanMaterials() {
  if (!activeProject.value) return
  try {
    const report = await appStore.scanMaterialDirectory(activeProject.value.materialDirectory)
    ElMessage.success(`已发现 ${report.audioFileCount} 个音频文件`)
  } catch (error) {
    ElMessage.error(typeof error === 'string' ? error : '素材目录扫描失败')
  }
}

async function refresh() {
  try { await appStore.loadProjects() } catch { ElMessage.error(projectError.value ?? '无法刷新项目列表') }
}

watch(() => route.query.create, (value) => {
  if (value === '1') {
    showCreate()
    router.replace({ path: '/projects' })
  }
}, { immediate: true })

onMounted(() => {
  if (!appStore.projectsLoaded) refresh()
})
</script>

<template>
  <section class="projects-view">
    <header class="topbar projects-header">
      <div>
        <span class="eyebrow">本地项目库</span>
        <h1>训练项目</h1>
        <p>项目数据保存在本机 SQLite，素材目录只记录路径，不会复制或上传。</p>
      </div>
      <el-button type="primary" :icon="Plus" @click="showCreate">新建项目</el-button>
    </header>

    <section v-if="activeProject" class="active-project" aria-label="当前项目">
      <div class="active-signal"><span v-for="height in [12, 24, 18, 34, 22, 30, 15]" :key="height" :style="{ height: `${height}px` }" /></div>
      <div>
        <span class="section-index">当前项目</span>
        <h2>{{ activeProject.name }}</h2>
      </div>
      <div class="active-path">
        <span>素材目录</span>
        <strong>{{ activeProject.materialDirectory }}</strong>
      </div>
      <span class="project-status">{{ statusCopy[activeProject.trainingStatus] }}</span>
      <el-button plain :icon="DataAnalysis" :loading="isScanningMaterials" @click="scanMaterials">扫描素材</el-button>
    </section>

    <section v-if="activeProject && materialScanReport?.directory === activeProject.materialDirectory" class="status-strip material-summary">
      <div class="status-heading">
        <span class="status-dot" :class="materialScanReport.issues.length ? 'incomplete' : 'ready'" />
        <div><strong>素材扫描完成</strong><span>{{ materialScanReport.directory }}</span></div>
      </div>
      <div class="status-metrics">
        <div><span>音频文件</span><strong>{{ materialScanReport.audioFileCount }}</strong></div>
        <div><span>音频大小</span><strong>{{ formatBytes(materialScanReport.totalAudioBytes) }}</strong></div>
        <div><span>其他文件</span><strong>{{ materialScanReport.unsupportedFileCount }}</strong></div>
        <div><span>读取问题</span><strong>{{ materialScanReport.issues.length }}</strong></div>
      </div>
    </section>

    <section class="project-library">
      <div class="library-toolbar">
        <div>
          <span class="section-index">项目列表</span>
          <h3>{{ projects.length }} 个本地项目</h3>
        </div>
        <div class="library-actions">
          <label class="search-field">
            <el-icon><Search /></el-icon>
            <input v-model="query" type="search" placeholder="搜索名称或素材目录" />
          </label>
          <button class="icon-button" type="button" title="刷新项目列表" :disabled="isLoadingProjects" @click="refresh">
            <el-icon :class="{ spinning: isLoadingProjects }"><Refresh /></el-icon>
          </button>
        </div>
      </div>

      <div v-if="projectError && !projects.length" class="project-empty">
        <span class="empty-index">ERR</span><h2>无法读取项目库</h2><p>{{ projectError }}</p>
        <el-button plain :icon="Refresh" @click="refresh">重新加载</el-button>
      </div>
      <div v-else-if="!isLoadingProjects && !visibleProjects.length" class="project-empty">
        <el-icon><FolderOpened /></el-icon>
        <h2>{{ query ? '没有匹配的项目' : '还没有训练项目' }}</h2>
        <p>{{ query ? '尝试使用其他项目名称或素材目录。' : '创建项目后即可登记素材目录并进入训练流程。' }}</p>
        <el-button v-if="!query" type="primary" :icon="Plus" @click="showCreate">新建第一个项目</el-button>
      </div>
      <div v-else class="project-list" :class="{ loading: isLoadingProjects }">
        <article v-for="(project, index) in visibleProjects" :key="project.id" class="project-row" :class="{ active: String(project.id) === activeProjectId }">
          <span class="project-index">{{ String(index + 1).padStart(2, '0') }}</span>
          <button class="project-main" type="button" @click="openProject(project)">
            <strong>{{ project.name }}</strong><span>{{ project.materialDirectory }}</span>
          </button>
          <div class="project-meta"><span>创建时间</span><strong>{{ formatDate(project.createdAt) }}</strong></div>
          <span class="project-status">{{ statusCopy[project.trainingStatus] }}</span>
          <div class="row-actions">
            <button type="button" title="打开项目" @click="openProject(project)"><el-icon><FolderOpened /></el-icon></button>
            <button type="button" title="重命名项目" @click="showRename(project)"><el-icon><EditPen /></el-icon></button>
            <button class="danger" type="button" title="删除项目" @click="showDelete(project)"><el-icon><Delete /></el-icon></button>
          </div>
        </article>
      </div>
    </section>

    <transition name="dialog">
      <div v-if="dialogMode" class="dialog-backdrop" role="presentation" @mousedown.self="closeDialog">
        <section class="project-dialog" role="dialog" aria-modal="true" :aria-label="dialogTitle">
          <div class="dialog-kicker"><span>{{ dialogMode === 'create' ? 'NEW PROJECT' : 'PROJECT ACTION' }}</span><el-icon><MoreFilled /></el-icon></div>
          <h2>{{ dialogTitle }}</h2>
          <template v-if="dialogMode !== 'delete'">
            <label class="form-field"><span>项目名称</span><input v-model="form.name" maxlength="80" autofocus placeholder="例如：主唱模型 01" @keyup.enter="submitDialog" /></label>
            <label v-if="dialogMode === 'create'" class="form-field"><span>素材目录</span><input v-model="form.materialDirectory" placeholder="例如：D:\VoiceData\Singer01" @keyup.enter="submitDialog" /><small>请填写已获授权的人声音频所在目录。</small></label>
          </template>
          <p v-else class="delete-copy">确定删除“{{ selectedProject?.name }}”吗？此操作会删除项目记录，但不会删除素材目录中的文件。</p>
          <div class="dialog-actions">
            <el-button plain :disabled="isSubmitting" @click="closeDialog">取消</el-button>
            <el-button :type="dialogMode === 'delete' ? 'danger' : 'primary'" :loading="isSubmitting" @click="submitDialog">{{ dialogMode === 'delete' ? '确认删除' : '保存' }}</el-button>
          </div>
        </section>
      </div>
    </transition>
  </section>
</template>
