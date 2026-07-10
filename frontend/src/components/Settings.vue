<template>
  <div class="fixed inset-0 z-50 bg-slate-950/45 sm:p-4">
    <div class="mx-auto flex h-[100dvh] w-full max-w-3xl flex-col bg-white shadow-2xl sm:h-auto sm:max-h-[92dvh] sm:rounded-2xl">
      <div class="flex shrink-0 items-center justify-between border-b border-slate-200 px-5 py-4 sm:px-6">
        <div>
          <h2 class="text-xl font-bold text-slate-900">{{ initialSetup ? '首次设置' : '设置' }}</h2>
          <p v-if="initialSetup" class="mt-1 text-sm text-slate-500">选择学段和每日任务后就可以开始学习。</p>
        </div>
        <button
          v-if="!initialSetup"
          @click="emit('close')"
          class="grid h-10 w-10 place-items-center rounded-full text-2xl leading-none text-slate-400 hover:bg-slate-100 hover:text-slate-700"
          aria-label="关闭设置"
        >
          ×
        </button>
      </div>

      <div class="flex-1 overflow-y-auto px-5 py-5 sm:px-6">
        <div class="space-y-5">
          <section class="rounded-2xl bg-slate-50 p-4 sm:p-5">
            <h3 class="mb-4 text-base font-semibold text-slate-900">学习设置</h3>
            <div class="grid gap-4 sm:grid-cols-3">
              <label class="block sm:col-span-1">
                <span class="mb-1 block text-sm font-medium text-slate-600">当前学段</span>
                <select v-model="config.active_level" class="field">
                  <option v-for="level in levelOptions" :key="level.key" :value="level.key">
                    {{ level.label }}
                  </option>
                </select>
              </label>
              <label class="block">
                <span class="mb-1 block text-sm font-medium text-slate-600">每日新词</span>
                <input v-model.number="config.daily_new_words" type="number" min="1" max="200" class="field" />
              </label>
              <label class="block">
                <span class="mb-1 block text-sm font-medium text-slate-600">每日复习</span>
                <input v-model.number="config.daily_review_words" type="number" min="0" max="300" class="field" />
              </label>
            </div>
          </section>

          <section class="rounded-2xl bg-slate-50 p-4 sm:p-5">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
              <h3 class="text-base font-semibold text-slate-900">词库</h3>
              <span class="text-xs text-slate-500">导入 JSON 会覆盖所选学段词库</span>
            </div>
            <div class="grid gap-3 sm:grid-cols-[180px_1fr_auto]">
              <select v-model="importLevel" class="field">
                <option v-for="level in levelOptions" :key="level.key" :value="level.key">
                  {{ level.label }}
                </option>
              </select>
              <input
                ref="fileInput"
                type="file"
                accept=".json,application/json"
                class="field file:mr-3 file:rounded-lg file:border-0 file:bg-blue-50 file:px-3 file:py-1.5 file:text-sm file:font-medium file:text-blue-700"
                @change="importWords"
              />
              <button type="button" class="btn-secondary" @click="clearImportFile">清空</button>
            </div>
            <p v-if="importMessage" class="mt-3 text-sm" :class="importError ? 'text-red-600' : 'text-emerald-600'">
              {{ importMessage }}
            </p>
          </section>

          <section class="rounded-2xl bg-slate-50 p-4 sm:p-5">
            <h3 class="mb-4 text-base font-semibold text-slate-900">模型配置</h3>
            <div class="mb-4 flex flex-wrap gap-2">
              <button
                v-for="preset in modelPresets"
                :key="preset.key"
                type="button"
                class="rounded-xl px-3 py-2 text-sm font-medium transition"
                :class="isPresetActive(preset)
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'bg-white text-slate-600 ring-1 ring-slate-200 hover:bg-slate-100'"
                @click="applyModelPreset(preset)"
              >
                {{ preset.label }}
              </button>
            </div>
            <div class="space-y-4">
              <label class="block">
                <span class="mb-1 block text-sm font-medium text-slate-600">API 地址</span>
                <input v-model="config.model.api_url" class="field" />
              </label>
              <label class="block">
                <span class="mb-1 block text-sm font-medium text-slate-600">API 密钥</span>
                <input v-model="config.model.api_key" type="password" class="field" />
              </label>
              <div class="grid gap-4 sm:grid-cols-3">
                <label class="block sm:col-span-1">
                  <span class="mb-1 block text-sm font-medium text-slate-600">模型名称</span>
                  <input v-model="config.model.model_name" class="field" />
                </label>
                <label class="block">
                  <span class="mb-1 block text-sm font-medium text-slate-600">最大 Token</span>
                  <input v-model.number="config.model.max_tokens" type="number" min="256" max="8192" class="field" />
                </label>
                <label class="block">
                  <span class="mb-1 block text-sm font-medium text-slate-600">温度</span>
                  <input v-model.number="config.model.temperature" type="number" step="0.1" min="0" max="2" class="field" />
                </label>
              </div>
            </div>
          </section>

          <section class="rounded-2xl bg-slate-50 p-4 sm:p-5">
            <h3 class="mb-4 text-base font-semibold text-slate-900">联网搜索</h3>
            <div class="space-y-4">
              <label class="flex items-center gap-3">
                <input v-model="config.search.enabled" type="checkbox" class="h-5 w-5 rounded text-blue-600" />
                <span class="text-sm font-medium text-slate-700">启用联网搜索</span>
              </label>
              <div class="grid gap-4 sm:grid-cols-2">
                <label class="block">
                  <span class="mb-1 block text-sm font-medium text-slate-600">搜索条数</span>
                  <input v-model.number="config.search.search_count" type="number" min="1" max="10" class="field" />
                </label>
                <label class="block">
                  <span class="mb-1 block text-sm font-medium text-slate-600">超时秒数</span>
                  <input v-model.number="config.search.timeout_seconds" type="number" min="5" max="60" class="field" />
                </label>
              </div>
            </div>
          </section>

          <section class="rounded-2xl bg-slate-50 p-4 sm:p-5">
            <h3 class="mb-4 text-base font-semibold text-slate-900">应用设置</h3>
            <label class="block">
              <span class="mb-1 block text-sm font-medium text-slate-600">音频缓存过期时间（小时）</span>
              <input v-model.number="config.audio_expire_hours" type="number" min="1" max="24" class="field" />
            </label>
          </section>

          <section class="rounded-2xl bg-slate-50 p-4 sm:p-5">
            <h3 class="mb-4 text-base font-semibold text-slate-900">预置提示词</h3>
            <div class="space-y-3">
              <div class="grid gap-2 sm:grid-cols-[1fr_auto]">
                <select v-model="selectedPrompt" class="field">
                  <option v-for="(val, key) in config.prompts" :key="key" :value="key">{{ key }}</option>
                </select>
                <button type="button" class="btn-secondary" @click="addPrompt">新增</button>
              </div>
              <textarea
                v-model="promptEdit"
                rows="5"
                class="field min-h-32 resize-y"
                placeholder="用 {var} 表示变量..."
              ></textarea>
              <button type="button" class="btn-secondary" @click="savePrompt">保存当前提示词</button>
            </div>
          </section>
        </div>
      </div>

      <div class="flex shrink-0 flex-col-reverse gap-3 border-t border-slate-200 bg-white px-5 py-4 sm:flex-row sm:justify-end sm:px-6">
        <button v-if="!initialSetup" type="button" class="btn-secondary" @click="emit('close')">取消</button>
        <button type="button" class="btn-primary" @click="saveAll">
          {{ initialSetup ? '保存并开始' : '保存所有设置' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

defineProps({
  initialSetup: { type: Boolean, default: false },
})

const emit = defineEmits(['close'])

const levelOptions = [
  { key: 'junior', label: '初中' },
  { key: 'high', label: '高中' },
  { key: 'cet4', label: '四级' },
  { key: 'cet6', label: '六级' },
  { key: 'primary', label: '小学（导入）' },
]

const modelPresets = [
  {
    key: 'agnes',
    label: 'Agnes 2.0 Flash',
    api_url: 'https://apihub.agnes-ai.com/v1/chat/completions',
    model_name: 'agnes-2.0-flash',
    max_tokens: 2000,
    temperature: 0.7,
  },
  {
    key: 'openai',
    label: 'OpenAI',
    api_url: 'https://api.openai.com/v1/chat/completions',
    model_name: 'gpt-4o',
    max_tokens: 2000,
    temperature: 0.7,
  },
]

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
  daily_new_words: 20,
  daily_review_words: 30,
  active_level: 'junior',
  setup_complete: false,
  prompts: {},
})

