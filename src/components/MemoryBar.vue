<script setup lang="ts">
import { useConverter } from "../composables/useConverter";

const { memoryPath, memoryError, revealMemoryFile } = useConverter();
</script>

<template>
  <div class="rounded-lg border border-gray-700 bg-gray-800/60 px-4 py-2.5 text-sm">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="min-w-0">
        <p class="text-gray-400">Settings are remembered in</p>
        <p
          v-if="memoryPath"
          class="font-mono text-xs text-gray-200 truncate select-all"
          :title="memoryPath"
        >
          {{ memoryPath }}
        </p>
        <p v-else class="text-gray-500">Locating memory file…</p>
      </div>

      <button
        v-if="memoryPath"
        @click="revealMemoryFile"
        class="shrink-0 px-3 py-1 rounded-md border border-gray-600 text-gray-200 transition-colors hover:bg-gray-700"
      >
        Show in folder
      </button>
    </div>

    <p v-if="memoryError" class="text-red-400 mt-2">{{ memoryError }}</p>
  </div>
</template>
