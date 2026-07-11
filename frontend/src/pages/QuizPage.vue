<template>
  <div class="mx-auto max-w-4xl p-4 sm:p-6">
    <div class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="text-sm font-medium text-blue-600">{{ levelName }} · 模拟测试</p>
        <h2 class="text-2xl font-bold text-slate-900">考试题型训练</h2>
      </div>
      <div class="grid gap-2 sm:grid-cols-[minmax(0,1fr)_9rem]">
        <select v-model="selectedExamType" class="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm text-slate-700">
          <option v-for="type in examTypes" :key="type.key" :value="type.key">{{ type.label }}</option>
        </select>
        <label class="relative block">
          <span class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-xs font-semibold text-slate-400">题数</span>
          <input
            v-model.number="quizSize"
            type="number"
            min="1"
            max="50"
            class="w-full rounded-xl border border-slate-200 bg-white py-2 pl-12 pr-3 text-sm text-slate-700"
          />
        </label>
      </div>
    </div>

    <div v-if="loading" class="rounded-2xl bg-white py-20 text-center text-slate-500 shadow-sm ring-1 ring-slate-100">
      <div class="mb-3 inline-block animate-spin text-3xl">⟳</div>
      <p>正在载入测试词库...</p>
    </div>

    <div v-else-if="words.length === 0" class="rounded-2xl bg-white py-20 text-center shadow-sm ring-1 ring-slate-100">
      <div class="mb-4 text-5xl">📝</div>
      <p class="text-lg font-semibold text-slate-800">当前学段暂无测试词库</p>
      <p class="mt-2 text-sm text-slate-500">可以在设置里切换学段或导入词库。</p>
    </div>

    <div v-else-if="!dailyPlanComplete" class="rounded-2xl bg-white py-20 text-center shadow-sm ring-1 ring-slate-100">
      <div class="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-blue-50 text-lg font-bold text-blue-700">锁</div>
      <p class="text-lg font-semibold text-slate-800">请先完成今日词卡学习</p>
      <p class="mt-2 text-sm text-slate-500">当天计划完成后，才会开放基于今日单词的测验。</p>
    </div>

    <div v-else-if="todayWords.length === 0" class="rounded-2xl bg-white py-20 text-center shadow-sm ring-1 ring-slate-100">
      <div class="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-slate-50 text-lg font-bold text-slate-600">空</div>
      <p class="text-lg font-semibold text-slate-800">今日暂无可测单词</p>
      <p class="mt-2 text-sm text-slate-500">测验只会基于当天完成的词卡单词生成，请先完成今日学习队列。</p>
    </div>

    <div v-else-if="!started" class="rounded-2xl bg-white p-6 shadow-sm ring-1 ring-slate-100 sm:p-8">
      <div class="mb-6">
        <p class="text-sm font-medium text-slate-500">优先用 AI 按考试高频题型生成；当天单词少于题数时会围绕已学词生成多题。</p>
        <h3 class="mt-1 text-2xl font-bold text-slate-900">准备开始 {{ availableQuizCount }} 题测试</h3>
      </div>
      <button class="rounded-xl bg-blue-600 px-5 py-3 text-sm font-semibold text-white hover:bg-blue-700" @click="startQuiz">
        开始测试
      </button>
    </div>

    <div v-else-if="generating" class="rounded-2xl bg-white py-20 text-center text-slate-500 shadow-sm ring-1 ring-slate-100">
      <div class="mb-3 inline-block animate-spin text-3xl">⟳</div>
      <p>AI 正在生成考试题型...</p>
    </div>

    <div v-else-if="grading" class="rounded-2xl bg-white py-20 text-center text-slate-500 shadow-sm ring-1 ring-slate-100">
      <div class="mb-3 inline-block animate-spin text-3xl">⟳</div>
      <p>正在批阅并生成解析...</p>
    </div>

    <div v-else-if="result" class="space-y-5">
      <div class="rounded-2xl bg-white p-6 shadow-sm ring-1 ring-slate-100 sm:p-8">
        <div class="grid gap-4 sm:grid-cols-[160px_1fr] sm:items-center">
          <div class="rounded-2xl bg-blue-50 p-5 text-center">
            <p class="text-sm font-semibold text-blue-700">得分</p>
            <p class="mt-2 text-5xl font-bold text-blue-700">{{ result.score }}</p>
          </div>
          <div>
            <h3 class="text-xl font-bold text-slate-900">批阅完成</h3>
            <p class="mt-2 whitespace-pre-line text-sm leading-6 text-slate-600">{{ result.summary }}</p>
            <div v-if="result.advice.length" class="mt-4 flex flex-wrap gap-2">
              <span v-for="item in result.advice" :key="item" class="rounded-full bg-slate-100 px-3 py-1.5 text-sm text-slate-700">
                {{ item }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <div class="space-y-3">
        <div v-for="(item, index) in result.items" :key="`${item.word}-${index}`" class="rounded-2xl bg-white p-5 shadow-sm ring-1 ring-slate-100">
          <div class="mb-3 flex flex-wrap items-center justify-between gap-3">
            <div>
              <p class="text-lg font-bold text-slate-900">{{ item.word }}</p>
              <p class="text-sm text-slate-500">你的答案：{{ item.answer || '未作答' }}</p>
            </div>
            <span class="rounded-full px-3 py-1 text-sm font-semibold" :class="item.is_correct ? 'bg-emerald-100 text-emerald-700' : 'bg-red-100 text-red-600'">
              {{ item.score }} 分
            </span>
          </div>
          <p class="text-sm leading-6 text-slate-700">{{ item.analysis }}</p>
          <p class="mt-2 text-sm leading-6 text-blue-700">{{ item.suggestion }}</p>
        </div>
      </div>

      <button class="rounded-xl bg-blue-600 px-5 py-3 text-sm font-semibold text-white hover:bg-blue-700" @click="resetQuiz">
        再测一次
      </button>
    </div>

    <form v-else class="space-y-4" @submit.prevent="submitQuiz">
      <div v-for="(question, index) in questions" :key="`${question.word.word}-${index}`" class="rounded-2xl bg-white p-5 shadow-sm ring-1 ring-slate-100">
        <div class="mb-4 flex items-center justify-between gap-3">
          <span class="rounded-full bg-slate-100 px-3 py-1 text-sm font-semibold text-slate-600">第 {{ index + 1 }} 题 · {{ question.type }}</span>
          <span class="text-sm text-slate-400">#{{ question.word.frequency }}</span>
        </div>
        <div class="block">
          <span class="mb-2 block text-lg font-bold leading-7 text-slate-900">{{ question.stem }}</span>
          <div v-if="question.options.length" class="mt-4 grid gap-2">
            <label
              v-for="option in question.options"
              :key="option"
              class="flex cursor-pointer items-start gap-3 rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-700 hover:bg-blue-50"
            >
              <input v-model="answers[index]" type="radio" :name="`q-${index}`" :value="option" class="mt-1" />
              <span>{{ option }}</span>
            </label>
          </div>
          <input
            v-else
            v-model="answers[index]"
            class="mt-4 w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-slate-900 outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-100"
            placeholder="输入答案"
          />
          <input
            v-if="question.options.length"
            v-model="answers[index]"
            class="w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-slate-900 outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-100"
            placeholder="也可以手动输入答案"
          />
        </div>
      </div>

      <div class="sticky bottom-4 rounded-2xl border border-slate-200 bg-white/95 p-3 shadow-lg backdrop-blur">
        <button class="w-full rounded-xl bg-blue-600 px-5 py-3 text-sm font-semibold text-white hover:bg-blue-700" type="submit">
          提交并批阅
        </button>
      </div>
    </form>
  </div>
</template>

<script>
const quizQuestionCache = new Map()
const quizQuestionRequests = new Map()
const quizGradingCache = new Map()
const quizGradingRequests = new Map()
const quizPageState = {
  selectedExamType: 'mixed',
  quizSize: 10,
  activeGenerationKey: '',
  activeGradingKey: '',
}
</script>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const MODEL_REQUEST_TIMEOUT_MS = 25000
const MODEL_REQUEST_RETRIES = 1

const levelOptions = [
  { key: 'junior', label: '初中' },
  { key: 'high', label: '高中' },
  { key: 'cet4', label: '四级' },
  { key: 'cet6', label: '六级' },
  { key: 'primary', label: '小学' },
]

const examTypes = [
  { key: 'mixed', label: '综合题型' },
  { key: 'single_choice', label: '单项选择' },
  { key: 'cloze', label: '完形填空' },
  { key: 'context_blank', label: '语境填空' },
  { key: 'word_discrimination', label: '词义辨析' },
  { key: 'grammar_collocation', label: '语法搭配' },
  { key: 'reading_inference', label: '阅读词义推断' },
]

const config = ref(null)
const level = ref('junior')
const words = ref([])
const todayWords = ref([])
const dailyPlanComplete = ref(false)
const questions = ref([])
const answers = ref([])
const quizSize = ref(quizPageState.quizSize)
const selectedExamType = ref(quizPageState.selectedExamType)
const loading = ref(false)
const started = ref(false)
const generating = ref(false)
const grading = ref(false)
const result = ref(null)
let quizViewVersion = 0

const levelName = computed(() => levelOptions.find(item => item.key === level.value)?.label || level.value)
const availableQuizCount = computed(() => normalizedQuizSize())

onMounted(() => {
  loadWords()
  window.addEventListener('app-config-updated', handleAppConfigUpdated)
})
onBeforeUnmount(() => {
  quizViewVersion++
  window.removeEventListener('app-config-updated', handleAppConfigUpdated)
})

watch([selectedExamType, quizSize], () => {
  quizSize.value = normalizedQuizSize()
  quizPageState.selectedExamType = selectedExamType.value
  quizPageState.quizSize = quizSize.value
  if (started.value || generating.value || result.value) {
    resetQuiz()
  }
})

async function loadWords() {
  loading.value = true
  try {
    await refreshQuizSource()
  } catch (e) {
    console.error('载入测试词库失败:', e)
    words.value = []
    todayWords.value = []
    dailyPlanComplete.value = false
  } finally {
    loading.value = false
  }
  restoreActiveQuizGeneration().catch(e => console.warn('恢复题目生成状态失败:', e))
  restoreActiveQuizGrading().catch(e => console.warn('恢复批阅状态失败:', e))
}

async function refreshQuizSource() {
  const appConfig = await invoke('get_config')
  config.value = appConfig
  level.value = appConfig.active_level || 'junior'
  const data = await invoke('load_words', { level: level.value })
  words.value = [...data].sort((a, b) => (a.frequency || 999999) - (b.frequency || 999999))
  dailyPlanComplete.value = await isDailyPlanComplete(level.value)
  todayWords.value = await loadTodayStudyWords(level.value, words.value)
}

function handleAppConfigUpdated(event) {
  const nextConfig = event.detail
  if (!nextConfig) return
  const nextLevel = nextConfig.active_level || 'junior'
  const levelChanged = nextLevel !== level.value
  config.value = cloneJson(nextConfig)
  if (levelChanged && !grading.value) {
    resetQuiz()
    loadWords()
    return
  }
  level.value = nextLevel
}

async function startQuiz() {
  const version = ++quizViewVersion
  result.value = null
  started.value = true
  generating.value = true
  let seedWords = []

  try {
    await refreshQuizSource()
    if (!dailyPlanComplete.value) {
      started.value = false
      questions.value = []
      answers.value = []
      return
    }
    seedWords = selectQuizSeedWords()
    const context = createQuizGenerationContext(seedWords)
    quizPageState.activeGenerationKey = context.key
    const generatedQuestions = await getGeneratedQuestions(context, seedWords)
    if (version !== quizViewVersion) return
    questions.value = generatedQuestions
  } catch (e) {
    console.warn('AI 生成题目失败，使用本地题目:', e)
    if (version === quizViewVersion) {
      questions.value = buildLocalQuestions(seedWords)
    }
  } finally {
    if (version === quizViewVersion) {
      answers.value = questions.value.map(() => '')
      generating.value = false
    }
  }
}

async function submitQuiz() {
  const context = createGradingContext()
  const version = ++quizViewVersion
  quizPageState.activeGradingKey = context.key
  grading.value = true
  try {
    const finalResult = await getGradingResult(context)
    if (version !== quizViewVersion) return
    result.value = finalResult
  } catch (e) {
    console.warn('AI 批阅失败，使用本地评分:', e)
    const fallback = buildLocalResultFromContext(context)
    await saveQuizRecordFromContext(context, fallback)
    if (version !== quizViewVersion) return
    result.value = fallback
  } finally {
    if (version === quizViewVersion) {
      grading.value = false
    }
  }
}

function buildLocalResult() {
  const items = questions.value.map((question, index) => {
    const { answer, isCorrect } = gradeQuestion(question, index)
    return {
      word: question.word.word,
      answer,
      is_correct: isCorrect,
      score: questionScore(isCorrect),
      analysis: isCorrect
        ? `答案命中了参考答案：${question.correctAnswer || question.reference}`
        : `参考答案：${question.correctAnswer || question.reference}。${question.explanation || ''}`,
      suggestion: isCorrect ? '保持复习节奏。' : '把这个词加入明日重点复习。',
    }
  })
  const correctCount = items.filter(item => item.is_correct).length
  const score = questions.value.length ? Math.round((correctCount / questions.value.length) * 100) : 0

  return {
    score,
    summary: hasModelApiKey()
      ? `本地兜底评分：答对 ${correctCount} / ${questions.value.length}。`
      : `未配置模型，已使用本地关键词匹配评分：答对 ${correctCount} / ${questions.value.length}。`,
    advice: score >= 80 ? ['继续增加新词量', '保持每日复习'] : ['降低新词量', '优先复习错词'],
    items,
  }
}

async function restoreActiveQuizGrading() {
  const key = quizPageState.activeGradingKey
  if (!key || result.value) return
  const cached = quizGradingCache.get(key)
  if (cached) {
    started.value = true
    grading.value = false
    result.value = cloneJson(cached)
    return
  }

  const request = quizGradingRequests.get(key)
  if (!request) return
  const version = ++quizViewVersion
  started.value = true
  grading.value = true
  try {
    const finalResult = await request
    if (version !== quizViewVersion) return
    result.value = cloneJson(finalResult)
  } finally {
    if (version === quizViewVersion) {
      grading.value = false
    }
  }
}

async function getGradingResult(context) {
  const cached = quizGradingCache.get(context.key)
  if (cached) return cloneJson(cached)

  if (!quizGradingRequests.has(context.key)) {
    const request = buildGradingResult(context)
      .then(finalResult => {
        quizGradingCache.set(context.key, cloneJson(finalResult))
        return finalResult
      })
      .finally(() => {
        quizGradingRequests.delete(context.key)
      })
    quizGradingRequests.set(context.key, request)
  }

  const finalResult = await quizGradingRequests.get(context.key)
  return cloneJson(finalResult)
}

async function buildGradingResult(context) {
  const fallback = buildLocalResultFromContext(context)
  try {
    if (!hasModelApiKey(context.config)) {
      await saveQuizRecordFromContext(context, fallback)
      return fallback
    }

    const content = await callModelApiWithRetry({
      config: context.config,
      messages: [
        {
          role: 'system',
          content: '你是严谨的英语词汇测试阅卷老师。只返回 JSON，不要返回 Markdown。',
        },
        {
          role: 'user',
          content: JSON.stringify({
            instruction: '请根据参考释义批阅用户答案，给出总分、逐题解析、分数和学习建议。总分 0-100。',
            schema: {
              score: 88,
              summary: '整体表现总结',
              advice: ['建议1', '建议2'],
              items: [
                {
                  word: 'example',
                  answer: '用户答案',
                  is_correct: true,
                  score: 10,
                  analysis: '解析',
                  suggestion: '建议',
                },
              ],
            },
            questions: context.questions.map((question, index) => ({
              word: question.word.word,
              type: question.type,
              stem: question.stem,
              options: question.options,
              correct_answer: question.correctAnswer,
              reference: question.reference,
              answer: context.answers[index] || '',
            })),
          }),
        },
      ],
    })

    const parsed = extractJson(content)
    const finalResult = normalizeAiResultFromContext(parsed, fallback, context)
    await saveQuizRecordFromContext(context, finalResult)
    return finalResult
  } catch (e) {
    console.warn('AI 批阅失败，使用本地评分:', e)
    await saveQuizRecordFromContext(context, fallback)
    return fallback
  }
}

function createGradingContext() {
  const contextQuestions = cloneQuestions(questions.value)
  const contextAnswers = [...answers.value]
  const answerSignature = contextAnswers.map(answer => String(answer || '').trim()).join('|')
  const questionSignature = contextQuestions.map(question => `${question.word?.id ?? ''}:${question.word?.word ?? ''}:${question.correctAnswer ?? ''}`).join('|')
  const key = [dateKey(), level.value, selectedExamType.value, contextQuestions.length, questionSignature, answerSignature].join('::')
  return {
    key,
    date: dateKey(),
    level: level.value,
    levelName: levelName.value,
    examType: selectedExamType.value,
    examTypeName: examTypes.find(type => type.key === selectedExamType.value)?.label || selectedExamType.value,
    config: cloneJson(config.value),
    questions: contextQuestions,
    answers: contextAnswers,
  }
}

function buildLocalResultFromContext(context) {
  const items = context.questions.map((question, index) => {
    const answer = context.answers[index] || ''
    const isCorrect = answerMatches(answer, question.correctAnswer || question.reference)
    return {
      word: question.word.word,
      answer,
      is_correct: isCorrect,
      score: questionScoreFrom(isCorrect, context.questions.length),
      analysis: isCorrect
        ? `答案命中了参考答案：${question.correctAnswer || question.reference}`
        : `参考答案：${question.correctAnswer || question.reference}。${question.explanation || ''}`,
      suggestion: isCorrect ? '保持复习节奏。' : '把这个词加入明日重点复习。',
    }
  })
  const correctCount = items.filter(item => item.is_correct).length
  const score = context.questions.length ? Math.round((correctCount / context.questions.length) * 100) : 0

  return {
    score,
    summary: hasModelApiKey(context.config)
      ? `本地兜底评分：答对 ${correctCount} / ${context.questions.length}。`
      : `未配置模型，已使用本地关键词匹配评分：答对 ${correctCount} / ${context.questions.length}。`,
    advice: score >= 80 ? ['继续增加新词量', '保持每日复习'] : ['降低新词量', '优先复习错词'],
    items,
  }
}

function normalizeAiResultFromContext(parsed, fallback, context) {
  if (!parsed || typeof parsed !== 'object') return fallback
  const aiItems = Array.isArray(parsed.items) ? parsed.items : []
  const items = context.questions.map((question, index) => {
    const aiItem = aiItems[index] || {}
    const answer = context.answers[index] || ''
    const isCorrect = answerMatches(answer, question.correctAnswer || question.reference)
    return {
      word: String(aiItem.word || question.word.word || ''),
      answer,
      is_correct: isCorrect,
      score: questionScoreFrom(isCorrect, context.questions.length),
      analysis: String(aiItem.analysis || fallback.items[index]?.analysis || ''),
      suggestion: String(aiItem.suggestion || fallback.items[index]?.suggestion || ''),
    }
  })
  const correctCount = items.filter(item => item.is_correct).length
  const score = context.questions.length ? Math.round((correctCount / context.questions.length) * 100) : 0
  const advice = Array.isArray(parsed.advice)
    ? parsed.advice.map(String)
    : parsed.advice
      ? [String(parsed.advice)]
      : fallback.advice

  return {
    score,
    summary: `按参考答案自动核算：答对 ${correctCount} / ${context.questions.length}。AI 解析仅用于说明和建议。`,
    advice,
    items,
  }
}

function questionScoreFrom(isCorrect, questionCount) {
  if (!isCorrect || !questionCount) return 0
  return Math.round(100 / questionCount)
}

function hasModelApiKey(configValue = config.value) {
  const key = String(configValue?.model?.api_key || '').trim()
  return Boolean(key && !isPlaceholderApiKey(key))
}

async function callModelApiWithRetry(payload, options = {}) {
  const timeoutMs = options.timeoutMs ?? modelRequestTimeoutMs(payload?.config)
  const retries = options.retries ?? modelRetryCount(payload?.config)
  let lastError = null
  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      return await withTimeout(invoke('call_model_api', payload), timeoutMs, '模型请求超时')
    } catch (e) {
      lastError = e
      if (attempt < retries) {
        await delay(600)
      }
    }
  }
  throw lastError
}

