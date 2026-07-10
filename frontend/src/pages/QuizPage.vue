<template>
  <div class="mx-auto max-w-4xl p-4 sm:p-6">
    <div class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="text-sm font-medium text-blue-600">{{ levelName }} · 模拟测试</p>
        <h2 class="text-2xl font-bold text-slate-900">考试题型训练</h2>
      </div>
      <div class="grid gap-2 sm:grid-cols-2">
        <select v-model="selectedExamType" class="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm text-slate-700">
          <option v-for="type in examTypes" :key="type.key" :value="type.key">{{ type.label }}</option>
        </select>
        <select v-model.number="quizSize" class="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm text-slate-700">
          <option :value="5">5 题</option>
          <option :value="10">10 题</option>
          <option :value="15">15 题</option>
        </select>
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

    <div v-else-if="!started" class="rounded-2xl bg-white p-6 shadow-sm ring-1 ring-slate-100 sm:p-8">
      <div class="mb-6">
        <p class="text-sm font-medium text-slate-500">优先用 AI 按考试高频题型生成；未配置模型时使用本地释义默写兜底。</p>
        <h3 class="mt-1 text-2xl font-bold text-slate-900">准备开始 {{ Math.min(quizSize, words.length) }} 题测试</h3>
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
const questions = ref([])
const answers = ref([])
const quizSize = ref(10)
const selectedExamType = ref('mixed')
const loading = ref(false)
const started = ref(false)
const generating = ref(false)
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

async function startQuiz() {
  result.value = null
  started.value = true
  generating.value = true
  const seedWords = words.value.slice(0, Math.min(quizSize.value, words.value.length))

  try {
    questions.value = config.value?.model?.api_key
      ? await generateAiQuestions(seedWords)
      : buildLocalQuestions(seedWords)
  } catch (e) {
    console.warn('AI 生成题目失败，使用本地题目:', e)
    questions.value = buildLocalQuestions(seedWords)
  } finally {
    answers.value = questions.value.map(() => '')
    generating.value = false
  }
}

async function submitQuiz() {
  grading.value = true
  const fallback = buildLocalResult()
  try {
    if (!config.value?.model?.api_key) {
      result.value = fallback
      await saveQuizRecord(fallback)
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
              type: question.type,
              stem: question.stem,
              options: question.options,
              correct_answer: question.correctAnswer,
              reference: question.reference,
              answer: answers.value[index] || '',
            })),
          }),
        },
      ],
    })

    const parsed = extractJson(content)
    result.value = normalizeAiResult(parsed, fallback)
    await saveQuizRecord(result.value)
  } catch (e) {
    console.warn('AI 批阅失败，使用本地评分:', e)
    result.value = fallback
    await saveQuizRecord(fallback)
  } finally {
    grading.value = false
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
    summary: config.value?.model?.api_key
      ? `本地兜底评分：答对 ${correctCount} / ${questions.value.length}。`
      : `未配置模型，已使用本地关键词匹配评分：答对 ${correctCount} / ${questions.value.length}。`,
    advice: score >= 80 ? ['继续增加新词量', '保持每日复习'] : ['降低新词量', '优先复习错词'],
    items,
  }
}

async function generateAiQuestions(seedWords) {
  const selectedType = examTypes.find(type => type.key === selectedExamType.value)?.label || '综合题型'
  const searchContext = await collectSearchContext(seedWords, selectedType)
  const content = await invoke('call_model_api', {
    config: config.value,
    messages: [
      {
        role: 'system',
        content: '你是熟悉中国英语考试命题风格的老师。只返回 JSON，不要返回 Markdown。',
      },
      {
        role: 'user',
        content: JSON.stringify({
          instruction: '请基于给定词表生成中国考试常见题型。题目要像中考、高考、四六级常见命题：重视语境、固定搭配、词义辨析、完形填空、阅读词义推断。每题必须有题干、选项、正确答案、解析。',
          search_instruction: searchContext.length
            ? '已提供联网搜索摘要，请优先结合搜索摘要中的真实搭配、例句、考试语境；但不要编造来源链接。'
            : '联网搜索未启用、失败或没有结果，请直接基于本地词库和参考释义生成完整题目。',
          exam_type: selectedType,
          count: Math.min(quizSize.value, seedWords.length),
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
            frequency: word.frequency,
          })),
        }),
      },
    ],
  })

  const parsed = extractJson(content)
  const generated = Array.isArray(parsed?.questions) ? parsed.questions : []
  const normalized = generated
    .slice(0, seedWords.length)
    .map((item, index) => normalizeGeneratedQuestion(item, seedWords[index]))
    .filter(Boolean)

  return normalized.length ? normalized : buildLocalQuestions(seedWords)
}

async function collectSearchContext(seedWords, selectedType) {
  if (!config.value?.search?.enabled) return []
  const wordsText = seedWords.map(word => word.word).join(' ')
  try {
    const results = await invoke('web_search', {
      config: config.value,
      query: `${selectedType} 英语考试 词汇 搭配 例句 ${wordsText}`,
    })
    return Array.isArray(results)
      ? results.slice(0, config.value.search.search_count || 5).map(item => ({
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

function normalizeGeneratedQuestion(item, fallbackWord) {
  const wordText = String(item?.word || fallbackWord.word)
  const word = words.value.find(candidate => candidate.word.toLowerCase() === wordText.toLowerCase()) || fallbackWord
  const options = Array.isArray(item?.options) ? item.options.map(String).filter(Boolean) : []
  const answer = String(item?.answer || item?.correct_answer || '')
  const reference = String(item?.reference || plainDefinition(word.definition))

  return {
    word,
    type: String(item?.type || examTypes.find(type => type.key === selectedExamType.value)?.label || '综合题型'),
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
  try {
    await invoke('save_quiz_record', {
      record: buildQuizRecord(finalResult),
      retentionDays: config.value?.record_retention_days ?? 7,
    })
  } catch (e) {
    console.warn('淇濆瓨娴嬭瘯璁板綍澶辫触:', e)
  }
}

function buildQuizRecord(finalResult) {
  return {
    level: level.value,
    level_name: levelName.value,
    exam_type: selectedExamType.value,
    exam_type_name: examTypes.find(type => type.key === selectedExamType.value)?.label || selectedExamType.value,
    quiz_size: questions.value.length,
    created_at: new Date().toISOString(),
    questions: questions.value.map((question, index) => ({
      index: index + 1,
      word: question.word.word,
      type: question.type,
      stem: question.stem,
      options: question.options,
      correct_answer: question.correctAnswer,
      reference: question.reference,
      explanation: question.explanation,
      answer: answers.value[index] || '',
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
  started.value = false
  generating.value = false
  grading.value = false
  result.value = null
  questions.value = []
  answers.value = []
}
</script>
