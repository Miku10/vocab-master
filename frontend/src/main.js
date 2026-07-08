import { createApp } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import App from './App.vue'
import router from './router'

const app = createApp(App)
app.config.globalProperties.$invoke = invoke
app.use(router)
app.mount('#app')
