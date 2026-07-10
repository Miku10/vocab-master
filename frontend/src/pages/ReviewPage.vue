<template>
  <div class="mx-auto max-w-6xl space-y-5 p-4 sm:p-6">
    <header class="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
      <div>
        <p class="text-sm font-medium text-blue-600">复习巩固</p>
        <h2 class="text-2xl font-bold text-slate-900">错题本与测验记录</h2>
        <p class="mt-1 text-sm text-slate-500">测验完成后会保存完整题目、作答、解析和学习建议；错题会自动进入这里。</p>
      </div>
      <button
        type="button"
        class="rounded-xl bg-white px-4 py-2 text-sm font-semibold text-slate-700 ring-1 ring-slate-200 hover:bg-slate-50"
        @click="loadReviewData"
      >
        刷新记录
      </button>
    </header>

    <div class="grid gap-2 rounded-2xl bg-slate-100 p-1 sm:w-fit sm:grid-cols-2">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        type="button"
        class="rounded-xl px-4 py-2 text-sm font-semibold transition"
        :class="activeTab === tab.key ? 'bg-white text-blue-700 shadow-sm' : 'text-slate-600 hover:text-slate-900'"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <div v-if="loading" class="rounded-2xl bg-white py-16 text-center text-slate-500 shadow-sm ring-1 ring-slate-100">
      正在读取记录...
    </div>

    <section v-else-if="activeTab === 'wrong'" class="space-y-4">
      <div class="grid gap-3 sm:grid-cols-3">
        <div class="rounded-2xl bg-white p-4 shadow-sm ring-1 ring-slate-100">
          <p class="text-sm text-slate-500">错题条目</p>
          <p class="mt-1 text-2xl font-bold text-slate-900">{{ wrongItems.length }}</p>
        </div>
        <div class="rounded-2xl bg-white p-4 shadow-sm ring-1 ring-slate-100">
          <p class="text-sm text-slate-500">词卡来源</p>
          <p class="mt-1 text-2xl font-bold text-slate-900">{{ wordWrongCount }}</p>
        </div>
        <div class="rounded-2xl bg-white p-4 shadow-sm ring-1 ring-slate-100">
          <p class="text-sm text-slate-500">测验来源</p>
          <p class="mt-1 text-2xl font-bold text-slate-900">{{ quizWrongCount }}</p>
        </div>
      </div>

      <div v-if="wrongItems.length === 0" class="rounded-2xl bg-white py-16 text-center text-slate-500 shadow-sm ring-1 ring-slate-100">
        暂无错题。完成词卡或测验后，这里会自动记录需要复习的内容。
      </div>

      <div v-else class="grid gap-3 lg:grid-cols-2">
        <article
          v-for="(item, index) in wrongItems"
          :key="`${item.source}-${item.word}-${index}`"
          class="rounded-2xl bg-white p-5 shadow-sm ring-1 ring-slate-100"
        >
          <div class="flex items-start justify-between gap-3">
            <div>
              <p class="text-lg font-bold text-slate-900">{{ item.word }}</p>
              <p class="mt-1 text-sm text-slate-500">{{ levelLabel(item.level) }} · {{ formatDate(item.created_at) || '暂无时间' }}</p>
            </div>
            <span
              class="rounded-full px-3 py-1 text-xs font-semibold"
              :class="item.source === 'quiz' ? 'bg-amber-100 text-amber-700' : 'bg-blue-100 text-blue-700'"
            >
              {{ item.source === 'quiz' ? '测验错题' : '词卡错词' }}
            </span>
          </div>

          <div v-if="displayValue(item.answer)" class="mt-4 rounded-xl bg-slate-50 p-3 text-sm text-slate-700">
            <span class="font-semibold text-slate-900">你的答案：</span>{{ displayValue(item.answer) }}
          </div>
          <p class="mt-4 text-sm leading-6 text-slate-700">{{ displayValue(item.analysis) }}</p>
          <p class="mt-2 text-sm leading-6 text-blue-700">{{ displayValue(item.suggestion) }}</p>
        </article>
      </div>
    </section>

    <section v-else class="space-y-4">
      <div v-if="quizRecords.length === 0" class="rounded-2xl bg-white py-16 text-center text-slate-500 shadow-sm ring-1 ring-slate-100">
        暂无测验记录。完成一次 AI 测验后会自动保存到这里。
      </div>

      <template v-else>
        <article
          v-for="record in quizRecords"
          :key="record.id || record.created_at"
          class="rounded-2xl bg-white p-5 shadow-sm ring-1 ring-slate-100"
        >
          <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p class="text-sm font-medium text-blue-600">{{ record.exam_type_name || record.exam_type || '综合题型' }}</p>
              <h3 class="mt-1 text-xl font-bold text-slate-900">{{ levelLabel(record.level) }}测验 · {{ formatDate(record.created_at) }}</h3>
              <p class="mt-2 text-sm leading-6 text-slate-600">{{ recordSummary(record) }}</p>
            </div>
            <div class="rounded-2xl bg-blue-50 px-5 py-4 text-center">
              <p class="text-sm font-semibold text-blue-700">得分</p>
              <p class="text-3xl font-bold text-blue-700">{{ recordScore(record) }}</p>
            </div>
          </div>

          <div v-if="adviceList(record).length" class="mt-4 flex flex-wrap gap-2">
            <span v-for="item in adviceList(record)" :key="item" class="rounded-full bg-slate-100 px-3 py-1 text-sm text-slate-700">
              {{ item }}
            </span>
          </div>

          <details class="mt-5 rounded-xl border border-slate-200 bg-slate-50">
            <summary class="cursor-pointer px-4 py-3 text-sm font-semibold text-slate-700">
              查看全部题目和解析（{{ record.questions?.length || 0 }} 题）
            </summary>
            <div class="space-y-3 border-t border-slate-200 p-4">
              <div
                v-for="question in record.questions || []"
                :key="`${record.id || record.created_at}-${question.index}`"
                class="rounded-xl bg-white p-4 ring-1 ring-slate-100"
              >
                <div class="flex flex-wrap items-center justify-between gap-2">
                  <p class="text-sm font-semibold text-slate-500">第 {{ question.index }} 题 · {{ question.type }}</p>
                  <span
                    v-if="question.result"
                    class="rounded-full px-2.5 py-1 text-xs font-semibold"
                    :class="question.result.is_correct ? 'bg-emerald-100 text-emerald-700' : 'bg-red-100 text-red-600'"
                  >
                    {{ question.result.score ?? 0 }} 分
                  </span>
                </div>
                <p class="mt-2 text-base font-semibold leading-7 text-slate-900">{{ question.stem }}</p>
                <div v-if="question.options?.length" class="mt-3 grid gap-2 text-sm text-slate-600">
                  <p v-for="option in question.options" :key="option">{{ option }}</p>
                </div>
                <div class="mt-3 grid gap-2 text-sm text-slate-700 sm:grid-cols-2">
                  <p><span class="font-semibold text-slate-900">你的答案：</span>{{ question.answer || '未作答' }}</p>
                  <p><span class="font-semibold text-slate-900">参考答案：</span>{{ question.correct_answer || question.reference || '暂无' }}</p>
                </div>
                <p class="mt-3 text-sm leading-6 text-slate-700">{{ question.result?.analysis || question.explanation || '暂无解析' }}</p>
                <p v-if="question.result?.suggestion" class="mt-2 text-sm leading-6 text-blue-700">{{ question.result.suggestion }}</p>
              </div>
            </div>
          </details>
        </article>
      </template>
    </section>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const tabs = [
  { key: 'wrong', label: '错题本' },
  { key: 'records', label: '测验记录' },
]

