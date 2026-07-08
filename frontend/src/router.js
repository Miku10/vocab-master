import { createRouter, createWebBrowserHistory } from 'vue-router'
import DashboardPage from './pages/DashboardPage.vue'
import WordsPage from './pages/WordsPage.vue'
import ReviewPage from './pages/ReviewPage.vue'
import QuizPage from './pages/QuizPage.vue'

const router = createRouter({
  history: createWebBrowserHistory(),
  routes: [
    { path: '/', component: DashboardPage },
    { path: '/words', component: WordsPage },
    { path: '/review', component: ReviewPage },
    { path: '/quiz', component: QuizPage },
  ]
})

export default router
