<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { PlayIcon } from "@heroicons/vue/24/solid";
import { getStartupStatus, saveGlobalHotkey } from "../api/startup";

const DEFAULT_ACCELERATOR = "Ctrl+N";
const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);

const isReady = ref(false);
const isRecording = ref(false);
const isSaving = ref(false);
const errorMessage = ref("");
const successMessage = ref("");
const accelerator = ref("");
const currentStep = ref<1 | 2>(1);
const isBrowserPreview =
  import.meta.env.DEV &&
  typeof window !== "undefined" &&
  !("__TAURI_INTERNALS__" in window);

const resolvedAccelerator = computed(() => accelerator.value || DEFAULT_ACCELERATOR);
const acceleratorSegments = computed(() =>
  resolvedAccelerator.value ? resolvedAccelerator.value.split("+") : [],
);

const canSave = computed(() => acceleratorSegments.value.length > 1 && !isSaving.value);

const helpText = computed(() => {
  if (errorMessage.value) return errorMessage.value;
  if (successMessage.value) return successMessage.value;
  if (isRecording.value) return "按下修饰键和一个主键。仅修饰键不能作为全局快捷键。";
  if (accelerator.value) return "这个快捷键会在桌面任意位置唤起 Focus。";
  return "默认快捷键已经准备好。你也可以点击下方区域重新录制。";
});

const panelClass =
  "w-full max-w-[1120px] border border-white/12 bg-[linear-gradient(180deg,rgba(133,145,175,0.14),rgba(105,113,145,0.18))] shadow-[inset_0_1px_0_rgba(255,255,255,0.08),0_16px_42px_rgba(18,24,40,0.14)] backdrop-blur-[10px]";

const focusRingClass =
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[rgba(129,214,255,0.82)] focus-visible:ring-offset-0";

function formatDisplayLabel(segment: string) {
  if (segment.length === 1) return segment.toUpperCase();
  if (segment.startsWith("F")) return segment.toUpperCase();
  return segment;
}

function normalizeCode(code: string, key: string) {
  if (code.startsWith("Key")) return code.slice(3).toUpperCase();
  if (code.startsWith("Digit")) return code.slice(5);
  if (code.startsWith("Numpad")) return code;
  if (/^F\d{1,2}$/i.test(code)) return code.toUpperCase();

  const codeMap: Record<string, string> = {
    Space: "Space",
    Enter: "Enter",
    Tab: "Tab",
    Escape: "Escape",
    Backspace: "Backspace",
    Delete: "Delete",
    Insert: "Insert",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    ArrowUp: "ArrowUp",
    ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
    Minus: "Minus",
    Equal: "Equal",
    BracketLeft: "BracketLeft",
    BracketRight: "BracketRight",
    Backslash: "Backslash",
    Semicolon: "Semicolon",
    Quote: "Quote",
    Backquote: "Backquote",
    Comma: "Comma",
    Period: "Period",
    Slash: "Slash",
  };

  if (codeMap[code]) return codeMap[code];
  if (key === " ") return "Space";
  return key.length === 1 ? key.toUpperCase() : "";
}

function setAccelerator(value: string) {
  accelerator.value = value;
  errorMessage.value = "";
  successMessage.value = "";
}

function goToShortcutStep() {
  currentStep.value = 2;
  isRecording.value = false;
  errorMessage.value = "";
  successMessage.value = "";
}

function goToWelcomeStep() {
  currentStep.value = 1;
  isRecording.value = false;
  errorMessage.value = "";
  successMessage.value = "";
}

function beginRecording() {
  isRecording.value = true;
  errorMessage.value = "";
  successMessage.value = "";
}

function stopRecording() {
  isRecording.value = false;
}

function handleShortcutCapture(event: KeyboardEvent) {
  if (!isRecording.value || currentStep.value !== 2) return;

  event.preventDefault();
  event.stopPropagation();

  if (event.key === "Escape") {
    stopRecording();
    return;
  }

  if (MODIFIER_KEYS.has(event.key)) {
    return;
  }

  const mainKey = normalizeCode(event.code, event.key);
  if (!mainKey) {
    errorMessage.value = "这个按键暂不支持作为全局快捷键。";
    return;
  }

  const segments: string[] = [];
  if (event.ctrlKey) segments.push("Ctrl");
  if (event.shiftKey) segments.push("Shift");
  if (event.altKey) segments.push("Alt");
  if (event.metaKey) segments.push("Super");
  segments.push(mainKey);

  if (segments.length < 2) {
    errorMessage.value = "至少需要一个修饰键和一个主键。";
    return;
  }

  setAccelerator(segments.join("+"));
  stopRecording();
}

