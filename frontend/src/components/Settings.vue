<template>
  <div class="fixed inset-0 z-50 bg-slate-950/45 sm:p-4">
    <div class="mx-auto flex h-[100dvh] w-full max-w-3xl flex-col bg-white shadow-2xl sm:h-auto sm:max-h-[92dvh] sm:rounded-2xl">
      <div class="shrink-0 border-b border-slate-200 px-5 py-4 sm:px-6">
        <div class="flex items-start justify-between gap-4">
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

        <div class="mt-4 grid grid-cols-4 gap-2 rounded-2xl bg-slate-100 p-1">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            type="button"
            class="rounded-xl px-2 py-2 text-sm font-semibold transition"
            :class="activeTab === tab.key ? 'bg-white text-blue-700 shadow-sm' : 'text-slate-600 hover:text-slate-900'"
            @click="activeTab = tab.key"
          >
            {{ tab.label }}
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto px-5 py-5 sm:px-6">
        <section v-if="activeTab === 'study'" class="space-y-5">
          <div class="panel">
            <h3 class="section-title">学习计划</h3>
            <div class="grid gap-4 sm:grid-cols-3">
              <label class="block">
                <span class="label">当前学段</span>
                <select v-model="config.active_level" class="field">
                  <option v-for="level in levelOptions" :key="level.key" :value="level.key">{{ level.label }}</option>
                </select>
              </label>
              <label class="block">
                <span class="label">每日新词</span>
                <input v-model.number="config.daily_new_words" type="number" min="1" max="200" class="field" />
              </label>
              <label class="block">
                <span class="label">每日复习</span>
                <input v-model.number="config.daily_review_words" type="number" min="0" max="300" class="field" />
              </label>
            </div>
          </div>

          <div class="panel">
            <h3 class="section-title">词卡交互</h3>
            <div class="grid gap-4 sm:grid-cols-2">
              <label class="block">
                <span class="label">详情后继续方式</span>
                <select v-model="config.card_advance_mode" class="field">
                  <option value="auto">自动进入下一个</option>
                  <option value="manual">点击继续</option>
                </select>
              </label>
              <label class="block">
                <span class="label">详情停留秒数</span>
                <input
                  v-model.number="config.card_detail_seconds"
                  type="number"
                  min="1"
                  max="10"
                  class="field"
                  :disabled="config.card_advance_mode === 'manual'"
                />
              </label>
            </div>
          </div>

          <div class="panel">
            <h3 class="section-title">应用</h3>
            <label class="block">
              <span class="label">音频缓存过期时间（小时）</span>
              <input v-model.number="config.audio_expire_hours" type="number" min="1" max="24" class="field" />
            </label>
          </div>
        </section>

        <section v-else-if="activeTab === 'words'" class="space-y-5">
          <div class="panel">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
              <h3 class="section-title mb-0">导入词库</h3>
              <span class="rounded-full bg-amber-100 px-3 py-1 text-xs font-semibold text-amber-800">确认后覆盖所选学段</span>
            </div>
            <div class="grid gap-3 sm:grid-cols-[180px_1fr]">
              <select v-model="importLevel" class="field">
                <option v-for="level in levelOptions" :key="level.key" :value="level.key">{{ level.label }}</option>
              </select>
              <input
                ref="fileInput"
                type="file"
                accept=".json,application/json"
                class="field file:mr-3 file:rounded-lg file:border-0 file:bg-blue-50 file:px-3 file:py-1.5 file:text-sm file:font-medium file:text-blue-700"
                @change="previewImport"
              />
            </div>

            <div v-if="importError" class="mt-4 rounded-xl bg-red-50 p-4 text-sm text-red-600">{{ importError }}</div>

            <div v-if="importPreview.length" class="mt-4 space-y-4">
              <div class="grid gap-3 sm:grid-cols-3">
                <div class="rounded-xl bg-white p-4 ring-1 ring-slate-200">
                  <p class="text-sm text-slate-500">总词数</p>
                  <p class="mt-1 text-2xl font-bold text-slate-900">{{ importStats.total }}</p>
                </div>
                <div class="rounded-xl bg-white p-4 ring-1 ring-slate-200">
                  <p class="text-sm text-slate-500">重复词</p>
                  <p class="mt-1 text-2xl font-bold" :class="importStats.duplicates ? 'text-amber-600' : 'text-emerald-600'">{{ importStats.duplicates }}</p>
                </div>
                <div class="rounded-xl bg-white p-4 ring-1 ring-slate-200">
                  <p class="text-sm text-slate-500">预览</p>
                  <p class="mt-1 text-2xl font-bold text-slate-900">{{ importPreview.length }}</p>
                </div>
              </div>

              <div class="overflow-hidden rounded-xl ring-1 ring-slate-200">
                <div v-for="item in importPreview" :key="item.index" class="grid grid-cols-[64px_1fr] gap-3 border-b border-slate-100 bg-white p-3 text-sm last:border-b-0">
                  <span class="text-slate-400">#{{ item.index + 1 }}</span>
                  <div>
                    <p class="font-semibold text-slate-900">{{ item.word }}</p>
                    <p class="line-clamp-2 text-slate-500">{{ item.definition }}</p>
                  </div>
                </div>
              </div>

              <div class="flex flex-col gap-3 sm:flex-row sm:justify-end">
                <button type="button" class="btn-secondary" @click="clearImportFile">重新选择</button>
                <button type="button" class="btn-primary" @click="confirmImport">确认覆盖导入</button>
              </div>
            </div>

            <p v-if="importMessage" class="mt-3 text-sm text-emerald-600">{{ importMessage }}</p>
          </div>
        </section>

        <section v-else-if="activeTab === 'ai'" class="space-y-5">
          <div class="panel">
            <h3 class="section-title">模型预设</h3>
            <div class="flex flex-wrap gap-2">
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
          </div>

          <div class="panel">
            <h3 class="section-title">模型配置</h3>
            <div class="space-y-4">
              <label class="block">
                <span class="label">API 地址</span>
                <input v-model="config.model.api_url" class="field" />
              </label>
              <label class="block">
                <span class="label">API 密钥</span>
                <input v-model="config.model.api_key" type="password" class="field" />
              </label>
              <div class="grid gap-4 sm:grid-cols-3">
                <label class="block">
                  <span class="label">模型名称</span>
                  <input v-model="config.model.model_name" class="field" />
                </label>
                <label class="block">
                  <span class="label">最大 Token</span>
                  <input v-model.number="config.model.max_tokens" type="number" min="256" max="8192" class="field" />
                </label>
                <label class="block">
                  <span class="label">温度</span>
                  <input v-model.number="config.model.temperature" type="number" step="0.1" min="0" max="2" class="field" />
                </label>
              </div>
            </div>
          </div>

          <div class="panel">
            <h3 class="section-title">预置提示词</h3>
            <div class="space-y-3">
              <div class="grid gap-2 sm:grid-cols-[1fr_auto]">
                <select v-model="selectedPrompt" class="field">
                  <option v-for="(val, key) in config.prompts" :key="key" :value="key">{{ key }}</option>
                </select>
                <button type="button" class="btn-secondary" @click="addPrompt">新增</button>
              </div>
              <textarea v-model="promptEdit" rows="5" class="field min-h-32 resize-y" placeholder="用 {var} 表示变量..."></textarea>
              <button type="button" class="btn-secondary" @click="savePrompt">保存当前提示词</button>
            </div>
          </div>
        </section>

        <section v-else class="space-y-5">
          <div class="panel">
            <h3 class="section-title">联网搜索</h3>
            <div class="space-y-4">
              <label class="flex items-center gap-3">
                <input v-model="config.search.enabled" type="checkbox" class="h-5 w-5 rounded text-blue-600" />
                <span class="text-sm font-medium text-slate-700">启用联网搜索</span>
              </label>
              <div class="grid gap-4 sm:grid-cols-2">
                <label class="block">
                  <span class="label">搜索条数</span>
                  <input v-model.number="config.search.search_count" type="number" min="1" max="10" class="field" />
                </label>
                <label class="block">
                  <span class="label">超时秒数</span>
                  <input v-model.number="config.search.timeout_seconds" type="number" min="5" max="60" class="field" />
                </label>
              </div>
            </div>
          </div>
        </section>
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

