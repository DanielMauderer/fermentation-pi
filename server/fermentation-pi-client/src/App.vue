<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
const delay = (ms: number) => new Promise(res => setTimeout(res, ms));

async function elementGotClicked(event: Event) {
  await delay(50);
  setElementClass((event.target as HTMLElement).parentElement?.previousElementSibling, "prev")
  setElementClass((event.target as HTMLElement).parentElement?.nextElementSibling, "next")
  setElementClass((event.target as HTMLElement).parentElement, "selected")
}

function setElementClass(element: HTMLElement | Element | null| undefined , className: string) {
  while (document.getElementsByClassName(className).length > 0) {
    document.getElementsByClassName(className)[0].classList.remove(className);
  }

  if (element) {
    element.classList.toggle(className, true);
  }
}
</script>

<template>
  <header>

    <div>
      <nav class="navbar">
        <RouterLink to="/dummy" class="prev" @click="elementGotClicked($event)"><div></div></RouterLink>
        <RouterLink to="/" class="selected" @click="elementGotClicked($event)"><div>Home</div></RouterLink>
        <RouterLink to="/Sensor" class="next" @click="elementGotClicked($event)"><div>Sensor</div></RouterLink>
        <RouterLink to="/Webcam" @click="elementGotClicked($event)"><div>Webcam</div></RouterLink>
        <RouterLink to="/Settings" @click="elementGotClicked($event)"><div>Settings</div></RouterLink>
        <RouterLink to="/dummy" @click="elementGotClicked($event)"><div></div></RouterLink>

      </nav>
    </div>
  </header>
<Suspense>
    <RouterView />

  <template #fallback>
    <main>Loading...</main>
  </template>
</Suspense>
</template>
