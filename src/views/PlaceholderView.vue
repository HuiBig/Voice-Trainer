<script setup lang="ts">
defineProps<{
  title: string
  description: string
}>()
</script>

<template>
  <section class="placeholder-view">
    <span class="eyebrow">Voice Trainer</span>
    <h1>{{ title }}</h1>
    <p>{{ description }}</p>
    <div class="placeholder-line" />
  </section>
</template>