const tabs = [
  { key: 'study', label: '学习' },
  { key: 'words', label: '词库' },
  { key: 'ai', label: 'AI' },
  { key: 'search', label: '搜索' },
]

const activeTab = ref('study')

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
  card_advance_mode: 'auto',
  card_detail_seconds: 2,
  active_level: 'junior',
  setup_complete: false,
  prompts: {},
})

const selectedPrompt = ref('')
const promptEdit = ref('')
const importLevel = ref('junior')
const importMessage = ref('')
const importError = ref('')
const importRawContent = ref('')
const importPreview = ref([])
const importStats = reactive({ total: 0, duplicates: 0 })
const fileInput = ref(null)

invoke('get_config').then(data => {
  if (data.model) Object.assign(config.model, data.model)
  if (data.search) Object.assign(config.search, data.search)
  config.audio_expire_hours = data.audio_expire_hours ?? config.audio_expire_hours
  config.daily_new_words = data.daily_new_words ?? config.daily_new_words
  config.daily_review_words = data.daily_review_words ?? config.daily_review_words
  config.card_advance_mode = data.card_advance_mode || config.card_advance_mode
  config.card_detail_seconds = data.card_detail_seconds ?? config.card_detail_seconds
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

async function previewImport(event) {
  const file = event.target.files?.[0]
  clearImportState(false)
  if (!file) return

  try {
    const content = await file.text()
    const words = JSON.parse(content)
    if (!Array.isArray(words)) {
      throw new Error('词库文件必须是 JSON 数组')
    }

    const seen = new Set()
    let duplicates = 0
    const preview = []

    words.forEach((item, index) => {
      const word = String(item?.word || '').trim()
      if (!word) return
      const key = word.toLowerCase()
      if (seen.has(key)) duplicates++
      seen.add(key)
      if (preview.length < 10) {
        preview.push({
          index,
          word,
          definition: stringifyDefinition(item?.definition ?? item?.meaning ?? ''),
        })
      }
    })

    importRawContent.value = content
    importPreview.value = preview
    importStats.total = words.length
    importStats.duplicates = duplicates
    importMessage.value = ''
  } catch (e) {
    importError.value = `无法预览：${e.message || e}`
  }
}

async function confirmImport() {
  if (!importRawContent.value || importError.value) return
  try {
    const count = await invoke('import_words', {
      level: importLevel.value,
      content: importRawContent.value,
    })
    config.active_level = importLevel.value
    importMessage.value = `已导入 ${count} 个单词，并切换到该学段。`
    clearImportState(true)
  } catch (e) {
    importError.value = `导入失败：${e}`
  }
}

function stringifyDefinition(value) {
  if (typeof value === 'string') return value
  try {
    return JSON.stringify(value)
  } catch {
    return ''
  }
}

function clearImportFile() {
  if (fileInput.value) fileInput.value.value = ''
  clearImportState(false)
}

function clearImportState(keepMessage) {
  importRawContent.value = ''
  importPreview.value = []
  importStats.total = 0
  importStats.duplicates = 0
  importError.value = ''
  if (!keepMessage) importMessage.value = ''
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
    config.card_detail_seconds = Math.min(10, Math.max(1, Number(config.card_detail_seconds) || 2))
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
.panel {
  border-radius: 1rem;
  background: rgb(248 250 252);
  padding: 1rem;
}

@media (min-width: 640px) {
  .panel {
    padding: 1.25rem;
  }
}

.section-title {
  margin-bottom: 1rem;
  font-size: 1rem;
  font-weight: 700;
  color: rgb(15 23 42);
}

.label {
  margin-bottom: 0.25rem;
  display: block;
  font-size: 0.875rem;
  font-weight: 600;
  color: rgb(71 85 105);
}

.field {
  width: 100%;
  border-radius: 0.875rem;
  border: 1px solid rgb(203 213 225);
  background: white;
  padding: 0.625rem 0.875rem;
  color: rgb(15 23 42);
  outline: none;
}

.field:disabled {
  cursor: not-allowed;
  background: rgb(241 245 249);
  color: rgb(148 163 184);
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