const selectedPrompt = ref('')
const promptEdit = ref('')
const importLevel = ref('junior')
const importMessage = ref('')
const importError = ref(false)
const fileInput = ref(null)

invoke('get_config').then(data => {
  if (data.model) Object.assign(config.model, data.model)
  if (data.search) Object.assign(config.search, data.search)
  config.audio_expire_hours = data.audio_expire_hours ?? config.audio_expire_hours
  config.daily_new_words = data.daily_new_words ?? config.daily_new_words
  config.daily_review_words = data.daily_review_words ?? config.daily_review_words
  config.active_level = data.active_level || config.active_level
  config.setup_complete = data.setup_complete ?? config.setup_complete
  importLevel.value = config.active_level
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
  if (val && config.prompts[val] !== undefined) {
    promptEdit.value = config.prompts[val]
  }
})

function applyModelPreset(preset) {
  config.model.api_url = preset.api_url
  config.model.model_name = preset.model_name
  config.model.max_tokens = preset.max_tokens
  config.model.temperature = preset.temperature
}

function isPresetActive(preset) {
  return config.model.api_url === preset.api_url && config.model.model_name === preset.model_name
}

async function importWords(event) {
  const file = event.target.files?.[0]
  if (!file) return

  importMessage.value = ''
  importError.value = false
  try {
    const content = await file.text()
    const count = await invoke('import_words', {
      level: importLevel.value,
      content,
    })
    config.active_level = importLevel.value
    importMessage.value = `已导入 ${count} 个单词，并切换到该学段。`
  } catch (e) {
    importError.value = true
    importMessage.value = `导入失败：${e}`
  }
}