function withTimeout(promise, timeoutMs, message) {
  let timer = null
  const timeout = new Promise((_, reject) => {
    timer = window.setTimeout(() => reject(new Error(message)), timeoutMs)
  })
  return Promise.race([promise, timeout]).finally(() => {
    if (timer) window.clearTimeout(timer)
  })
}

function delay(ms) {
  return new Promise(resolve => window.setTimeout(resolve, ms))
}

function modelRequestTimeoutMs(configValue) {
  const seconds = Number(configValue?.model?.request_timeout_seconds ?? MODEL_REQUEST_TIMEOUT_MS / 1000)
  return Math.min(120, Math.max(5, seconds || 25)) * 1000
}

function modelRetryCount(configValue) {
  const retries = Number(configValue?.model?.retry_count ?? MODEL_REQUEST_RETRIES)
  return Math.min(5, Math.max(0, Number.isFinite(retries) ? Math.floor(retries) : MODEL_REQUEST_RETRIES))
}

function isPlaceholderApiKey(key) {
  return ['api_key', 'your_api_key', 'your-api-key', 'sk-xxx', 'sk-xxxx'].includes(key.toLowerCase())
}

function selectQuizSeedWords() {
  const source = todayWords.value
  if (!source.length) return []
  const count = normalizedQuizSize()
  return Array.from({ length: count }, (_, index) => source[index % source.length])
}

