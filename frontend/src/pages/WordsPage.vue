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
        <h3 class="mt-1 text-2xl font-bold text-slate-900">
          {{ dailyPlanAlreadyComplete && queueItems.length === 0 ? '今日计划已完成' : `完成 ${queueItems.length} 张词卡` }}
        </h3>
      </div>
      <div class="mb-6 grid gap-3 sm:grid-cols-6">
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
          <p class="text-sm text-slate-500">模糊</p>
          <p class="mt-1 text-2xl font-bold text-amber-600">{{ sessionStats.fuzzy }}</p>
        </div>
        <div class="rounded-xl bg-slate-50 p-4">
          <p class="text-sm text-slate-500">不记得</p>
          <p class="mt-1 text-2xl font-bold text-red-600">{{ sessionStats.forgotten }}</p>
        </div>
        <div class="rounded-xl bg-slate-50 p-4">
          <p class="text-sm text-slate-500">正确率</p>
          <p class="mt-1 text-2xl font-bold text-blue-600">{{ accuracyPercent }}%</p>
        </div>
      </div>
      <div class="rounded-xl bg-blue-50 p-4 text-sm leading-6 text-slate-700">
        <p class="mb-2 font-semibold text-blue-800">明日学习计划</p>
        <p class="whitespace-pre-line">{{ nextPlan || '正在生成明日计划...' }}</p>
      </div>
      <div class="mt-6 flex flex-col gap-3 sm:flex-row">
        <button class="rounded-xl bg-blue-600 px-5 py-3 text-sm font-semibold text-white hover:bg-blue-700" @click="startExtraStudy">
          加学
        </button>
        <p class="text-sm leading-6 text-slate-500">
          今天会保留同一份明日计划；加学完成后才会更新它。
        </p>
      </div>
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
        <p class="mt-4 text-sm" :class="audioError ? 'text-red-600' : 'text-slate-500'">{{ audioStatusText }}</p>
        <div class="mt-8 grid gap-3 sm:grid-cols-3">
          <button class="rounded-2xl bg-red-50 px-5 py-4 text-base font-bold text-red-600 hover:bg-red-100" @click="answerMemory('forgotten')">
            不记得
          </button>
          <button class="rounded-2xl bg-amber-50 px-5 py-4 text-base font-bold text-amber-700 hover:bg-amber-100" @click="answerMemory('fuzzy')">
            模糊
          </button>
          <button class="rounded-2xl bg-emerald-50 px-5 py-4 text-base font-bold text-emerald-700 hover:bg-emerald-100" @click="answerMemory('remembered')">
            记得
          </button>
        </div>
      </div>

      <div v-else class="overflow-hidden rounded-2xl bg-white shadow-sm ring-1 ring-slate-100">
        <div class="h-2" :class="memoryStateMeta.barClass"></div>
        <div class="p-6 sm:p-8">
          <div class="mb-5 flex flex-wrap items-center gap-3">
            <span class="rounded-full px-3 py-1 text-xs font-semibold" :class="memoryStateMeta.badgeClass">
              {{ memoryStateMeta.label }}
            </span>
            <span class="text-sm text-slate-400">{{ detailStatusText }}</span>
          </div>

          <div class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h3 class="break-words text-4xl font-bold text-slate-900">{{ currentWord.word }}</h3>
              <div v-if="currentWord.phonetic_en || currentWord.phonetic_us" class="mt-2 flex flex-wrap gap-3 text-sm text-slate-500">
                <span v-if="currentWord.phonetic_en">英 {{ currentWord.phonetic_en }}</span>
                <span v-if="currentWord.phonetic_us">美 {{ currentWord.phonetic_us }}</span>
              </div>
              <div class="mt-4 flex flex-wrap items-center gap-3 text-sm">
                <button
                  class="rounded-xl bg-blue-50 px-4 py-2 font-semibold text-blue-700 hover:bg-blue-100 disabled:cursor-wait disabled:opacity-60"
                  :disabled="audioLoading"
                  @click="playPronunciation"
                >
                  {{ audioButtonText }}
                </button>
                <span :class="audioError ? 'text-red-600' : 'text-slate-500'">{{ audioStatusText }}</span>
                <a v-if="audioUrl" :href="audioUrl" target="_blank" class="font-medium text-slate-500 hover:text-blue-700">
                  音频源
                </a>
              </div>
            </div>
            <button class="rounded-xl bg-slate-100 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-200" @click="goNextNow">
              {{ cardAdvanceMode === 'manual' ? '继续' : '立即下一个' }}
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
            <p v-if="currentWord.example_translation" class="mt-2 text-sm leading-6 text-slate-600">{{ currentWord.example_translation }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onActivated, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'

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
const cardAdvanceMode = ref('auto')
const cardDetailSeconds = ref(2)
const queueItems = ref([])
const currentIndex = ref(0)
const loading = ref(false)
const showDetail = ref(false)
const lastMemoryState = ref('remembered')
const sessionComplete = ref(false)
const dailyPlanAlreadyComplete = ref(false)
const nextPlan = ref('')
const extraStudyMode = ref(false)
const audioUrl = ref('')
const audioLoading = ref(false)
const audioError = ref('')
let advanceTimer = null
let sessionStartedAt = 0
let hasMounted = false
let viewVersion = 0
let audioRequestId = 0
let audioPlaybackId = 0
let activeAudio = null

const sessionStats = reactive({
  newWords: 0,
  reviewWords: 0,
  remembered: 0,
  fuzzy: 0,
  forgotten: 0,
})

const currentItem = computed(() => queueItems.value[currentIndex.value] || { word: {}, kind: 'new' })
const currentWord = computed(() => currentItem.value.word || {})
const levelName = computed(() => levelOptions.find(level => level.key === currentLevel.value)?.label || currentLevel.value)
const currentIndexLabel = computed(() => {
  if (queueItems.value.length === 0) return '0 / 0'
  return `${Math.min(currentIndex.value + 1, queueItems.value.length)} / ${queueItems.value.length}`
})
const accuracyPercent = computed(() => {
  const total = sessionStats.remembered + sessionStats.fuzzy + sessionStats.forgotten
  if (!total) return 0
  return Math.round((sessionStats.remembered / total) * 100)
})
const memoryStateMeta = computed(() => {
  if (lastMemoryState.value === 'fuzzy') {
    return {
      label: '已标记模糊',
      barClass: 'bg-amber-500',
      badgeClass: 'bg-amber-100 text-amber-700',
    }
  }
  if (lastMemoryState.value === 'forgotten') {
    return {
      label: '已加入重点复习',
      barClass: 'bg-red-500',
      badgeClass: 'bg-red-100 text-red-600',
    }
  }
  return {
    label: '已标记认识',
    barClass: 'bg-emerald-500',
    badgeClass: 'bg-emerald-100 text-emerald-700',
  }
})
const detailStatusText = computed(() => {
  if (cardAdvanceMode.value === 'manual') return '详情已展开，点击继续进入下一个单词'
  return `${cardDetailSeconds.value} 秒后自动进入下一个单词`
})
const audioStatusText = computed(() => {
  if (audioLoading.value) return '正在获取发音...'
  if (audioError.value) return audioError.value
  if (audioUrl.value) return '发音已获取'
  return '等待发音接口返回'
})
const audioButtonText = computed(() => {
  if (audioLoading.value) return '获取中'
  return audioUrl.value ? '播放发音' : '获取发音'
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

onMounted(() => {
  hasMounted = true
  loadStudyQueue()
})

onActivated(() => {
  if (hasMounted && !loading.value && (sessionComplete.value || queueItems.value.length === 0)) {
    loadStudyQueue()
  }
})

onBeforeRouteLeave(() => {
  resetStudyView()
})

onBeforeUnmount(() => {
  viewVersion++
  clearAdvanceTimer()
})

async function loadStudyQueue(options = {}) {
  const version = ++viewVersion
  clearAdvanceTimer()
  extraStudyMode.value = Boolean(options.extra)
  loading.value = true
  sessionComplete.value = false
  dailyPlanAlreadyComplete.value = false
  showDetail.value = false
  resetAudioState()
  currentIndex.value = 0
  nextPlan.value = ''
  resetStats()
  sessionStartedAt = Date.now()

  try {
    const config = await invoke('get_config')
    if (version !== viewVersion) return
    appConfig.value = config
    currentLevel.value = config.active_level || 'junior'
    plannedNewWords.value = config.daily_new_words ?? 20
    plannedReviewWords.value = config.daily_review_words ?? 30
    cardAdvanceMode.value = config.card_advance_mode || 'auto'
    cardDetailSeconds.value = Math.min(10, Math.max(1, Number(config.card_detail_seconds) || 2))

    if (!extraStudyMode.value && (await isDailyPlanComplete())) {
      if (version !== viewVersion) return
      dailyPlanAlreadyComplete.value = true
      sessionComplete.value = true
      queueItems.value = []
      await loadCompletedSessionStats()
      if (version !== viewVersion) return
      nextPlan.value = await readCachedNextPlan()
      return
    }

    const queue = await invoke('get_study_queue', {
      level: currentLevel.value,
      newCount: plannedNewWords.value,
      reviewCount: plannedReviewWords.value,
    })
    if (version !== viewVersion) return
    queueItems.value = queue
    if (queueItems.value.length > 0) {
      autoPlayCurrentWordAudio()
    }
  } catch (e) {
    console.error('加载学习队列失败:', e)
    if (version === viewVersion) {
      queueItems.value = []
    }
  } finally {
    if (version === viewVersion) {
      loading.value = false
    }
  }
}

async function answerMemory(state) {
  if (showDetail.value || !currentWord.value.word) return
  clearAdvanceTimer()
  lastMemoryState.value = state
  showDetail.value = true
  if (!audioUrl.value && !audioLoading.value) {
    fetchPronunciation()
  }

  try {
    if (state === 'remembered') {
      await invoke('mark_word_learned', { word: currentWord.value })
      sessionStats.remembered++
    } else if (state === 'fuzzy') {
      await invoke('mark_word_fuzzy', { word: currentWord.value })
      sessionStats.fuzzy++
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

  if (cardAdvanceMode.value === 'auto') {
    advanceTimer = window.setTimeout(goNextNow, cardDetailSeconds.value * 1000)
  }
}

function goNextNow() {
  clearAdvanceTimer()
  if (currentIndex.value < queueItems.value.length - 1) {
    currentIndex.value++
    showDetail.value = false
    resetAudioState()
    lastMemoryState.value = 'remembered'
    window.setTimeout(autoPlayCurrentWordAudio, 0)
    return
  }
  completeSession()
}

async function completeSession() {
  const version = viewVersion
  const completingExtraStudy = extraStudyMode.value
  sessionComplete.value = true
  dailyPlanAlreadyComplete.value = !completingExtraStudy
  showDetail.value = false
  resetAudioState()
  currentIndex.value = queueItems.value.length
  await saveSessionProgress()
  if (!completingExtraStudy) {
    await markDailyPlanComplete()
  }
  if (version !== viewVersion) return
  await generateNextPlan({
    allowUpdateExisting: completingExtraStudy,
    version,
  })
}

async function saveSessionProgress() {
  try {
    await invoke('save_progress', {
      data: {
        new_words: sessionStats.newWords,
        reviewed_words: sessionStats.reviewWords,
        fuzzy_count: sessionStats.fuzzy,
        correct_count: sessionStats.remembered,
        incorrect_count: sessionStats.forgotten,
        duration_seconds: sessionDurationSeconds(),
        level: currentLevel.value,
      },
    })
  } catch (e) {
    console.warn('保存会话进度失败:', e)
  }
}

async function loadCompletedSessionStats() {
  try {
    const summary = await invoke('get_daily_study_summary', {
      level: currentLevel.value,
      date: dateKey(0),
    })
    applySessionSummary(summary)
  } catch (e) {
    console.warn('读取今日学习统计失败:', e)
  }
}

function applySessionSummary(summary) {
  sessionStats.newWords = Number(summary?.new_words || 0)
  sessionStats.reviewWords = Number(summary?.reviewed_words || 0)
  sessionStats.remembered = Number(summary?.correct_count || 0)
  sessionStats.fuzzy = Number(summary?.fuzzy_count || 0)
  sessionStats.forgotten = Number(summary?.incorrect_count || 0)
}

async function generateNextPlan({ allowUpdateExisting = false, version = viewVersion } = {}) {
  if (version !== viewVersion) {
    return
  }

  const cached = await readCachedNextPlan()
  if (cached && !allowUpdateExisting) {
    if (version === viewVersion) {
      nextPlan.value = cached
    }
    return
  }

  if (version !== viewVersion) {
    return
  }

  const forecast = await loadReviewForecast()
  const reviewTarget = Math.max(
    plannedReviewWords.value,
    Number(forecast.due_tomorrow || 0),
    sessionStats.fuzzy,
    sessionStats.forgotten,
  )
  const fallback = `明天建议学习 ${plannedNewWords.value} 个新词，复习 ${reviewTarget} 个单词。\n安排理由：按 1/3/7/15/30/60 天记忆曲线，明天预计到期 ${forecast.due_tomorrow || 0} 个，其中模糊词 ${forecast.fuzzy_words || 0} 个、困难词 ${forecast.hard_words || 0} 个。`
  const config = appConfig.value
  if (!hasModelApiKey(config)) {
    if (version === viewVersion) {
      nextPlan.value = fallback
      await saveNextPlan(fallback)
    }
    return
  }

  try {
    const tomorrow = dateKey(1)
    const content = await invoke('call_model_api', {
      config,
      messages: [
        {
          role: 'system',
          content: '你是英语词汇学习计划助手。请用中文给出简洁、可执行的学习安排。',
        },
        {
          role: 'user',
          content: `今天学段：${levelName.value}。新词 ${sessionStats.newWords} 个，复习 ${sessionStats.reviewWords} 个，认识 ${sessionStats.remembered} 个，模糊 ${sessionStats.fuzzy} 个，不认识 ${sessionStats.forgotten} 个，掌握率 ${accuracyPercent.value}%。记忆曲线预测：当前到期 ${forecast.due_now || 0} 个，明天到期 ${forecast.due_tomorrow || 0} 个，模糊词 ${forecast.fuzzy_words || 0} 个，困难词 ${forecast.hard_words || 0} 个。请生成 ${tomorrow} 的学习计划，包含新词数量、复习数量、一个重点提醒，以及“为什么这样安排”的简短理由，140 字以内。`,
        },
      ],
    })
    if (version === viewVersion) {
      nextPlan.value = content
      await saveNextPlan(content)
    }
  } catch (e) {
    console.warn('生成明日计划失败:', e)
    if (version === viewVersion) {
      nextPlan.value = fallback
      await saveNextPlan(fallback)
    }
  }
}

function hasModelApiKey(config) {
  const key = String(config?.model?.api_key || '').trim()
  return Boolean(key && !isPlaceholderApiKey(key))
}

function isPlaceholderApiKey(key) {
  return ['api_key', 'your_api_key', 'your-api-key', 'sk-xxx', 'sk-xxxx'].includes(key.toLowerCase())
}

async function saveNextPlan(content) {
  await invoke('save_next_plan', { record: {
    createdAt: localDateTimeString(),
    createdForDate: dateKey(0),
    planDate: dateKey(1),
    level: currentLevel.value,
    content,
  } })
}

async function loadReviewForecast() {
  try {
    return await invoke('get_review_forecast', { level: currentLevel.value })
  } catch (e) {
    console.warn('读取记忆曲线预测失败:', e)
    return { due_now: 0, due_tomorrow: plannedReviewWords.value, fuzzy_words: sessionStats.fuzzy, hard_words: sessionStats.forgotten }
  }
}

async function readCachedNextPlan() {
  try {
    const cached = await invoke('get_next_plan')
    if (cached?.planDate === dateKey(1) && cached?.level === currentLevel.value && cached?.content) {
      return cached.content
    }
  } catch {
    return ''
  }
  return ''
}

async function markDailyPlanComplete() {
  try {
    await invoke('mark_daily_plan_complete', {
      level: currentLevel.value,
      date: dateKey(0),
    })
  } catch (e) {
    console.warn('保存今日计划完成状态失败:', e)
  }
}

async function isDailyPlanComplete() {
  try {
    return await invoke('is_daily_plan_complete', {
      level: currentLevel.value,
      date: dateKey(0),
    })
  } catch (e) {
    console.warn('读取今日计划完成状态失败:', e)
    return false
  }
}

function startExtraStudy() {
  loadStudyQueue({ extra: true })
}

function sessionDurationSeconds() {
  if (!sessionStartedAt) return 0
  const elapsed = Math.round((Date.now() - sessionStartedAt) / 1000)
  return Math.max(1, elapsed)
}

function dateKey(offsetDays) {
  const date = new Date()
  date.setDate(date.getDate() + offsetDays)
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function resetStats() {
  sessionStats.newWords = 0
  sessionStats.reviewWords = 0
  sessionStats.remembered = 0
  sessionStats.fuzzy = 0
  sessionStats.forgotten = 0
}

function resetStudyView() {
  viewVersion++
  clearAdvanceTimer()
  loading.value = false
  sessionComplete.value = false
  dailyPlanAlreadyComplete.value = false
  showDetail.value = false
  nextPlan.value = ''
  queueItems.value = []
  currentIndex.value = 0
  extraStudyMode.value = false
  lastMemoryState.value = 'remembered'
  sessionStartedAt = 0
  resetStats()
  resetAudioState()
}

function clearAdvanceTimer() {
  if (advanceTimer) {
    window.clearTimeout(advanceTimer)
    advanceTimer = null
  }
}

async function fetchPronunciation() {
  if (!currentWord.value.word) return
  const requestId = ++audioRequestId
  const requestedWord = currentWord.value.word
  audioLoading.value = true
  audioError.value = ''
  audioUrl.value = ''

  try {
    const url = await invoke('play_word_audio', { word: requestedWord })
    if (requestId === audioRequestId && currentWord.value.word === requestedWord) {
      audioUrl.value = normalizeAudioSource(url)
    }
  } catch (e) {
    if (requestId === audioRequestId && currentWord.value.word === requestedWord) {
      audioError.value = `发音获取失败：${e}`
    }
  } finally {
    if (requestId === audioRequestId && currentWord.value.word === requestedWord) {
      audioLoading.value = false
    }
  }
}

async function playPronunciation() {
  const requestedWord = currentWord.value.word
  if (!audioUrl.value) {
    await fetchPronunciation()
  }
  if (currentWord.value.word !== requestedWord) return
  if (!audioUrl.value) return

  await playAudioSource(audioUrl.value)
}

async function autoPlayCurrentWordAudio() {
  const requestedWord = currentWord.value.word
  if (!requestedWord || showDetail.value || sessionComplete.value) return
  await fetchPronunciation()
  if (currentWord.value.word !== requestedWord) return
  if (!audioUrl.value || showDetail.value || sessionComplete.value) return
  await playAudioSource(audioUrl.value)
}

async function playAudioSource(source) {
  const playbackId = ++audioPlaybackId
  const playbackWord = currentWord.value.word
  try {
    stopActiveAudio()
    const audio = new Audio(source)
    activeAudio = audio
    await audio.play()
  } catch (e) {
    if (playbackId !== audioPlaybackId || currentWord.value.word !== playbackWord) return
    if (isInterruptedAudioPlay(e)) return
    audioError.value = `播放失败：${formatAudioError(e)}`
  }
}

function resetAudioState() {
  audioRequestId++
  audioPlaybackId++
  stopActiveAudio()
  audioUrl.value = ''
  audioLoading.value = false
  audioError.value = ''
}

function stopActiveAudio() {
  if (!activeAudio) return
  const audio = activeAudio
  activeAudio = null
  audio.pause()
  audio.src = ''
}

function isInterruptedAudioPlay(error) {
  const name = String(error?.name || '')
  const message = formatAudioError(error)
  return name === 'AbortError' || message.includes('The play() request was interrupted')
}

function formatAudioError(error) {
  return String(error?.message || error || '未知错误')
}

function normalizeAudioSource(source) {
  const value = String(source || '')
  if (/^(https?:|asset:|data:|blob:)/i.test(value)) {
    return value
  }
  return convertFileSrc(value)
}

function localDateTimeString() {
  const date = new Date()
  const pad = value => String(value).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
}
</script>
