import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'login',
      component: () => import('../views/LoginView.vue'),
    },
    {
      path: '/list',
      name: 'list',
      component: () => import('../views/PasswordListView.vue'),
    },
    {
      path: '/entry/:id',
      name: 'detail',
      component: () => import('../views/PasswordDetailView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsView.vue'),
    },
  ],
})

export default router
