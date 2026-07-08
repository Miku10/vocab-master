<template>
  <div class="max-w-4xl mx-auto p-6">
    <!-- 顶部：学段选择 + 统计 -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex gap-2">
        <button v-for="lv in levels" :key="lv.key"
                @click="switchLevel(lv.key)"
                class="px-4 py-2 rounded-xl text-sm font-medium transition-all duration-200"
                :class="currentLevel === lv.key
                  ? 'bg-blue-600 text-white shadow-md'
                  : 'bg-white text-slate-600 border border-slate-200 hover:bg-slate-50'">
          {{ lv.label }}
          <span v-if="lv.count > 0" class="ml-1.5 px-1.5 py-0.5 rounded-full text-xs"
                :class="currentLevel === lv.key ? 'bg-blue-500' : 'bg-slate-100 text-slate-500'">
            {{ lv.count }}
          </span>
        </button>
      </div>
      <div class="text-sm text-slate-400">
        已学 {{ learnedCount }} / {{ words.length }}
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-center py-20 text-slate-400">
      <div class="inline-block animate-spin text-3xl mb-3">🔄</div>
      <p>加载词库中...</p>
    </div>

    <!-- 空状态 -->
    <div v-else-if="words.length === 0" class="text-center py-20">
      <div class="text-6xl mb-4">📭</div>
      <p class="text-slate-500 mb-2">当前学段暂无词库数据</p>
      <p class="text-sm text-slate-400">请确保词库文件已正确放置</p>
    </div>

    <!-- 词卡模式 -->
    <div v-else-if="mode === 'card'">
      <WordCard :wordData="currentWord" :currentIndex="currentIndex" :total="words.length"
                @play-audio="playAudio" @mark-easy="markEasy" @mark-hard="markHard"
                @generate-quiz="generateQuiz" @prev="prevWord" @next="nextWord" />
    </div>

    <!-- 选择题模式 -->
    <div v-else-if="mode === 'quiz'">
      <div class="bg-white rounded-3xl shadow-lg border border-slate-100 overflow-hidden">
        <div class="h-2 bg-gradient-to-r from-indigo-500 to-purple-500"></div>
        <div class="p-8">
          <div class="flex items-center gap-2 mb-4">
            <span class="px-3 py-1 rounded-full text-xs font-semibold bg-indigo-100 text-indigo-700">📝 选择题</span>
            <span class="text-xs text-slate-400">AI 出题</span>
          </div>
          <h3 class="text-lg font-semibold text-slate-800 mb-6">{{ quiz.question }}</h3>
          <div class="space-y-3 mb-6">
            <button v-for="(opt, i) in quiz.options" :key="i"
                    @click="selectAnswer(i)"
                    :disabled="answered"
                    class="w-full text-left px-5 py-3.5 rounded-xl border transition-all duration-200"
                    :class="answerClass(i)">
              <span class="font-semibold mr-3">{{ ['A','B','C','D'][i] }}</span>
              {{ opt }}
            </button>
          </div>
          <div v-if="answered" class="mb-6 p-4 rounded-xl"
               :class="isCorrect ? 'bg-emerald-50 border border-emerald-200' : 'bg-red-50 border border-red-200'">
            <p :class="isCorrect ? 'text-emerald-700' : 'text-red-600'" class="font-medium">
              {{ isCorrect ? '✅ 回答正确！' : '❌ 回答错误' }}
            </p>
            <p class="text-sm text-slate-600 mt-2">{{ quiz.explanation }}</p>
          </div>
          <button @click="mode = 'card'" class="px-5 py-2.5 rounded-xl text-sm font-medium text-slate-600 hover:bg-slate-100">
            ← 返回词卡
          </button>
        </div>
      </div>
    </div>

    <!-- 完形填空模式 -->
    <div v-else-if="mode === 'passage'">
      <div class="bg-white rounded-3xl shadow-lg border border-slate-100 overflow-hidden">
        <div class="h-2 bg-gradient-to-r from-amber-500 to-orange-500"></div>
        <div class="p-8">
          <div class="flex items-center gap-2 mb-4">
            <span class="px-3 py-1 rounded-full text-xs font-semibold bg-amber-100 text-amber-700">📖 完形填空</span>
            <span class="text-xs text-slate-400">AI 生成</span>
          </div>
          <div class="prose prose-slate max-w-none mb-6 p-4 bg-slate-50 rounded-xl" v-html="formattedPassage"></div>
          <div v-for="(q, qi) in passage.questions" :key="qi" class="mb-6 p-4 rounded-xl border border-slate-100">
            <p class="font-medium text-slate-800 mb-3">第 {{ q.blank_index }} 空</p>
            <div class="space-y-2">
              <button v-for="(opt, oi) in q.options" :key="oi"
                      @click="selectPassageAnswer(qi, oi)"
                      :disabled="passageAnswered[qi] !== undefined"
                      class="w-full text-left px-4 py-2.5 rounded-lg border transition-all"
                      :class="passageAnswerClass(qi, oi)">
                <span class="font-semibold mr-2">{{ ['A','B','C','D'][oi] }}</span>
                {{ opt }}
              </button>
            </div>
            <div v-if="passageAnswered[qi] !== undefined" class="mt-3 text-sm"
                 :class="passageAnswered[qi] ? 'text-emerald-600' : 'text-red-600'">
              {{ passageAnswered[qi] ? '✅ 正确' : '❌ 错误' }} — {{ q.explanation }}
            </div>
          </div>
          <button @click="mode = 'card'" class="px-5 py-2.5 rounded-xl text-sm font-medium text-slate-600 hover:bg-slate-100">
            ← 返回词卡
          </button>
        </div>
      </div>
    </div>

    <!-- 底部操作 -->
    <div v-if="mode === 'card' && words.length > 0" class="mt-6 flex justify-center gap-3">
      <button @click="generatePassage" class="px-5 py-2.5 rounded-xl text-sm font-medium text-white bg-gradient-to-r from-amber-500 to-orange-500 hover:from-amber-600 hover:to-orange-600 shadow-md transition-all">
        📝 生成完形填空练习
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import WordCard from '../components/WordCard.vue'

