<template>
  <div class="space-y-6 p-2">
    <div v-if="loading" class="rounded-2xl bg-white py-12 text-center text-slate-500 shadow-sm ring-1 ring-slate-100">
      正在加载仪表盘...
    </div>

    <section class="grid gap-4 lg:grid-cols-[1.4fr_1fr]">
      <div class="rounded-2xl bg-white p-6 shadow-sm ring-1 ring-slate-100">
        <p class="text-sm font-semibold text-blue-600">{{ activeLevelLabel }} · 今日学习工作台</p>
        <h2 class="mt-2 text-2xl font-bold text-slate-900">先完成词卡，再进入测验</h2>
        <div class="mt-5 grid gap-3 sm:grid-cols-3">
          <div class="rounded-xl bg-blue-50 p-4">
            <p class="text-sm text-blue-700">当前到期</p>
            <p class="mt-1 text-2xl font-bold text-blue-900">{{ reviewForecast.due_now }}</p>
          </div>
          <div class="rounded-xl bg-amber-50 p-4">
            <p class="text-sm text-amber-700">模糊词</p>
            <p class="mt-1 text-2xl font-bold text-amber-900">{{ reviewForecast.fuzzy_words }}</p>
          </div>
          <div class="rounded-xl bg-red-50 p-4">
            <p class="text-sm text-red-700">困难词</p>
            <p class="mt-1 text-2xl font-bold text-red-900">{{ reviewForecast.hard_words }}</p>
          </div>
        </div>
        <div class="mt-5 flex flex-col gap-3 sm:flex-row">
          <router-link to="/words" class="rounded-xl bg-blue-600 px-5 py-3 text-center text-sm font-semibold text-white hover:bg-blue-700">开始今日词卡</router-link>
          <router-link to="/quiz" class="rounded-xl bg-slate-100 px-5 py-3 text-center text-sm font-semibold text-slate-700 hover:bg-slate-200">完成后去测验</router-link>
        </div>
      </div>

      <div class="rounded-2xl bg-white p-6 shadow-sm ring-1 ring-slate-100">
        <p class="text-sm font-semibold text-slate-500">明日计划摘要</p>
        <p class="mt-3 whitespace-pre-line text-sm leading-6 text-slate-700">{{ nextPlanSummary || '完成今日学习后会生成明日计划。' }}</p>
      </div>
    </section>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-2 gap-4 lg:grid-cols-6">
      <div v-for="stat in stats" :key="stat.label"
           class="bg-white rounded-2xl p-5 shadow-sm border border-slate-100 hover:shadow-md transition-shadow">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-slate-500">{{ stat.label }}</p>
            <p class="text-2xl font-bold text-slate-800 mt-1">{{ stat.value }}</p>
          </div>
          <div :class="stat.iconBg" class="w-12 h-12 rounded-xl flex items-center justify-center text-2xl">
            {{ stat.icon }}
          </div>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 gap-6">
      <!-- 学习进度环形图 -->
      <div class="bg-white rounded-2xl p-6 shadow-sm border border-slate-100">
        <h3 class="text-lg font-semibold text-slate-800 mb-4">📊 学习进度</h3>
        <div ref="progressChart" style="height: 280px;"></div>
      </div>
    </div>

    <!-- 学习趋势 -->
    <div class="bg-white rounded-2xl p-6 shadow-sm border border-slate-100">
      <h3 class="text-lg font-semibold text-slate-800 mb-4">📈 每日学习趋势</h3>
      <div ref="trendChart" style="height: 260px;"></div>
    </div>

    <!-- 错词Top10 -->
    <div class="bg-white rounded-2xl p-6 shadow-sm border border-slate-100">
      <h3 class="text-lg font-semibold text-slate-800 mb-4">⚠️ 易错词 Top 10</h3>
      <div ref="wrongChart" style="height: 320px;"></div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, nextTick, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import * as echarts from 'echarts'

const progressChart = ref(null)
const trendChart = ref(null)
const wrongChart = ref(null)
const loading = ref(true)
const activeLevelLabel = ref('当前学段')
const nextPlanSummary = ref('')
const reviewForecast = reactive({
  due_now: 0,
  due_tomorrow: 0,
  fuzzy_words: 0,
  hard_words: 0,
})

const stats = reactive([
  { label: '词库总量', value: '0', icon: '📚', iconBg: 'bg-slate-50' },
  { label: '已学单词', value: '0', icon: '📝', iconBg: 'bg-blue-50' },
  { label: '未学单词', value: '0', icon: '○', iconBg: 'bg-slate-50' },
  { label: '掌握率', value: '0%', icon: '✅', iconBg: 'bg-green-50' },
  { label: '连续学习', value: '0天', icon: '🔥', iconBg: 'bg-orange-50' },
  { label: '学习时长', value: '0h', icon: '⏱️', iconBg: 'bg-purple-50' },
])

onMounted(async () => {
  await nextTick()
  initCharts()
  loadData()
  window.addEventListener('app-config-updated', handleAppConfigUpdated)
})

onBeforeUnmount(() => {
  window.removeEventListener('app-config-updated', handleAppConfigUpdated)
})

function handleAppConfigUpdated() {
  loadData()
}

