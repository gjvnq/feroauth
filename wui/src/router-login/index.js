import Vue from 'vue'
import VueRouter from 'vue-router'
import Login from '../views/Login.vue'
import Error404 from '../views/Error404.vue'

Vue.use(VueRouter)

const routes = [
  {
    path: '/login',
    name: 'Login',
    component: Login
  },
  {
    path: '/login/about123',
    name: 'About',
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () => import(/* webpackChunkName: "about" */ '../views/About.vue')
  },
  { path: '*', component: Error404 }
]

const router = new VueRouter({
  mode: 'history',
  routes: routes
})

export default router;