function normalizedQuizSize() {
  const count = Number(quizSize.value)
  return Math.min(50, Math.max(1, Number.isFinite(count) ? Math.floor(count) : 10))
}

async function restoreActiveQuizGeneration() {
  if (started.value || result.value || loading.value || !dailyPlanComplete.value || todayWords.value.length === 0) return
  const seedWords = selectQuizSeedWords()
  const context = createQuizGenerationContext(seedWords)
  pruneQuizQuestionCache(context.date)
  if (context.key !== quizPageState.activeGenerationKey) return

  const cached = readCachedGeneratedQuestions(context)
  if (cached) {
    started.value = true
    generating.value = false
    questions.value = cached
    answers.value = cached.map(() => '')
    return
  }

  const request = quizQuestionRequests.get(context.key)
  if (!request) return
  const version = ++quizViewVersion
  started.value = true
  generating.value = true
  result.value = null
  try {
    const generatedQuestions = await request
    if (version !== quizViewVersion) return
    questions.value = cloneQuestions(generatedQuestions)
    answers.value = questions.value.map(() => '')
  } finally {
    if (version === quizViewVersion) {
      generating.value = false
    }
  }
}

async function getGeneratedQuestions(context, seedWords) {
  pruneQuizQuestionCache(context.date)
  const cached = readCachedGeneratedQuestions(context)
  if (cached) return cached

  if (!quizQuestionRequests.has(context.key)) {
    const request = buildGeneratedQuestions(context, seedWords)
      .then(generatedQuestions => {
        const cloned = cloneQuestions(generatedQuestions)
        quizQuestionCache.set(context.key, {
          date: context.date,
          questions: cloned,
        })
        return cloned
      })
      .finally(() => {
        quizQuestionRequests.delete(context.key)
      })
    quizQuestionRequests.set(context.key, request)
  }

  const generatedQuestions = await quizQuestionRequests.get(context.key)
  return cloneQuestions(generatedQuestions)
}

