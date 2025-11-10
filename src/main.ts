import { createApp } from "vue";
import { createWebHistory, createRouter, Router } from "vue-router";

import App from "./App.vue";

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

createApp(App).use(router).mount("#app");
