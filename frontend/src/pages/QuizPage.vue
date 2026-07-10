<template>
  <div class="mx-auto max-w-4xl p-4 sm:p-6">
    <div class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="text-sm font-medium text-blue-600">{{ levelName }} · 模拟测试</p>
        <h2 class="text-2xl font-bold text-slate-900">释义问答</h2>
      </div>
      <select v-model.number="quizSize" class="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm text-slate-700">
        <option :value="5">5 题</option>
        <option :value="10">10 题</option>
        <option :value="15">15 题</option>
      </select>
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

    <div v-else-if="!started" class="rounded-2xl bg-white p-6 shadow-sm ring-1 ring-slate-100 sm:p-8">
      <div class="mb-6">
        <p class="text-sm font-medium text-slate-500">题目会优先抽取高频词</p>
        <h3 class="mt-1 text-2xl font-bold text-slate-900">准备开始 {{ Math.min(quizSize, words.length) }} 题测试</h3>
      </div>
      <button class="rounded-xl bg-blue-600 px-5 py-3 text-sm font-semibold text-white hover:bg-blue-700" @click="startQuiz">
        开始测试
      </button>
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
        <div v-for="item in result.items" :key="item.word" class="rounded-2xl bg-white p-5 shadow-sm ring-1 ring-slate-100">
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
      <div v-for="(question, index) in questions" :key="question.word.id" class="rounded-2xl bg-white p-5 shadow-sm ring-1 ring-slate-100">
        <div class="mb-4 flex items-center justify-between gap-3">
          <span class="rounded-full bg-slate-100 px-3 py-1 text-sm font-semibold text-slate-600">第 {{ index + 1 }} 题</span>
          <span class="text-sm text-slate-400">#{{ question.word.frequency }}</span>
        </div>
        <label class="block">
          <span class="mb-2 block text-2xl font-bold text-slate-900">{{ question.word.word }}</span>
          <input
            v-model="answers[index]"
            class="w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-slate-900 outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-100"
            placeholder="输入你记得的中文释义"
          />
        </label>
      </div>

      <div class="sticky bottom-4 rounded-2xl border border-slate-200 bg-white/95 p-3 shadow-lg backdrop-blur">
        <button class="w-full rounded-xl bg-blue-600 px-5 py-3 text-sm font-semibold text-white hover:bg-blue-700" type="submit">
          提交并批阅
        </button>
      </div>
    </form>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const levelOptions = [
  { key: 'junior', label: '初中' },
  { key: 'high', label: '高中' },
  { key: 'cet4', label: '四级' },
  { key: 'cet6', label: '六级' },
  { key: 'primary', label: '小学' },
]

const config = ref(null)
const level = ref('junior')
const words = ref([])
const questions = ref([])
const answers = ref([])
const quizSize = ref(10)
const loading = ref(false)
const started = ref(false)
const grading = ref(false)
const result = ref(null)

const levelName = computed(() => levelOptions.find(item => item.key === level.value)?.label || level.value)

onMounted(loadWords)

async function loadWords() {
  loading.value = true
  try {
    const appConfig = await invoke('get_config')
    config.value = appConfig
    level.value = appConfig.active_level || 'junior'
    const data = await invoke('load_words', { level: level.value })
    words.value = [...data].sort((a, b) => (a.frequency || 999999) - (b.frequency || 999999))
  } catch (e) {
    console.error('载入测试词库失败:', e)
    words.value = []
  } finally {
    loading.value = false
  }
}

function startQuiz() {
  result.value = null
  started.value = true
  questions.value = words.value.slice(0, Math.min(quizSize.value, words.value.length)).map(word => ({
    word,
    reference: plainDefinition(word.definition),
  }))
  answers.value = questions.value.map(() => '')
}

async function submitQuiz() {
  grading.value = true
  const fallback = buildLocalResult()
  try {
    if (!config.value?.model?.api_key) {
      result.value = fallback
      return
    }

    const content = await invoke('call_model_api', {
      config: config.value,
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
            questions: questions.value.map((question, index) => ({
              word: question.word.word,
              reference: question.reference,
              answer: answers.value[index] || '',
            })),
          }),
        },
      ],
    })

    const parsed = extractJson(content)
    result.value = normalizeAiResult(parsed, fallback)
  } catch (e) {
    console.warn('AI 批阅失败，使用本地评分:', e)
    result.value = fallback
  } finally {
    grading.value = false
  }
}

function buildLocalResult() {
  const items = questions.value.map((question, index) => {
    const answer = answers.value[index] || ''
    const isCorrect = answerMatches(answer, question.reference)
    return {
      word: question.word.word,
      answer,
      is_correct: isCorrect,
      score: isCorrect ? 10 : 0,
      analysis: isCorrect
        ? `答案命中了参考释义：${question.reference}`
        : `参考释义：${question.reference}`,
      suggestion: isCorrect ? '保持复习节奏。' : '把这个词加入明日重点复习。',
    }
  })
  const correctCount = items.filter(item => item.is_correct).length
  const score = questions.value.length ? Math.round((correctCount / questions.value.length) * 100) : 0

  return {
    score,
    summary: config.value?.model?.api_key
      ? `本地兜底评分：答对 ${correctCount} / ${questions.value.length}。`
      : `未配置模型，已使用本地关键词匹配评分：答对 ${correctCount} / ${questions.value.length}。`,
    advice: score >= 80 ? ['继续增加新词量', '保持每日复习'] : ['降低新词量', '优先复习错词'],
    items,
  }
}

function normalizeAiResult(parsed, fallback) {
  if (!parsed || typeof parsed !== 'object') return fallback
  const items = Array.isArray(parsed.items) && parsed.items.length
    ? parsed.items.map((item, index) => ({
      word: String(item.word || questions.value[index]?.word.word || ''),
      answer: String(item.answer ?? answers.value[index] ?? ''),
      is_correct: Boolean(item.is_correct),
      score: Number(item.score ?? 0),
      analysis: String(item.analysis || fallback.items[index]?.analysis || ''),
      suggestion: String(item.suggestion || fallback.items[index]?.suggestion || ''),
    }))
    : fallback.items
  const advice = Array.isArray(parsed.advice)
    ? parsed.advice.map(String)
    : parsed.advice
      ? [String(parsed.advice)]
      : fallback.advice

  return {
    score: Math.max(0, Math.min(100, Number(parsed.score ?? fallback.score))),
    summary: String(parsed.summary || fallback.summary),
    advice,
    items,
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
  const normalizedAnswer = normalize(answer)
  const normalizedReference = normalize(reference)
  return normalizedAnswer.length >= 2 && normalizedReference.includes(normalizedAnswer)
}

function normalize(value) {
  return String(value || '').toLowerCase().replace(/[^\p{L}\p{N}]+/gu, '')
}

function resetQuiz() {
  started.value = false
  grading.value = false
  result.value = null
  questions.value = []
  answers.value = []
}
</script>
