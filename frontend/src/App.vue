<template>
  <div v-if="appLoading" class="grid h-screen place-items-center bg-slate-50">
    <div class="text-center">
      <div class="mx-auto mb-4 h-14 w-14 animate-spin rounded-2xl border-4 border-blue-100 border-t-blue-600"></div>
      <p class="text-sm font-semibold text-slate-700">Vocab Master 正在加载...</p>
    </div>
  </div>

  <div v-else class="flex h-screen bg-slate-50">
    <!-- 侧边栏 -->
    <aside class="w-64 bg-white border-r border-slate-200 flex flex-col">
      <!-- Logo -->
      <div class="p-6 border-b border-slate-100">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-indigo-600 
                      flex items-center justify-center text-white text-xl font-bold">V</div>
          <div>
            <h1 class="font-bold text-slate-800">Vocab Master</h1>
            <p class="text-xs text-slate-400">英语词汇大师</p>
          </div>
        </div>
      </div>

      <!-- 导航 -->
      <nav class="flex-1 p-4 space-y-1 overflow-y-auto">
        <router-link v-for="item in menuItems" :key="item.path"
                     :to="item.path"
                     class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium
                            transition-all duration-200"
                     :class="isActive(item.path) 
                        ? 'bg-blue-50 text-blue-700 shadow-sm' 
                        : 'text-slate-600 hover:bg-slate-100'">
          <span class="text-lg">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </router-link>
      </nav>

      <!-- 底部 -->
      <div class="p-4 border-t border-slate-100">
        <button @click="openSettings"
                class="flex items-center gap-3 px-4 py-3 rounded-xl w-full
                       text-sm font-medium text-slate-600 hover:bg-slate-100 transition-all">
          <span class="text-lg">⚙️</span>
          <span>设置</span>
        </button>
        <p class="mt-3 px-4 text-xs text-slate-400">© 2026 miku</p>
      </div>
    </aside>

    <!-- 主内容 -->
    <main class="flex-1 overflow-auto">
      <router-view />
    </main>

    <!-- 设置弹窗 -->
    <Settings v-if="showSettings" :initial-setup="needsSetup" @close="closeSettings" />
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import Settings from './components/Settings.vue'

const route = useRoute()
const showSettings = ref(false)
const needsSetup = ref(false)
const appLoading = ref(true)

const menuItems = [
  { path: '/', icon: '📊', label: '学习仪表盘' },
  { path: '/words', icon: '📚', label: '词库学习' },
  { path: '/review', icon: '🔄', label: '复习巩固' },
  { path: '/quiz', icon: '📝', label: '模拟测试' },
]

function isActive(path) {
  return route.path === path
}

function openSettings() {
  needsSetup.value = false
  showSettings.value = true
}

async function closeSettings() {
  needsSetup.value = false
  showSettings.value = false
  try {
    const config = await invoke('get_config')
    triggerWordBankEnrichment(config)
  } catch (e) {
    console.warn('读取保存后的配置失败:', e)
  }
}

onMounted(async () => {
  try {
    const config = await invoke('get_config')
    if (!config.setup_complete) {
      needsSetup.value = true
      showSettings.value = true
    } else {
      triggerWordBankEnrichment(config)
    }
  } catch (e) {
    console.warn('读取配置失败:', e)
  }
  appLoading.value = false
})

function triggerWordBankEnrichment(config) {
  const key = String(config?.model?.api_key || '').trim()
  const level = String(config?.active_level || '').trim()
  if (!key || !level || isPlaceholderApiKey(key)) return
  invoke('auto_enrich_word_bank_examples', { config, level })
    .catch(e => console.warn('后台补充词库例句失败:', e))
}

function isPlaceholderApiKey(key) {
  return ['api_key', 'your_api_key', 'your-api-key', 'sk-xxx', 'sk-xxxx'].includes(key.toLowerCase())
}
</script>