function initCharts() {
  // 1. 环形图
  const c1 = echarts.init(progressChart.value)
  c1.setOption({
    tooltip: { trigger: 'item', backgroundColor: '#1e293b', textStyle: { color: '#fff' } },
    series: [{
      type: 'pie', radius: ['55%', '80%'], center: ['50%', '50%'],
      label: { show: true, formatter: '{b}\n{c}词', fontSize: 13 },
      data: [
        { value: 0, name: '已掌握', itemStyle: { color: '#10b981' } },
        { value: 0, name: '学习中', itemStyle: { color: '#3b82f6' } },
        { value: 100, name: '未学习', itemStyle: { color: '#e2e8f0' } },
      ],
      emphasis: { scaleSize: 10 }
    }]
  })

  // 2. 折线图
  const c3 = echarts.init(trendChart.value)
  c3.setOption({
    tooltip: { trigger: 'axis', backgroundColor: '#1e293b', textStyle: { color: '#fff' } },
    grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
    xAxis: { type: 'category', data: ['暂无数据'], axisLabel: { color: '#94a3b8', fontSize: 11 } },
    yAxis: { type: 'value', name: '学习单词数', axisLabel: { color: '#94a3b8' } },
    series: [{
      type: 'line', smooth: true, symbol: 'circle', symbolSize: 6,
      lineStyle: { width: 3, color: '#6366f1' },
      areaStyle: { color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
        { offset: 0, color: 'rgba(99,102,241,0.3)' },
        { offset: 1, color: 'rgba(99,102,241,0.02)' }
      ])},
      data: [0]
    }]
  })

  // 3. 错词图
  const c4 = echarts.init(wrongChart.value)
  c4.setOption({
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: '20%', right: '5%', bottom: '3%', containLabel: true },
    xAxis: { type: 'value', axisLabel: { color: '#94a3b8' } },
    yAxis: { type: 'category', data: ['暂无数据'], axisLabel: { color: '#475569', fontSize: 12 } },
    series: [{
      type: 'bar', barWidth: 20,
      data: [0],
      itemStyle: { borderRadius: [0, 6, 6, 0], color: new echarts.graphic.LinearGradient(0, 0, 1, 0, [
        { offset: 0, color: '#fef3c7' }, { offset: 1, color: '#f59e0b' }
      ])},
      label: { show: true, position: 'right', formatter: '{c}次', color: '#92400e' }
    }]
  })

  window.addEventListener('resize', () => { c1.resize(); c3.resize(); c4.resize() })
}

async function loadData() {
  loading.value = true
  try {
    const [data, config] = await Promise.all([
      invoke('get_dashboard_data'),
      invoke('get_config'),
    ])
    activeLevelLabel.value = levelLabel(config.active_level || data.active_level)
    await loadReviewForecast(config.active_level || data.active_level)
    await loadNextPlan(config.active_level || data.active_level)
    stats[0].value = data.total_words
    stats[1].value = data.total_learned
    stats[2].value = data.unlearned_words
    stats[3].value = data.mastery_rate + '%'
    stats[4].value = data.streak_days
    stats[5].value = data.total_time

    // Update progress chart
    const c1 = echarts.getInstanceByDom(progressChart.value)
    if (c1) {
      const progress = Array.isArray(data.progress) ? data.progress : []
      c1.setOption({
        series: [{ data: progress.map(item => ({
          value: item.value,
          name: item.name,
          itemStyle: { color: item.color },
        }))}]
      })
    }

    // Update trend chart
    const c3 = echarts.getInstanceByDom(trendChart.value)
    if (c3 && data.daily_trend.length > 0) {
      c3.setOption({
        xAxis: { data: data.daily_trend.map(d => d.date) },
        series: [{ data: data.daily_trend.map(d => d.count) }]
      })
    }

    // Update wrong words chart
    const c4 = echarts.getInstanceByDom(wrongChart.value)
    if (c4 && data.wrong_words.length > 0) {
      c4.setOption({
        yAxis: { data: data.wrong_words.map(w => w.word) },
        series: [{ data: data.wrong_words.map(w => w.count) }]
      })
    }
  } catch (e) {
    console.error('加载仪表盘数据失败:', e)
  }
  loading.value = false
}

async function loadReviewForecast(level) {
  try {
    const forecast = await invoke('get_review_forecast', { level })
    reviewForecast.due_now = Number(forecast.due_now || 0)
    reviewForecast.due_tomorrow = Number(forecast.due_tomorrow || 0)
    reviewForecast.fuzzy_words = Number(forecast.fuzzy_words || 0)
    reviewForecast.hard_words = Number(forecast.hard_words || 0)
  } catch {
    reviewForecast.due_now = 0
    reviewForecast.due_tomorrow = 0
    reviewForecast.fuzzy_words = 0
    reviewForecast.hard_words = 0
  }
}

async function loadNextPlan(level) {
  try {
    const plan = await invoke('get_next_plan')
    nextPlanSummary.value = plan?.level === level ? String(plan.content || '') : ''
  } catch {
    nextPlanSummary.value = ''
  }
}

function levelLabel(level) {
  const labels = {
    primary: '小学',
    junior: '初中',
    high: '高中',
    cet4: '四级',
    cet6: '六级',
  }
  return labels[level] || level || '当前学段'
}
</script>
