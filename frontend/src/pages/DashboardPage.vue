<template>
  <div class="space-y-6 p-2">
    <!-- 统计卡片 -->
    <div class="grid grid-cols-4 gap-4">
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
import { ref, onMounted, nextTick, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import * as echarts from 'echarts'

const progressChart = ref(null)
const trendChart = ref(null)
const wrongChart = ref(null)

const stats = reactive([
  { label: '已学单词', value: '0', icon: '📝', iconBg: 'bg-blue-50' },
  { label: '掌握率', value: '0%', icon: '✅', iconBg: 'bg-green-50' },
  { label: '连续学习', value: '0天', icon: '🔥', iconBg: 'bg-orange-50' },
  { label: '学习时长', value: '0h', icon: '⏱️', iconBg: 'bg-purple-50' },
])

onMounted(async () => {
  await nextTick()
  initCharts()
  loadData()
})

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
  try {
    const data = await invoke('get_dashboard_data')
    stats[0].value = data.total_learned
    stats[1].value = data.mastery_rate + '%'
    stats[2].value = data.streak_days
    stats[3].value = data.total_time

    // Update progress chart
    const c1 = echarts.getInstanceByDom(progressChart.value)
    if (c1) {
      const learned = parseInt(data.total_learned) || 0
      const rate = parseFloat(data.mastery_rate) || 0
      c1.setOption({
        series: [{ data: [
          { value: Math.round(learned * rate / 100), name: '已掌握', itemStyle: { color: '#10b981' } },
          { value: learned - Math.round(learned * rate / 100), name: '学习中', itemStyle: { color: '#3b82f6' } },
          { value: Math.max(0, 100 - learned), name: '未学习', itemStyle: { color: '#e2e8f0' } },
        ]}]
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
}
</script>