async function buildGeneratedQuestions(context, seedWords) {
  try {
    return hasModelApiKey(context.config)
      ? await generateAiQuestions(seedWords, context)
      : buildLocalQuestions(seedWords)
  } catch (e) {
    console.warn('AI 生成题目失败，使用本地题目:', e)
    return buildLocalQuestions(seedWords)
  }
}

function createQuizGenerationContext(seedWords) {
  const date = dateKey()
  const examTypeLabel = examTypes.find(type => type.key === selectedExamType.value)?.label || '综合题型'
  const wordSignature = seedWords
    .map(word => `${word.id ?? ''}:${String(word.word || '').toLowerCase()}`)
    .join('|')
  return {
    key: [date, level.value, selectedExamType.value, seedWords.length, wordSignature].join('::'),
    date,
    level: level.value,
    levelName: levelName.value,
    examType: selectedExamType.value,
    examTypeLabel,
    quizSize: seedWords.length,
    wordSignature,
    config: cloneJson(config.value),
  }
}

function readCachedGeneratedQuestions(context) {
  const cached = quizQuestionCache.get(context.key)
  if (!cached || cached.date !== context.date) return null
  return cloneQuestions(cached.questions)
}

function pruneQuizQuestionCache(currentDate) {
  for (const [key, item] of quizQuestionCache.entries()) {
    if (item.date !== currentDate) {
      quizQuestionCache.delete(key)
    }
  }
  for (const key of quizQuestionRequests.keys()) {
    if (!key.startsWith(`${currentDate}::`)) {
      quizQuestionRequests.delete(key)
    }
  }
  if (quizPageState.activeGenerationKey && !quizPageState.activeGenerationKey.startsWith(`${currentDate}::`)) {
    quizPageState.activeGenerationKey = ''
  }
}

