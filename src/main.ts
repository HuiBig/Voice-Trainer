import { createPinia } from 'pinia'
import { createApp } from 'vue'
import { ElButton, ElIcon } from 'element-plus'
import 'element-plus/es/components/base/style/css'
import 'element-plus/es/components/button/style/css'
import 'element-plus/es/components/icon/style/css'
import 'element-plus/es/components/message/style/css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import App from './App.vue'
import router from './router'
import './style.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.component('ElButton', ElButton)
app.component('ElIcon', ElIcon)
app.mount('#app')
