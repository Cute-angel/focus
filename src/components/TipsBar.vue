<script setup lang="ts">
interface Tips {
  keys:string[]
  name:string
}


const prop = defineProps<{
  value:Tips[]
}>()


</script>

<template>
  <div class="w-full flex gap-4 py-4">
    <span class="flex gap-1 text-xl" v-for="tips in prop.value">
      {{tips.name}}
      <div v-for="key in tips.keys" class="flex justify-center" >
        <div class="text-sm rounded-lg outline bg-base-content/500 text-justify">
          {{key.toUpperCase()}}
        </div>
      </div>
    </span>
  </div>
</template>

<style scoped>
</style>