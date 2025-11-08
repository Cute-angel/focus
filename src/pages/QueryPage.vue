<template>
    <div @keydown="handleKeydown" tabindex="0" class="focus-visible:outline-none" ref="mainPage">
        <query-box class="w-full box-border sticky top-0 z-10 shadow-lg" v-model:cursorPos="cursorPos"
            v-model:query="inputText"></query-box>

        <div class="mt-2 overflow-y-auto max-h-[300px] " v-show="hasResults">
            <ul class="flex flex-col box-border" ref="scrollContainer">
                <li class=" w-full flex-1 " v-for="(result, index) in results">
                    <ResultItem :key="index" :icon="result.icon" :title="result.title" :description="result.description"
                        :actions="result.actions" :is-select="selectedIndex === index" :selected-action="selectedAction"
                        class=" mx-4 mb-2 mt-1" />
                </li>
            </ul>
        </div>


    </div>
</template>

<script setup lang="ts">
import QueryBox from '../components/QueryBox.vue';
import ResultItem from '../components/ResultItem.vue';

import { computed, ComputedRef, onMounted, ref, shallowRef, watch } from 'vue';
import { type Action } from '../components/ActionsBox.vue';
import { useRunAction } from '../api';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalSize } from '@tauri-apps/api/dpi';
import { invoke } from '@tauri-apps/api/core';

interface Result {
    icon: string;
    title: string;
    description: string;
    actions: Array<Action>;
}

const selectedIndex = ref(-1);
const selectedAction = ref(-1);
const scrollContainer = ref<HTMLElement | null>(null);
const cursorPos = ref<Number>();
const inputText = ref<String>("");
const mainPage = ref<HTMLElement | null>(null);


const isAtEnd: ComputedRef<boolean> = computed(() => {
    if (cursorPos.value === inputText.value.length) {
        return true;
    } else {
        return false;
    }
})

const hasResults: ComputedRef<boolean> = computed(() => {
    return results.value.length > 0;
})

const appWindow = getCurrentWindow();

// Handle keyboard events (up/down arrows)
const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Enter') {
        if (selectedIndex.value != -1) {
            const _lt: Array<Action> = results.value[selectedIndex.value].actions;
            useRunAction(_lt[selectedAction.value === -1 ? 0 : selectedAction.value].id);
        }
    }

    if (event.key === 'ArrowDown') {
        // Move down the list
        if (selectedIndex.value < results.value.length - 1) {
            selectedIndex.value++;

        } else {
            selectedIndex.value = 0;
        }
        selectedAction.value = -1;
    } else if (event.key === 'ArrowUp') {

        // Move up the list
        if (selectedIndex.value > 0) {
            selectedIndex.value--;

        } else {
            selectedIndex.value = results.value.length - 1;
        }
        selectedAction.value = -1;
    }

    if (selectedAction.value >= 0) {
        // 只阻止一些特定按键，避免阻止所有输入
        if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
            event.preventDefault();
        }
    }

    const container = scrollContainer.value;
    const selectedItem = container?.children[selectedIndex.value] as HTMLElement;
    selectedItem?.scrollIntoView({
        behavior: 'smooth',
        block: 'start' // Align the element at the top of the scroll container
    });

    if (event.key === 'ArrowLeft') {
        if (selectedAction.value >= 0) selectedAction.value--;


    }
    else if (event.key === 'ArrowRight') {
        if (selectedAction.value < results.value[selectedIndex.value].actions.length - 1 && isAtEnd.value) {
            selectedAction.value++;
        }

    }
};


const autoResizeWithObserver = (el: HTMLElement) => {
    const observer = new ResizeObserver(async () => {
        const reac = el.getBoundingClientRect()
        await appWindow.setSize(new LogicalSize(
            Math.ceil(reac.width),
            Math.ceil(reac.height)
        ))
    })

    observer.observe(el)
}

onMounted(() => {
    if (mainPage.value) autoResizeWithObserver(mainPage.value)
})

watch(inputText, () => {
    selectedAction.value = -1;

    invoke("query", { inputText: inputText.value }).then((res: any) => {
        console.log(res);
        results.value = res.items as Array<Result>;
        selectedIndex.value = results.value.length > 0 ? 0 : -1;
    })

})

const svg = shallowRef(`<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2.5"
                   stroke="currentColor" class="size-[1.2em]">
                    <path stroke-linecap="round" stroke-linejoin="round"
                        d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12Z" />
                </svg>`)


const results = ref<Array<Result>>([]);

const _results = ref<Array<Result>>([
    {
        icon: 'document-icon',
        title: 'Result Title 1 j q l',
        description: 'This is a description for result 1.',
        actions: [{
            id: "open 1",
            tooltip: "open 1",
            icon: svg.value
        },
        {
            id: "open 2",
            tooltip: "open 2",
            icon: svg.value
        }, {
            id: "open 3",
            tooltip: "open 3",
            icon: svg.value
        }, {
            id: "open 4",
            tooltip: "open 4",
            icon: svg.value
        }]

    },
    {
        icon: 'document-icon',
        title: 'Result Title 1 j q l',
        description: 'This is a description for result 1. test long long long long long long long long long text',
        actions: [{
            id: "open 3",
            tooltip: "open file 3",
            icon: svg.value
        },
        {
            id: "open 4",
            tooltip: "open file 4",
            icon: svg.value
        }]

    },
    {
        icon: 'document-icon',
        title: 'Result Title 1 j q l',
        description: 'This is a description for result 1. test long long long long long long long long long text',
        actions: [{
            id: "open",
            tooltip: "open file",
            icon: svg.value
        },
        {
            id: "open",
            tooltip: "open file",
            icon: svg.value
        }]

    },





]);

</script>

<style lang="css" scoped>
:root {
    background-color: transparent;
}

@layer utilities {
    .mask-box {
        position: sticky;
        top: 0;
        z-index: 20;
        background: transparent;
        -webkit-mask-image: linear-gradient(to bottom, rgba(255, 255, 255, 1) 0%, rgba(255, 255, 255, 0) 100%);
        mask-image: linear-gradient(to bottom, rgba(255, 255, 255, 1) 0%, rgba(255, 255, 255, 0) 100%);
    }
}
</style>