function useDefaultShortcut() {
  setAccelerator(DEFAULT_ACCELERATOR);
  stopRecording();
}

async function submitHotkey() {
  if (!canSave.value) return;

  isSaving.value = true;
  errorMessage.value = "";
  successMessage.value = "";

  try {
    const result = await saveGlobalHotkey(resolvedAccelerator.value);
    accelerator.value = result.hotkey;
    successMessage.value = `已保存 ${result.hotkey.split("+").join(" + ")}，正在进入 Focus…`;
    window.setTimeout(() => {
      void getCurrentWebviewWindow().close();
    }, 500);
  } catch (error) {
    errorMessage.value =
      typeof error === "string" ? error : "快捷键注册失败，请尝试换一个组合。";
  } finally {
    isSaving.value = false;
  }
}

onMounted(async () => {
  try {
    const status = await getStartupStatus();
    accelerator.value = status.currentHotkey || DEFAULT_ACCELERATOR;
  } catch {
    accelerator.value = DEFAULT_ACCELERATOR;
  } finally {
    if (!accelerator.value) accelerator.value = DEFAULT_ACCELERATOR;
    isReady.value = true;
  }
});
</script>

<template>
  <main
    class="relative min-h-screen overflow-hidden bg-transparent text-[rgba(248,250,255,0.96)]"
    @keydown="handleShortcutCapture"
  >
    <div
      v-if="isBrowserPreview"
      class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_18%_18%,rgba(210,217,231,0.56),transparent_28%),radial-gradient(circle_at_78%_12%,rgba(111,120,170,0.28),transparent_24%),radial-gradient(circle_at_62%_72%,rgba(183,167,184,0.2),transparent_26%),linear-gradient(135deg,#808896_0%,#676d81_35%,#595a72_100%)]"
      aria-hidden="true"
    ></div>
    <div
      v-if="isBrowserPreview"
      class="pointer-events-none absolute inset-0 bg-[linear-gradient(180deg,rgba(255,255,255,0.08),transparent_26%),linear-gradient(135deg,rgba(255,255,255,0.05),transparent_42%)]"
      aria-hidden="true"
    ></div>

    <section
      class="relative z-10 flex min-h-screen flex-col justify-between px-4 pb-6 pt-5 transition-all duration-500 ease-out sm:px-4 md:px-4"
      :class="isReady ? 'translate-y-0 opacity-100' : 'translate-y-3 opacity-0'"
    >
      <header class="flex items-start justify-between gap-4 px-1 pt-[0.15rem]">
        <div class="flex flex-col">
          <p class="m-0 text-[0.78rem] font-bold uppercase tracking-[0.32em] text-white/92">
            Focus
          </p>
          <p class="mt-[0.45rem] text-[0.9rem] text-[rgba(232,238,250,0.74)]">
            首次启动引导
          </p>
        </div>

        <div class="flex items-center gap-[0.65rem] pt-[0.2rem]" aria-label="Onboarding progress">
          <span
            class="h-[0.58rem] w-[0.58rem] rounded-full border transition-all duration-200"
            :class="
              currentStep === 1 || currentStep === 2
                ? currentStep === 1
                  ? 'scale-[1.08] border-[rgba(111,202,255,0.72)] bg-[rgba(111,202,255,0.8)]'
                  : 'border-[rgba(111,202,255,0.72)] bg-[rgba(111,202,255,0.8)]'
                : 'border-white/20 bg-white/6'
            "
          ></span>
          <span class="block h-px w-[2.6rem] bg-white/14"></span>
          <span
            class="h-[0.58rem] w-[0.58rem] rounded-full border transition-all duration-200"
            :class="
              currentStep === 2
                ? 'scale-[1.08] border-[rgba(111,202,255,0.72)] bg-[rgba(111,202,255,0.8)]'
                : 'border-white/20 bg-white/6'
            "
          ></span>
        </div>
      </header>

      <Transition
        mode="out-in"
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="translate-x-[18px] opacity-0"
        enter-to-class="translate-x-0 opacity-100"
        leave-active-class="transition duration-300 ease-out"
        leave-from-class="translate-x-0 opacity-100"
        leave-to-class="-translate-x-[18px] opacity-0"
      >
        <section
          v-if="currentStep === 1"
          key="welcome"
          class="flex min-h-0 flex-1 flex-col items-center justify-center gap-8"
        >
          <div class="mt-auto max-w-[44rem] text-center">
            <p
              class="mb-4 text-[0.82rem] font-bold uppercase tracking-[0.24em] text-[rgba(236,241,250,0.72)]"
            >
              欢迎使用
            </p>
            <h1
              class="m-0 text-[clamp(2.4rem,10vw,4.4rem)] font-bold leading-[0.96] tracking-[-0.035em] text-white/98"
            >
              Focus
            </h1>
            <p
              class="mx-auto mt-4 max-w-[46rem] text-[1.12rem] leading-[1.6] text-[rgba(246,248,255,0.9)]"
            >
              用一个快捷键，快速唤起搜索、指令和常用动作，把注意力留在当下的工作里。
            </p>
            <p
              class="mx-auto mt-4 max-w-[42rem] text-base leading-[1.6] text-[rgba(236,241,250,0.8)]"
            >
              这个引导只有两步。接下来先设置一个你愿意长期记住的全局快捷键。
            </p>
          </div>

          <div :class="[panelClass, 'mt-auto flex justify-center rounded-2xl px-4 py-[1.9rem]']">
            <div class="flex flex-col items-start gap-4 sm:flex-row sm:items-center">
              <button
                type="button"
                :class="[
                  focusRingClass,
                  'inline-flex h-[3.9rem] w-[3.9rem] items-center justify-center rounded-full border border-[rgba(108,205,255,0.68)] bg-[linear-gradient(180deg,rgba(113,205,255,0.95),rgba(84,180,244,0.88))] text-white/98 shadow-[inset_0_1px_0_rgba(255,255,255,0.34),0_12px_28px_rgba(84,180,244,0.24)] transition-all duration-200 hover:translate-x-[3px] hover:border-[rgba(141,220,255,0.92)] hover:bg-[linear-gradient(180deg,rgba(127,214,255,1),rgba(93,189,249,0.92))]'
                ]"
                aria-label="继续"
                @click="goToShortcutStep"
              >
                <PlayIcon class="size-6" />
              </button>
              <p class="m-0 text-[0.95rem] text-[rgba(240,244,252,0.82)]">
                继续设置快捷键
              </p>
            </div>
          </div>
        </section>

        <section
          v-else
          key="shortcut"
          class="flex min-h-0 flex-1 flex-col items-center justify-center gap-[1.35rem]"
        >
          <div class="max-w-[44rem] text-center">
            <p
              class="mb-4 text-[0.82rem] font-bold uppercase tracking-[0.24em] text-[rgba(236,241,250,0.72)]"
            >
              步骤 2 / 2
            </p>
            <h2
              class="m-0 text-[clamp(1.8rem,5vw,2.7rem)] font-bold leading-[0.96] tracking-[-0.035em] text-white/98"
            >
              设置唤起快捷键
            </h2>
            <p
              class="mx-auto mt-4 max-w-[42rem] text-base leading-[1.6] text-[rgba(236,241,250,0.8)]"
            >
              默认快捷键已经准备好。你可以直接完成引导，也可以改成更符合自己习惯的组合。
            </p>
          </div>

          <div
            :class="[
              panelClass,
              'rounded-[0.9rem] p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.08),0_18px_40px_rgba(18,24,40,0.16)]'
            ]"
          >
            <div class="flex flex-col items-start justify-between gap-4 sm:flex-row">
              <div>
                <p
                  class="m-0 text-[0.78rem] font-bold uppercase tracking-[0.18em] text-[rgba(235,240,248,0.72)]"
                >
                  全局快捷键
                </p>
                <p class="mt-[0.45rem] text-[0.92rem] text-[rgba(232,238,250,0.72)]">
                  设置后可在系统任意位置唤起 Focus
                </p>
              </div>

              <button
                type="button"
                :class="[
                  focusRingClass,
                  'bg-transparent p-0 text-[0.92rem] text-[rgba(235,242,252,0.82)] transition-all duration-200 hover:-translate-x-[2px] hover:text-[rgba(129,214,255,0.96)]'
                ]"
                @click="goToWelcomeStep"
              >
                返回上一步
              </button>
            </div>

            <button
              type="button"
              :class="[
                focusRingClass,
                'mt-4 flex min-h-[7.4rem] w-full items-center justify-center rounded-[0.8rem] border bg-white/[0.045] px-5 py-5 transition-all duration-200 hover:-translate-y-px sm:min-h-[8.4rem]',
                isRecording
                  ? 'border-[rgba(112,202,255,0.86)] bg-[rgba(101,170,220,0.1)] shadow-[0_0_0_1px_rgba(112,202,255,0.22),0_0_0_6px_rgba(112,202,255,0.08)]'
                  : acceleratorSegments.length > 1
                    ? 'border-[rgba(112,202,255,0.36)]'
                    : 'border-white/13'
              ]"
              @click="beginRecording"
            >
              <span class="flex flex-wrap justify-center gap-[0.7rem]">
                <span
                  v-for="segment in acceleratorSegments"
                  :key="segment"
                  class="min-w-16 rounded-[0.8rem] border border-white/12 bg-[linear-gradient(180deg,rgba(255,255,255,0.14),rgba(255,255,255,0.08))] px-4 py-[0.95rem] text-center text-base font-bold tracking-[0.02em] text-white/95 shadow-[inset_0_-1px_0_rgba(255,255,255,0.08),0_10px_22px_rgba(12,18,30,0.08)]"
                >
                  {{ formatDisplayLabel(segment) }}
                </span>
              </span>
            </button>

            <p
              class="mt-[0.95rem] min-h-[2.8rem] text-[0.92rem] leading-[1.5]"
              :class="
                errorMessage
                  ? 'text-[rgba(255,166,166,0.96)]'
                  : successMessage
                    ? 'text-[rgba(171,241,200,0.96)]'
                    : 'text-[rgba(233,238,248,0.72)]'
              "
            >
              {{ helpText }}
            </p>

            <div class="flex flex-wrap items-center gap-[0.85rem]">
              <button
                type="button"
                :class="[
                  focusRingClass,
                  'w-full rounded-[0.6rem] bg-[linear-gradient(180deg,rgba(110,203,255,0.96),rgba(81,180,244,0.9))] px-[1.4rem] py-[0.85rem] text-[0.95rem] font-bold text-white/98 transition-all duration-200 hover:-translate-y-px hover:shadow-[0_12px_24px_rgba(84,180,244,0.22)] sm:w-auto',
                  !canSave ? 'cursor-not-allowed opacity-55 hover:translate-y-0 hover:shadow-none' : ''
                ]"
                :disabled="!canSave"
                @click="submitHotkey"
              >
                {{ isSaving ? "正在保存..." : "完成引导并进入 Focus" }}
              </button>

              <button
                type="button"
                :class="[
                  focusRingClass,
                  'bg-transparent py-[0.2rem] text-[0.92rem] font-semibold text-[rgba(235,241,250,0.82)] transition-all duration-200 hover:translate-x-[2px] hover:text-[rgba(129,214,255,0.96)]'
                ]"
                @click="beginRecording"
              >
                {{ isRecording ? "等待按键输入..." : "重新录制快捷键" }}
              </button>

              <button
                type="button"
                :class="[
                  focusRingClass,
                  'bg-transparent py-[0.2rem] text-[0.92rem] font-semibold text-[rgba(235,241,250,0.82)] transition-all duration-200 hover:translate-x-[2px] hover:text-[rgba(129,214,255,0.96)]'
                ]"
                @click="useDefaultShortcut"
              >
                使用默认 {{ DEFAULT_ACCELERATOR.replace("+", " + ") }}
              </button>
            </div>

            <div class="mt-[1.1rem] flex flex-wrap gap-[0.55rem]">
              <span
                class="rounded-full border border-white/10 bg-white/[0.04] px-[0.72rem] py-[0.38rem] text-[0.8rem] text-[rgba(232,237,247,0.7)]"
              >
                Esc 可取消录制
              </span>
              <span
                class="rounded-full border border-white/10 bg-white/[0.04] px-[0.72rem] py-[0.38rem] text-[0.8rem] text-[rgba(232,237,247,0.7)]"
              >
                至少需要一个修饰键
              </span>
            </div>
          </div>
        </section>
      </Transition>
    </section>
  </main>
</template>
