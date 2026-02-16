<template>
  <div class="container mx-auto p-8 max-w-4xl">
    <h1 class="text-3xl font-bold mb-8 text-base-content">Settings</h1>

    <section class="mb-8">
      <h2 class="text-xl font-semibold mb-4 text-base-content">Appearance</h2>

      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
          <h3 class="card-title">Theme</h3>
          <p class="text-base-content/70 mb-4">
            Choose your preferred color scheme for Focus
          </p>

          <div class="grid grid-cols-2 gap-4">
            <!-- 亮色主题选项 -->
            <div
              class="cursor-pointer border-2 rounded-lg p-4 transition-all"
              :class="isLight ? 'border-primary bg-base-200' : 'border-transparent hover:border-base-300'"
              @click="selectTheme('focus-light')"
            >
              <div class="flex items-center gap-3 mb-3">
                <div class="w-8 h-8 rounded-full bg-gradient-to-br from-yellow-200 to-orange-200"></div>
                <span class="font-semibold">Light</span>
              </div>
              <p class="text-sm text-base-content/60">Bright and clean, ideal for daytime use</p>
            </div>

            <!-- 暗色主题选项 -->
            <div
              class="cursor-pointer border-2 rounded-lg p-4 transition-all"
              :class="isDark ? 'border-primary bg-base-200' : 'border-transparent hover:border-base-300'"
              @click="selectTheme('focus-dark')"
            >
              <div class="flex items-center gap-3 mb-3">
                <div class="w-8 h-8 rounded-full bg-gradient-to-br from-gray-700 to-gray-900"></div>
                <span class="font-semibold">Dark</span>
              </div>
              <p class="text-sm text-base-content/60">Easy on the eyes, perfect for night use</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 预览部分 -->
    <section class="mb-8">
      <h2 class="text-xl font-semibold mb-4 text-base-content">Preview</h2>
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
          <h3 class="card-title">Sample UI Elements</h3>

          <!-- 示例结果项 -->
          <div class="flex flex-row hover:glass hover:shadow hover:rounded-field py-2 mb-2 transition-all">
            <div class="w-8 h-8 bg-primary rounded flex items-center justify-center mr-3">
              <span class="text-primary-content text-sm">📄</span>
            </div>
            <div class="flex-1">
              <h4 class="text-base-content text-lg">Sample File</h4>
              <p class="text-base-content/60 text-sm">/path/to/sample/file.txt</p>
            </div>
            <div class="flex gap-2">
              <button class="btn btn-square btn-outline btn-sm">⚡</button>
            </div>
          </div>

          <!-- 示例按钮 -->
          <div class="flex gap-2 mt-4">
            <button class="btn btn-primary">Primary</button>
            <button class="btn btn-secondary">Secondary</button>
            <button class="btn btn-outline">Outline</button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { useTheme, type Theme } from '../composables/useTheme';

const { setTheme, isLight, isDark } = useTheme();

const selectTheme = (theme: Theme) => {
  setTheme(theme);
};
</script>

<style scoped>
.card {
  transition: transform 0.2s, box-shadow 0.2s;
}

.card:hover {
  transform: translateY(-2px);
}
</style>