function cloneQuestions(items) {
  return (Array.isArray(items) ? items : []).map(question => ({
    ...question,
    options: Array.isArray(question.options) ? [...question.options] : [],
    word: question.word ? { ...question.word } : question.word,
  }))
}

function cloneJson(value) {
  return value ? JSON.parse(JSON.stringify(value)) : value
}

async function loadTodayStudyWords(levelKey, allWords) {
  let dbWords = []
  try {
    const { startIso, endIso } = localDayRange()
    const data = await invoke('get_study_words_between', {
      level: levelKey,
      startIso,
      endIso,
    })
    dbWords = Array.isArray(data) ? data : []
  } catch (e) {
    console.warn('读取今日学习词失败:', e)
  }

  return mergeWordLists([dbWords], allWords)
}

function mergeWordLists(lists, allWords) {
  const byId = new Map(allWords.map(word => [word.id, word]))
  const byText = new Map(allWords.map(word => [String(word.word || '').toLowerCase(), word]))
  const seen = new Set()
  const merged = []

  for (const item of lists.flat()) {
    const canonical = byId.get(item.id) || byText.get(String(item.word || '').toLowerCase()) || item
    const key = canonical.id ?? String(canonical.word || '').toLowerCase()
    if (!canonical.word || seen.has(key)) continue
    seen.add(key)
    merged.push(canonical)
  }

  return merged
}

