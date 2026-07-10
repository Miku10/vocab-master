<template>
  <div class="mx-auto max-w-4xl p-4 sm:p-6">
    <div class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="text-sm font-medium text-blue-600">{{ levelName }} · 今日计划</p>
        <h2 class="text-2xl font-bold text-slate-900">记忆词卡</h2>
      </div>
      <div class="flex flex-wrap gap-2 text-sm text-slate-600">
        <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">新词 {{ plannedNewWords }}</span>
        <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">复习 {{ plannedReviewWords }}</span>
        <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">{{ currentIndexLabel }}</span>
      </div>
    </div>

    <div v-if="loading" class="rounded-2xl bg-white py-20 text-center text-slate-500 shadow-sm ring-1 ring-slate-100">
      <div class="mb-3 inline-block animate-spin text-3xl">⟳</div>
      <p>正在生成今日学习队列...</p>
    </div>

    <div v-else-if="sessionComplete" class="rounded-2xl bg-white p-6 shadow-sm ring-1 ring-slate-100 sm:p-8">
      <div class="mb-6">
        <p class="text-sm font-medium text-emerald-600">今日学习完成</p>
        <h3 class="mt-1 text-2xl font-bold text-slate-900">完成 {{ queueItems.length }} 张词卡</h3>
      </div>
      <div class="mb-6 grid gap-3 sm:grid-cols-4">
        <div class="rounded-xl bg-slate-50 p-4">
          <p class="text-sm text-slate-500">新词</p>
          <p class="mt-1 text-2xl font-bold text-slate-900">{{ sessionStats.newWords }}</p>
        </div>
        <div class="rounded-xl bg-slate-50 p-4">
          <p class="text-sm text-slate-500">复习</p>
          <p class="mt-1 text-2xl font-bold text-slate-900">{{ sessionStats.reviewWords }}</p>
        </div>
        <div class="rounded-xl bg-slate-50 p-4">
          <p class="text-sm text-slate-500">记得</p>
          <p class="mt-1 text-2xl font-bold text-emerald-600">{{ sessionStats.remembered }}</p>
        </div>
        <div class="rounded-xl bg-slate-50 p-4">
          <p class="text-sm text-slate-500">不记得</p>
          <p class="mt-1 text-2xl font-bold text-red-600">{{ sessionStats.forgotten }}</p>
        </div>
      </div>
      <div class="rounded-xl bg-blue-50 p-4 text-sm leading-6 text-slate-700">
        <p class="mb-2 font-semibold text-blue-800">明日学习计划</p>
        <p class="whitespace-pre-line">{{ nextPlan || '正在生成明日计划...' }}</p>
      </div>
      <button class="mt-6 rounded-xl bg-blue-600 px-5 py-3 text-sm font-semibold text-white hover:bg-blue-700" @click="loadStudyQueue">
        重新生成今日队列
      </button>
    </div>

    <div v-else-if="queueItems.length === 0" class="rounded-2xl bg-white py-20 text-center shadow-sm ring-1 ring-slate-100">
      <div class="mb-4 text-5xl">📚</div>
      <p class="text-lg font-semibold text-slate-800">今日没有可学习的单词</p>
      <p class="mt-2 text-sm text-slate-500">可以在设置里调整学段、每日新词数，或导入自己的词库。</p>
    </div>

    <div v-else class="space-y-5">
      <div class="h-2 overflow-hidden rounded-full bg-slate-200">
        <div class="h-full rounded-full bg-blue-600 transition-all" :style="{ width: progressPercent + '%' }"></div>
      </div>

      <div v-if="!showDetail" class="rounded-2xl bg-white p-6 text-center shadow-sm ring-1 ring-slate-100 sm:p-10">
        <div class="mb-5 flex justify-center">
          <span class="rounded-full px-3 py-1 text-xs font-semibold" :class="currentItem.kind === 'review' ? 'bg-amber-100 text-amber-700' : 'bg-blue-100 text-blue-700'">
            {{ currentItem.kind === 'review' ? '复习词' : '新词' }}
          </span>
        </div>
        <p class="mb-3 text-sm text-slate-400">看到这个单词时，你能想起意思吗？</p>
        <h3 class="break-words text-5xl font-bold tracking-normal text-slate-900 sm:text-6xl">{{ currentWord.word }}</h3>
        <div class="mt-8 grid gap-3 sm:grid-cols-2">
          <button class="rounded-2xl bg-red-50 px-5 py-4 text-base font-bold text-red-600 hover:bg-red-100" @click="answerMemory(false)">
            不记得
          </button>
          <button class="rounded-2xl bg-emerald-50 px-5 py-4 text-base font-bold text-emerald-700 hover:bg-emerald-100" @click="answerMemory(true)">
            记得
          </button>
        </div>
      </div>

      <div v-else class="overflow-hidden rounded-2xl bg-white shadow-sm ring-1 ring-slate-100">
        <div class="h-2" :class="lastRemembered ? 'bg-emerald-500' : 'bg-red-500'"></div>
        <div class="p-6 sm:p-8">
          <div class="mb-5 flex flex-wrap items-center gap-3">
            <span class="rounded-full px-3 py-1 text-xs font-semibold" :class="lastRemembered ? 'bg-emerald-100 text-emerald-700' : 'bg-red-100 text-red-600'">
              {{ lastRemembered ? '已标记记得' : '已加入复习' }}
            </span>
            <span class="text-sm text-slate-400">即将自动进入下一个单词</span>
          </div>

          <div class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h3 class="break-words text-4xl font-bold text-slate-900">{{ currentWord.word }}</h3>
              <div v-if="currentWord.phonetic_en || currentWord.phonetic_us" class="mt-2 flex flex-wrap gap-3 text-sm text-slate-500">
                <span v-if="currentWord.phonetic_en">英 {{ currentWord.phonetic_en }}</span>
                <span v-if="currentWord.phonetic_us">美 {{ currentWord.phonetic_us }}</span>
              </div>
            </div>
            <button class="rounded-xl bg-slate-100 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-200" @click="goNextNow">
              立即下一个
            </button>
          </div>

          <div class="mb-6 space-y-2">
            <div v-for="(def, index) in parsedDefinitions" :key="index" class="flex gap-3 rounded-xl bg-slate-50 p-3">
              <span class="shrink-0 rounded-lg bg-blue-100 px-2 py-1 text-xs font-semibold text-blue-700">{{ def.pos || '释义' }}</span>
              <span class="text-sm leading-6 text-slate-700">{{ def.meaning }}</span>
            </div>
          </div>

          <div v-if="currentWord.example" class="rounded-xl border border-amber-100 bg-amber-50 p-4">
            <p class="text-sm font-semibold text-amber-800">例句</p>
            <p class="mt-2 text-sm leading-6 text-slate-700">{{ currentWord.example }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const levelOptions = [
  { key: 'junior', label: '初中' },
  { key: 'high', label: '高中' },
  { key: 'cet4', label: '四级' },
  { key: 'cet6', label: '六级' },
  { key: 'primary', label: '小学' },
]

const appConfig = ref(null)
const currentLevel = ref('junior')
const plannedNewWords = ref(20)
const plannedReviewWords = ref(30)
const queueItems = ref([])
const currentIndex = ref(0)
const loading = ref(false)
const showDetail = ref(false)
const lastRemembered = ref(false)
const sessionComplete = ref(false)
const nextPlan = ref('')
let advanceTimer = null

const sessionStats = reactive({
  newWords: 0,
  reviewWords: 0,
  remembered: 0,
  forgotten: 0,
})

const currentItem = computed(() => queueItems.value[currentIndex.value] || { word: {}, kind: 'new' })
const currentWord = computed(() => currentItem.value.word || {})
const levelName = computed(() => levelOptions.find(level => level.key === currentLevel.value)?.label || currentLevel.value)
const currentIndexLabel = computed(() => {
  if (queueItems.value.length === 0) return '0 / 0'
  return `${Math.min(currentIndex.value + 1, queueItems.value.length)} / ${queueItems.value.length}`
})
const progressPercent = computed(() => {
  if (queueItems.value.length === 0) return 0
  return Math.round((currentIndex.value / queueItems.value.length) * 100)
})

const parsedDefinitions = computed(() => {
  const definition = currentWord.value.definition || ''
  try {
    const defs = JSON.parse(definition)
    if (Array.isArray(defs)) {
      return defs.map(item => ({
        pos: item.pos || '',
        meaning: item.meaning || String(item),
      }))
    }
  } catch {
    // Plain text definitions are supported for imported word banks.
  }
  return [{ pos: '', meaning: definition || '暂无释义' }]
})

onMounted(loadStudyQueue)

onBeforeUnmount(() => {
  clearAdvanceTimer()
})

async function loadStudyQueue() {
  clearAdvanceTimer()
  loading.value = true
  sessionComplete.value = false
  showDetail.value = false
  currentIndex.value = 0
  nextPlan.value = ''
  resetStats()

  try {
    const config = await invoke('get_config')
    appConfig.value = config
    currentLevel.value = config.active_level || 'junior'
    plannedNewWords.value = config.daily_new_words ?? 20
    plannedReviewWords.value = config.daily_review_words ?? 30

    queueItems.value = await invoke('get_study_queue', {
      level: currentLevel.value,
      newCount: plannedNewWords.value,
      reviewCount: plannedReviewWords.value,
    })
  } catch (e) {
    console.error('加载学习队列失败:', e)
    queueItems.value = []
  } finally {
    loading.value = false
  }
}

async function answerMemory(remembered) {
  if (showDetail.value || !currentWord.value.word) return
  clearAdvanceTimer()
  lastRemembered.value = remembered
  showDetail.value = true

  try {
    if (remembered) {
      await invoke('mark_word_learned', { word: currentWord.value })
      sessionStats.remembered++
    } else {
      await invoke('mark_word_hard', { word: currentWord.value })
      sessionStats.forgotten++
    }
    if (currentItem.value.kind === 'review') {
      sessionStats.reviewWords++
    } else {
      sessionStats.newWords++
    }
  } catch (e) {
    console.warn('保存学习结果失败:', e)
  }

  advanceTimer = window.setTimeout(goNextNow, 1800)
}

function goNextNow() {
  clearAdvanceTimer()
  if (currentIndex.value < queueItems.value.length - 1) {
    currentIndex.value++
    showDetail.value = false
    lastRemembered.value = false
    return
  }
  completeSession()
}

async function completeSession() {
  sessionComplete.value = true
  showDetail.value = false
  currentIndex.value = queueItems.value.length
  await saveSessionProgress()
  await generateNextPlan()
}

async function saveSessionProgress() {
  try {
    await invoke('save_progress', {
      data: {
        new_words: sessionStats.newWords,
        reviewed_words: sessionStats.reviewWords,
        correct_count: sessionStats.remembered,
        incorrect_count: sessionStats.forgotten,
      },
    })
  } catch (e) {
    console.warn('保存会话进度失败:', e)
  }
}

async function generateNextPlan() {
  const fallback = `明天建议学习 ${plannedNewWords.value} 个新词，复习 ${plannedReviewWords.value} 个单词。优先复习今天标记“不记得”的单词。`
  const config = appConfig.value
  if (!config?.model?.api_key) {
    nextPlan.value = fallback
    saveNextPlan(fallback)
    return
  }

  try {
    const tomorrow = new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString().slice(0, 10)
    const content = await invoke('call_model_api', {
      config,
      messages: [
        {
          role: 'system',
          content: '你是英语词汇学习计划助手。请用中文给出简洁、可执行的学习安排。',
        },
        {
          role: 'user',
          content: `今天学段：${levelName.value}。新词 ${sessionStats.newWords} 个，复习 ${sessionStats.reviewWords} 个，记得 ${sessionStats.remembered} 个，不记得 ${sessionStats.forgotten} 个。请生成 ${tomorrow} 的学习计划，包含新词数量、复习数量和一个重点提醒，100 字以内。`,
        },
      ],
    })
    nextPlan.value = content
    saveNextPlan(content)
  } catch (e) {
    console.warn('生成明日计划失败:', e)
    nextPlan.value = fallback
    saveNextPlan(fallback)
  }
}

function saveNextPlan(content) {
  localStorage.setItem('vocab-master-next-plan', JSON.stringify({
    createdAt: new Date().toISOString(),
    level: currentLevel.value,
    content,
  }))
}

function resetStats() {
  sessionStats.newWords = 0
  sessionStats.reviewWords = 0
  sessionStats.remembered = 0
  sessionStats.forgotten = 0
}

function clearAdvanceTimer() {
  if (advanceTimer) {
    window.clearTimeout(advanceTimer)
    advanceTimer = null
  }
}
</script>