const levels = [
  { key: 'primary', label: '小学' },
  { key: 'junior', label: '初中' },
  { key: 'high', label: '高中' },
  { key: 'cet4', label: '四级' },
  { key: 'cet6', label: '六级' },
]

const currentLevel = ref('junior')
const words = ref([])
const currentIndex = ref(0)
const loading = ref(false)
const mode = ref('card') // card | quiz | passage
const learnedCount = ref(0)

// Quiz state
const quiz = ref({ question: '', options: [], answer: 0, explanation: '' })
const answered = ref(false)
const isCorrect = ref(false)
const selectedAnswer = ref(null)

// Passage state
const passage = ref({ passage: '', questions: [] })
const passageAnswered = ref([])

const currentWord = computed(() => words.value[currentIndex.value] || {})

const formattedPassage = computed(() => {
  let p = passage.value.passage || ''
  // Replace ___1___ with styled blanks
  p = p.replace(/___(\d+)___/g, '<span class="inline-block px-3 py-1 mx-1 rounded-lg bg-amber-100 text-amber-700 font-semibold border border-amber-300">___$1___</span>')
  return p
})

async function switchLevel(level) {
  if (level === currentLevel.value) return
  currentLevel.value = level
  currentIndex.value = 0
  mode.value = 'card'
  await loadWords()
}

async function loadWords() {
  loading.value = true
  try {
    const data = await invoke('load_words', { level: currentLevel.value })
    words.value = data
    currentIndex.value = 0
    // Update level counts
    const idx = levels.findIndex(l => l.key === currentLevel.value)
    if (idx >= 0) levels[idx].count = data.length
  } catch (e) {
    console.error('加载词库失败:', e)
    words.value = []
  }
  loading.value = false
}

function prevWord() {
  if (currentIndex.value > 0) currentIndex.value--
}

function nextWord() {
  if (currentIndex.value < words.value.length - 1) currentIndex.value++
}

async function playAudio() {
  try {
    await invoke('play_word_audio', { word: currentWord.value.word })
  } catch (e) {
    console.warn('播放发音失败:', e)
  }
}

async function markEasy() {
  try {
    await invoke('mark_word_learned', { wordId: currentWord.value.id })
    learnedCount.value++
    nextWord()
  } catch (e) {
    console.warn('标记失败:', e)
  }
}

async function markHard() {
  try {
    await invoke('mark_word_hard', { wordId: currentWord.value.id })
    nextWord()
  } catch (e) {
    console.warn('标记失败:', e)
  }
}

async function generateQuiz() {
  try {
    const result = await invoke('generate_word_quiz', {
      word: currentWord.value.word,
      definition: currentWord.value.definition,
      level: currentLevel.value
    })
    quiz.value = result
    answered.value = false
    isCorrect.value = false
    selectedAnswer.value = null
    mode.value = 'quiz'
  } catch (e) {
    alert('生成题目失败: ' + e)
  }
}

function selectAnswer(idx) {
  if (answered.value) return
  selectedAnswer.value = idx
  answered.value = true
  isCorrect.value = idx === quiz.value.answer
}

async function generatePassage() {
  try {
    const learnedWords = words.value.filter((_, i) => i <= currentIndex.value).slice(0, 10)
    if (learnedWords.length < 3) {
      alert('请先学习至少3个单词再生成练习')
      return
    }
    const wordList = learnedWords.map(w => w.word)
    const result = await invoke('generate_passage_quiz', { words: wordList })
    passage.value = result
    passageAnswered.value = new Array(result.questions.length).fill(undefined)
    mode.value = 'passage'
  } catch (e) {
    alert('生成完形填空失败: ' + e)
  }
}

function selectPassageAnswer(qIdx, oIdx) {
  if (passageAnswered.value[qIdx] !== undefined) return
  const q = passage.value.questions[qIdx]
  const correct = oIdx === q.answer
  passageAnswered.value = [...passageAnswered.value]
  passageAnswered.value[qIdx] = correct
}

function answerClass(idx) {
  if (!answered.value) return 'border-slate-200 hover:border-indigo-300 hover:bg-indigo-50'
  if (idx === quiz.value.answer) return 'border-emerald-500 bg-emerald-50 text-emerald-700'
  if (idx === selectedAnswer.value) return 'border-red-500 bg-red-50 text-red-600'
  return 'border-slate-200 opacity-50'
}

function passageAnswerClass(qIdx, oIdx) {
  const answered = passageAnswered.value[qIdx]
  if (answered === undefined) return 'border-slate-200 hover:border-amber-300 hover:bg-amber-50'
  const q = passage.value.questions[qIdx]
  if (idx === q.answer) return 'border-emerald-500 bg-emerald-50 text-emerald-700'
  if (idx === oIdx && !answered) return 'border-red-500 bg-red-50 text-red-600'
  return 'border-slate-200 opacity-50'
}

onMounted(() => {
  loadWords()
})
</script>
