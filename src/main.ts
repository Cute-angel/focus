import { createApp } from "vue";
import { createWebHistory, createRouter, Router } from "vue-router";

import App from "./App.vue";
import { useTheme } from './composables/useTheme';

const routes = [
  {
    path: "/settings",
    component: () => import("./pages/SettingPage.vue"),
  },
  {
    path: "/query",
    component: () => import("./pages/HomePage.vue"),
  },
  {
    path: "/",
    component: () => import("./pages/QueryPage.vue"),
  },
];

const router: Router = createRouter({
  history: createWebHistory(),
  routes,
});

const app = createApp(App);
app.use(router);

// 初始化主题
useTheme();
// 默认亮色主题

app.mount("#app");