const levelOptions = {
  primary: '小学',
  junior: '初中',
  high: '高中',
  cet4: '四级',
  cet6: '六级',
  unknown: '未知学段',
}

const activeTab = ref('wrong')
const loading = ref(false)
const wrongItems = ref([])
const quizRecords = ref([])

const wordWrongCount = computed(() => wrongItems.value.filter(item => item.source === 'word').length)
const quizWrongCount = computed(() => wrongItems.value.filter(item => item.source === 'quiz').length)

onMounted(loadReviewData)

async function loadReviewData() {
  loading.value = true
  try {
    const [wrong, records] = await Promise.all([
      invoke('get_wrong_book'),
      invoke('get_quiz_records'),
    ])
    wrongItems.value = Array.isArray(wrong) ? wrong : []
    quizRecords.value = Array.isArray(records) ? records : []
  } catch (e) {
    console.error('读取复习记录失败:', e)
    wrongItems.value = []
    quizRecords.value = []
  } finally {
    loading.value = false
  }
}

function levelLabel(level) {
  return levelOptions[level] || level || '未知学段'
}

function formatDate(value) {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function displayValue(value) {
  if (value === null || value === undefined) return ''
  if (typeof value === 'object') return JSON.stringify(value)
  return String(value)
}

function recordScore(record) {
  return record?.result?.score ?? record?.score ?? 0
}

function recordSummary(record) {
  return record?.result?.summary || record?.summary || '暂无总结'
}

function adviceList(record) {
  const advice = record?.result?.advice
  if (Array.isArray(advice)) return advice.map(String)
  return advice ? [String(advice)] : []
}
</script>
