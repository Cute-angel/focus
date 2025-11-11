<template>
    <div class="flex flex-row hover:glass hover:shadow hover:rounded-field pt-0.5"
        :class="{ 'bg-gray-200/30 shadow rounded-field': isSelect }">
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"
            class="size-8 my-auto">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
        </svg>

        <br>
        <div class="mx-2 flex-1 truncate">
            <h3 class="text-base-content text-2xl leading-none">{{ title }}</h3>
            <p class="text-base-content/60">{{ description }}</p>
        </div>
        <div class=" ml-auto">
            <actions-box :-action-list="actions" :-select-action="selectedAction" v-if="isShow" />
        </div>

    </div>
</template>

<script setup lang="ts">

import { computed, ComputedRef } from 'vue';
import ActionsBox from './ActionsBox.vue';
import { type Action } from './ActionsBox.vue'


const props = defineProps<{
    icon: String,
    title: String,
    description: String,
    actions: Array<Action>,
    isSelect: Boolean,
    selectedAction: Number
}>()

const isShow: ComputedRef<Boolean> = computed(() => {
    console.log(props.actions)
    if (props.actions.length === 1 && props.actions[0].icon === "hide") {
        return false
    } else {
        return props.isSelect
    }
})


</script>

<style scoped></style>