function localDayRange() {
  const start = new Date()
  start.setHours(0, 0, 0, 0)
  const end = new Date(start)
  end.setDate(end.getDate() + 1)
  return {
    startIso: localDateTimeString(start),
    endIso: localDateTimeString(end),
  }
}

async function isDailyPlanComplete(levelKey) {
  try {
    return await invoke('is_daily_plan_complete', {
      level: levelKey,
      date: dateKey(),
    })
  } catch (e) {
    console.warn('读取今日计划完成状态失败:', e)
    return false
  }
}

function dateKey() {
  const date = new Date()
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function localDateTimeString(date = new Date()) {
  const pad = value => String(value).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
}

async function generateAiQuestions(seedWords, context) {
  const searchContext = await collectSearchContext(seedWords, context.examTypeLabel, context.config)
  const content = await callModelApiWithRetry({
    config: context.config,
    messages: [
      {
        role: 'system',
        content: '你是熟悉中国英语考试命题风格的老师。只返回 JSON，不要返回 Markdown。',
      },
      {
        role: 'user',
        content: JSON.stringify({
          instruction: '请严格围绕 words 列表中的目标词生成中国考试常见题型。每一道题必须考查对应目标词，不要替换成其他词。题目要像中考、高考、四六级常见命题：重视语境、固定搭配、词义辨析、完形填空、阅读词义推断。每题必须有题干、选项、正确答案、解析。',
          search_instruction: searchContext.length
            ? '已提供联网搜索摘要，请优先结合搜索摘要中的真实搭配、例句、考试语境；但不要编造来源链接。'
            : '联网搜索未启用、失败或没有结果，请直接基于本地词库和参考释义生成完整题目。',
          exam_type: context.examTypeLabel,
          source: 'today_study_words',
          count: seedWords.length,
          search_context: searchContext,
          schema: {
            questions: [
              {
                word: 'target word',
                type: '单项选择/完形填空/语境填空/词义辨析/语法搭配/阅读词义推断',
                stem: '题干',
                options: ['A. ...', 'B. ...', 'C. ...', 'D. ...'],
                answer: 'A',
                reference: '正确答案完整文本或中文释义',
                explanation: '解析',
              },
            ],
          },
          words: seedWords.map(word => ({
            word: word.word,
            definition: plainDefinition(word.definition),
            example: word.example,
            example_translation: word.example_translation || '',
            frequency: word.frequency,
          })),
        }),
      },
    ],
  })

  const parsed = extractJson(content)
  const generated = Array.isArray(parsed?.questions) ? parsed.questions : []
  const normalized = seedWords
    .map((word, index) => normalizeGeneratedQuestion(generated[index] || {}, word, context))
    .filter(Boolean)

  return normalized.length ? normalized : buildLocalQuestions(seedWords)
}

async function collectSearchContext(seedWords, selectedType, configValue) {
  if (!configValue?.search?.enabled) return []
  const wordsText = seedWords.map(word => word.word).join(' ')
  try {
    const results = await invoke('web_search', {
      config: configValue,
      query: `${selectedType} 英语考试 词汇 搭配 例句 ${wordsText}`,
    })
    return Array.isArray(results)
      ? results.slice(0, configValue.search.search_count || 5).map(item => ({
        title: item.title,
        snippet: item.snippet,
        url: item.url,
      }))
      : []
  } catch (e) {
    console.warn('联网搜索失败，继续使用 AI 本地生成:', e)
    return []
  }
}

function normalizeGeneratedQuestion(item, fallbackWord, context) {
  const word = fallbackWord
  const options = Array.isArray(item?.options) ? item.options.map(String).filter(Boolean) : []
  const answer = String(item?.answer || item?.correct_answer || '')
  const reference = String(item?.reference || plainDefinition(word.definition))

  return {
    word,
    type: String(item?.type || context.examTypeLabel || '综合题型'),
    stem: String(item?.stem || `写出 ${word.word} 在考试语境中的含义。`),
    options,
    correctAnswer: answer || reference,
    reference,
    explanation: String(item?.explanation || ''),
  }
}

function buildLocalQuestions(seedWords) {
  return seedWords.map(word => ({
    word,
    type: '释义默写',
    stem: `写出单词 “${word.word}” 的中文释义。`,
    options: [],
    correctAnswer: '',
    reference: plainDefinition(word.definition),
    explanation: '',
  }))
}

function normalizeAiResult(parsed, fallback) {
  if (!parsed || typeof parsed !== 'object') return fallback
  const aiItems = Array.isArray(parsed.items) ? parsed.items : []
  const items = questions.value.map((question, index) => {
    const aiItem = aiItems[index] || {}
    const { answer, isCorrect } = gradeQuestion(question, index)
    return {
      word: String(aiItem.word || question.word.word || ''),
      answer,
      is_correct: isCorrect,
      score: questionScore(isCorrect),
      analysis: String(aiItem.analysis || fallback.items[index]?.analysis || ''),
      suggestion: String(aiItem.suggestion || fallback.items[index]?.suggestion || ''),
    }
  })
  const correctCount = items.filter(item => item.is_correct).length
  const score = questions.value.length ? Math.round((correctCount / questions.value.length) * 100) : 0
  const advice = Array.isArray(parsed.advice)
    ? parsed.advice.map(String)
    : parsed.advice
      ? [String(parsed.advice)]
      : fallback.advice

  return {
    score,
    summary: `按参考答案自动核算：答对 ${correctCount} / ${questions.value.length}。AI 解析仅用于说明和建议。`,
    advice,
    items,
  }
}

function gradeQuestion(question, index) {
  const answer = answers.value[index] || ''
  const reference = question.correctAnswer || question.reference
  return {
    answer,
    isCorrect: answerMatches(answer, reference),
  }
}

function questionScore(isCorrect) {
  if (!isCorrect || !questions.value.length) return 0
  return Math.round(100 / questions.value.length)
}

async function saveQuizRecord(finalResult) {
  await saveQuizRecordFromContext(createGradingContext(), finalResult)
}

async function saveQuizRecordFromContext(context, finalResult) {
  try {
    await invoke('save_quiz_record', {
      record: buildQuizRecordFromContext(context, finalResult),
      retentionDays: context.config?.record_retention_days ?? 7,
    })
  } catch (e) {
    console.warn('淇濆瓨娴嬭瘯璁板綍澶辫触:', e)
  }
}

function buildQuizRecord(finalResult) {
  return buildQuizRecordFromContext(createGradingContext(), finalResult)
}

function buildQuizRecordFromContext(context, finalResult) {
  return {
    level: context.level,
    level_name: context.levelName,
    exam_type: context.examType,
    exam_type_name: context.examTypeName,
    quiz_size: context.questions.length,
    created_at: localDateTimeString(),
    questions: context.questions.map((question, index) => ({
      index: index + 1,
      word: question.word.word,
      type: question.type,
      stem: question.stem,
      options: question.options,
      correct_answer: question.correctAnswer,
      reference: question.reference,
      explanation: question.explanation,
      answer: context.answers[index] || '',
      result: finalResult.items?.[index] || null,
    })),
    result: finalResult,
  }
}

function extractJson(content) {
  try {
    return JSON.parse(content)
  } catch {
    const start = content.indexOf('{')
    const end = content.lastIndexOf('}')
    if (start >= 0 && end > start) {
      return JSON.parse(content.slice(start, end + 1))
    }
    throw new Error('模型未返回 JSON')
  }
}

function plainDefinition(definition) {
  try {
    const parsed = JSON.parse(definition)
    if (Array.isArray(parsed)) {
      return parsed.map(item => item.meaning || item.pos || '').filter(Boolean).join('；')
    }
  } catch {
    // Imported word banks may use plain text definitions.
  }
  return definition || ''
}

function answerMatches(answer, reference) {
  const rawAnswer = String(answer || '').trim()
  const rawReference = String(reference || '').trim()
  const answerChoice = extractChoice(rawAnswer)
  const referenceChoice = extractChoice(rawReference)
  if (referenceChoice) {
    return answerChoice === referenceChoice || rawAnswer.toUpperCase() === referenceChoice
  }

  const normalizedAnswer = normalize(answer)
  const normalizedReference = normalize(reference)
  if (/^[a-z0-9]$/i.test(normalizedAnswer)) {
    return normalizedAnswer === normalizedReference
  }
  return Boolean(normalizedAnswer && normalizedReference)
    && (normalizedReference.includes(normalizedAnswer) || normalizedAnswer.includes(normalizedReference))
}

function extractChoice(value) {
  const match = String(value || '').trim().match(/^([A-D])(?:[\s.、):：]|$)/i)
  return match ? match[1].toUpperCase() : ''
}

function normalize(value) {
  return String(value || '').toLowerCase().replace(/[^\p{L}\p{N}]+/gu, '')
}

function resetQuiz() {
  quizViewVersion++
  quizPageState.activeGenerationKey = ''
  quizPageState.activeGradingKey = ''
  started.value = false
  generating.value = false
  grading.value = false
  result.value = null
  questions.value = []
  answers.value = []
}
</script>
