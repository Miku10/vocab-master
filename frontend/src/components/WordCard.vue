<template>
  <div class="max-w-2xl mx-auto">
    <div class="bg-white rounded-3xl shadow-lg border border-slate-100 overflow-hidden
                transform transition-all duration-300 hover:shadow-xl">
      <div class="h-2 bg-gradient-to-r from-blue-500 via-indigo-500 to-purple-500"></div>
      
      <div class="p-8">
        <div class="flex items-center gap-3 mb-4">
          <span class="px-3 py-1 rounded-full text-xs font-semibold" :class="levelColor">{{ levelName }}</span>
          <span class="text-xs text-slate-400">#{{ wordData.frequency }}</span>
        </div>

        <div class="flex items-center gap-4 mb-6">
          <h2 class="text-4xl font-bold text-slate-800">{{ wordData.word }}</h2>
          <button @click="$emit('play-audio')"
                  class="w-12 h-12 rounded-full bg-blue-50 hover:bg-blue-100 
                         flex items-center justify-center transition-colors text-blue-600 text-xl">
            🔊
          </button>
        </div>

        <div class="flex gap-6 mb-6" v-if="wordData.phonetic_en || wordData.phonetic_us">
          <div v-if="wordData.phonetic_en" class="flex items-center gap-2">
            <span class="text-xs text-slate-400">🇬🇧</span>
            <span class="text-sm text-slate-600">{{ wordData.phonetic_en }}</span>
          </div>
          <div v-if="wordData.phonetic_us" class="flex items-center gap-2">
            <span class="text-xs text-slate-400">🇺🇸</span>
            <span class="text-sm text-slate-600">{{ wordData.phonetic_us }}</span>
          </div>
        </div>

        <div class="mb-6">
          <h3 class="text-sm font-semibold text-slate-500 mb-2">释义</h3>
          <div class="space-y-2">
            <div v-for="(def, i) in parsedDefinitions" :key="i"
                 class="flex gap-3 p-3 rounded-xl bg-slate-50">
              <span class="px-2 py-0.5 rounded bg-blue-100 text-blue-700 text-xs font-semibold">
                {{ def.pos }}
              </span>
              <span class="text-sm text-slate-700">{{ def.meaning }}</span>
            </div>
          </div>
        </div>

        <div v-if="wordData.example" class="mb-6">
          <h3 class="text-sm font-semibold text-slate-500 mb-2">📖 例句</h3>
          <div class="p-4 rounded-xl bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-100">
            <p class="text-sm text-slate-700 italic">"{{ wordData.example }}"</p>
          </div>
        </div>

        <div class="flex gap-3">
          <button @click="$emit('mark-easy')"
                  class="flex-1 py-3 rounded-xl font-semibold text-sm transition-all
                         bg-emerald-50 text-emerald-700 hover:bg-emerald-100">
            ✅ 已掌握
          </button>
          <button @click="$emit('mark-hard')"
                  class="flex-1 py-3 rounded-xl font-semibold text-sm transition-all
                         bg-red-50 text-red-600 hover:bg-red-100">
            ❌ 困难
          </button>
          <button @click="$emit('generate-quiz')"
                  class="flex-1 py-3 rounded-xl font-semibold text-sm transition-all
                         bg-indigo-50 text-indigo-700 hover:bg-indigo-100">
            ✨ AI出题
          </button>
        </div>
      </div>
    </div>

    <div class="flex justify-between items-center mt-6">
      <button @click="$emit('prev')"
              class="px-5 py-2.5 rounded-xl text-sm font-medium text-slate-600 
                     bg-white border border-slate-200 hover:bg-slate-50 transition-all">
        ← 上一个
      </button>
      <span class="text-sm text-slate-400">{{ currentIndex + 1 }} / {{ total }}</span>
      <button @click="$emit('next')"
              class="px-5 py-2.5 rounded-xl text-sm font-medium text-white 
                     bg-blue-600 hover:bg-blue-700 transition-all">
        下一个 →
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  wordData: { type: Object, required: true },
  currentIndex: { type: Number, default: 0 },
  total: { type: Number, default: 0 },
})

const levelName = computed(() => {
  const map = { primary: '小学', junior: '初中', high: '高中', cet4: '四级', cet6: '六级' }
  return map[props.wordData.level] || props.wordData.level
})

const levelColor = computed(() => {
  const map = {
    primary: 'bg-purple-100 text-purple-700',
    junior: 'bg-blue-100 text-blue-700',
    high: 'bg-green-100 text-green-700',
    cet4: 'bg-amber-100 text-amber-700',
    cet6: 'bg-red-100 text-red-700',
  }
  return map[props.wordData.level] || 'bg-slate-100 text-slate-600'
})

const parsedDefinitions = computed(() => {
  try {
    const defs = JSON.parse(props.wordData.definition)
    return defs.map(d => ({
      pos: d.pos || '',
      meaning: d.meaning || ''
    }))
  } catch {
    return [{ pos: '', meaning: props.wordData.definition || '' }]
  }
})

defineEmits(['play-audio', 'mark-easy', 'mark-hard', 'generate-quiz', 'prev', 'next'])
</script>