function clearImportFile() {
  if (fileInput.value) fileInput.value.value = ''
  importMessage.value = ''
  importError.value = false
}

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
    config.daily_new_words = Math.max(1, Number(config.daily_new_words) || 20)
    config.daily_review_words = Math.max(0, Number(config.daily_review_words) || 0)
    config.audio_expire_hours = Math.max(1, Number(config.audio_expire_hours) || 5)
    config.setup_complete = true
    await invoke('save_config', { config })
    emit('close')
  } catch (e) {
    alert('保存失败: ' + e)
  }
}
</script>

<style scoped>
.field {
  width: 100%;
  border-radius: 0.875rem;
  border: 1px solid rgb(203 213 225);
  background: white;
  padding: 0.625rem 0.875rem;
  color: rgb(15 23 42);
  outline: none;
}

.field:focus {
  border-color: rgb(59 130 246);
  box-shadow: 0 0 0 3px rgb(191 219 254);
}

.btn-primary {
  border-radius: 0.875rem;
  background: rgb(37 99 235);
  padding: 0.75rem 1.25rem;
  font-size: 0.875rem;
  font-weight: 700;
  color: white;
}

.btn-primary:hover {
  background: rgb(29 78 216);
}

.btn-secondary {
  border-radius: 0.875rem;
  background: rgb(241 245 249);
  padding: 0.75rem 1.25rem;
  font-size: 0.875rem;
  font-weight: 600;
  color: rgb(51 65 85);
}

.btn-secondary:hover {
  background: rgb(226 232 240);
}
</style>
