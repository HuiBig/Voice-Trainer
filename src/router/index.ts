import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import PlaceholderView from '../views/PlaceholderView.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    {
      path: '/projects',
      name: 'projects',
      component: PlaceholderView,
      props: { title: '训练项目', description: '项目创建与素材管理将在下一阶段接入。' },
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
      component: PlaceholderView,
      props: { title: '设置', description: '运行环境、存储路径和日志设置将在这里管理。' },
    },
  ],
})

export default router
