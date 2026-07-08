<template>
  <div class="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-3xl shadow-2xl w-full max-w-2xl max-h-[85vh] overflow-hidden">
      <div class="p-6 border-b border-slate-100 flex items-center justify-between">
        <h2 class="text-xl font-bold text-slate-800">⚙️ 设置</h2>
        <button @click="$emit('close')" class="text-slate-400 hover:text-slate-600 text-2xl">×</button>
      </div>
      
      <div class="overflow-y-auto max-h-[70vh] p-6 space-y-6">
        <!-- 模型配置 -->
        <div class="bg-slate-50 rounded-2xl p-5">
          <h3 class="font-semibold text-slate-800 mb-4">🤖 模型配置</h3>
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-slate-600 mb-1">API 地址</label>
              <input v-model="config.model.api_url" class="w-full px-4 py-2.5 rounded-xl border border-slate-200 
                                          focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-600 mb-1">API 密钥</label>
              <input v-model="config.model.api_key" type="password" class="w-full px-4 py-2.5 rounded-xl border border-slate-200 
                                          focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-600 mb-1">模型名称</label>
              <input v-model="config.model.model_name" class="w-full px-4 py-2.5 rounded-xl border border-slate-200 
                                          focus:outline-none focus:ring-2 focus:ring-blue-500" />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-slate-600 mb-1">最大 Token</label>
                <input v-model.number="config.model.max_tokens" type="number" min="256" max="4096"
                       class="w-full px-4 py-2.5 rounded-xl border border-slate-200" />
              </div>
              <div>
                <label class="block text-sm font-medium text-slate-600 mb-1">温度</label>
                <input v-model.number="config.model.temperature" type="number" step="0.1" min="0" max="2"
                       class="w-full px-4 py-2.5 rounded-xl border border-slate-200" />
              </div>
            </div>
          </div>
        </div>

        <!-- 搜索配置 -->
        <div class="bg-slate-50 rounded-2xl p-5">
          <h3 class="font-semibold text-slate-800 mb-4">🔍 联网搜索</h3>
          <div class="space-y-4">
            <label class="flex items-center gap-3">
              <input v-model="config.search.enabled" type="checkbox" class="w-5 h-5 rounded text-blue-600" />
              <span class="text-sm font-medium text-slate-700">启用联网搜索</span>
            </label>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-slate-600 mb-1">搜索条数</label>
                <input v-model.number="config.search.search_count" type="number" min="1" max="10"
                       class="w-full px-4 py-2.5 rounded-xl border border-slate-200" />
              </div>
              <div>
                <label class="block text-sm font-medium text-slate-600 mb-1">超时(秒)</label>
                <input v-model.number="config.search.timeout_seconds" type="number" min="5" max="60"
                       class="w-full px-4 py-2.5 rounded-xl border border-slate-200" />
              </div>
            </div>
          </div>
        </div>

        <!-- 应用设置 -->
        <div class="bg-slate-50 rounded-2xl p-5">
          <h3 class="font-semibold text-slate-800 mb-4">⚙️ 应用设置</h3>
          <div>
            <label class="block text-sm font-medium text-slate-600 mb-1">音频缓存过期时间(小时)</label>
            <input v-model.number="config.audio_expire_hours" type="number" min="1" max="24"
                   class="w-full px-4 py-2.5 rounded-xl border border-slate-200" />
          </div>
        </div>

        <!-- 提示词管理 -->
        <div class="bg-slate-50 rounded-2xl p-5">
          <h3 class="font-semibold text-slate-800 mb-4">📝 预置提示词</h3>
          <div class="space-y-3">
            <div class="flex gap-2">
              <select v-model="selectedPrompt" class="flex-1 px-4 py-2.5 rounded-xl border border-slate-200 bg-white">
                <option v-for="(val, key) in config.prompts" :key="key" :value="key">{{ key }}</option>
              </select>
              <button @click="addPrompt" class="px-4 py-2.5 rounded-xl bg-blue-50 text-blue-700 hover:bg-blue-100 text-sm">＋</button>
            </div>
            <textarea v-model="promptEdit" rows="5" class="w-full px-4 py-3 rounded-xl border border-slate-200 
                                              focus:outline-none focus:ring-2 focus:ring-blue-500 resize-y"
                      placeholder="用 {var} 表示变量..."></textarea>
            <button @click="savePrompt" class="px-5 py-2.5 rounded-xl bg-indigo-50 text-indigo-700 hover:bg-indigo-100 text-sm font-medium">
              💾 保存当前提示词
            </button>
          </div>
        </div>
      </div>

      <div class="p-6 border-t border-slate-100 flex justify-end gap-3">
        <button @click="$emit('close')" class="px-5 py-2.5 rounded-xl text-sm font-medium text-slate-600 hover:bg-slate-100">
          取消
        </button>
        <button @click="saveAll" class="px-5 py-2.5 rounded-xl text-sm font-medium text-white bg-blue-600 hover:bg-blue-700">
          ✅ 保存所有设置
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['close'])

const config = reactive({
  model: {
    api_url: 'https://api.openai.com/v1/chat/completions',
    api_key: '',
    model_name: 'gpt-4o',
    max_tokens: 2000,
    temperature: 0.7,
  },
  search: {
    enabled: false,
    search_count: 5,
    timeout_seconds: 15,
  },
  audio_expire_hours: 5,
  prompts: {},
})

const selectedPrompt = ref('')
const promptEdit = ref('')

// Load config on mount
invoke('get_config').then(data => {
  if (data.model) Object.assign(config.model, data.model)
  if (data.search) Object.assign(config.search, data.search)
  if (data.audio_expire_hours) config.audio_expire_hours = data.audio_expire_hours
  if (data.prompts) {
    Object.assign(config.prompts, data.prompts)
    const keys = Object.keys(config.prompts)
    if (keys.length > 0) {
      selectedPrompt.value = keys[0]
      promptEdit.value = config.prompts[keys[0]]
    }
  }
})

watch(selectedPrompt, (val) => {
  if (val && config.prompts[val]) {
    promptEdit.value = config.prompts[val]
  }
})

function addPrompt() {
  const name = prompt('请输入提示词名称:')
  if (name && !config.prompts[name]) {
    config.prompts[name] = ''
    selectedPrompt.value = name
    promptEdit.value = ''
  }
}

function savePrompt() {
  if (selectedPrompt.value) {
    config.prompts[selectedPrompt.value] = promptEdit.value
  }
}

async function saveAll() {
  try {
    await invoke('save_config', { config })
    emit('close')
  } catch (e) {
    alert('保存失败: ' + e)
  }
}
</script>
