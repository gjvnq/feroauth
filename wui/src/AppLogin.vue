<template>
  <div id="app">
    <router-view/>
  </div>
</template>

<style lang="scss">
@import '@/global-style.scss';
html, body {
  background-color: $gray-100 !important;
  height: 100%;
}

div#app {
  margin-top: 5rem;
  display: flex;
  justify-content: space-evenly;
}
</style>