import { ref, computed ,watch } from 'vue';


export type Theme = 'focus-light' | 'focus-dark';

const currentTheme = ref<Theme>('focus-light');

export function useTheme() {
  const setTheme = (theme: Theme) => {
    currentTheme.value = theme;
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', theme);
    }
  };

  const toggleTheme = () => {
    setTheme(currentTheme.value === 'focus-light' ? 'focus-dark' : 'focus-light');
  };

  const isLight = computed(() => currentTheme.value === 'focus-light');
  const isDark = computed(() => currentTheme.value === 'focus-dark');

  const dark = computed(() => window.matchMedia("(prefers-color-scheme: dark)").matches);

  watch(dark, (value) => {
    if (value) {
      console.log(value);
      setTheme("focus-dark");
    }
    else {
      setTheme("focus-light");
    }
  })

  return {
    currentTheme: computed(() => currentTheme.value),
    setTheme,
    toggleTheme,
    isLight,
    isDark,
    watch
  };
}
