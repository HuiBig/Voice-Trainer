import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import EnvironmentView from '../views/EnvironmentView.vue'
import PlaceholderView from '../views/PlaceholderView.vue'
import ProjectsView from '../views/ProjectsView.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    {
      path: '/projects',
      name: 'projects',
      component: ProjectsView,
    },
    {
      path: '/training',
      name: 'training',
      component: PlaceholderView,
      props: { title: '训练任务', description: '这里将显示预处理、特征提取和训练进度。' },
    },
    {
      path: '/exports',
      name: 'exports',
      component: PlaceholderView,
      props: { title: '模型导出', description: '通过验证的模型将在这里导出为 .vcpkg。' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: EnvironmentView,
    },
  ],
})

export default router
