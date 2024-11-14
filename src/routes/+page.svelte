<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
</script>

<main>
  <div class="m-40 text-center">
    <h1 class="text-4xl">Welcome to mmdbt!</h1>
    <div>
      <a href="/main" class="inline-block mt-4">Create new Metamath database </a>
    </div>
    <div>
      <a href="/main/theorem/sp" class="inline-block mt-4">Example theorem 1</a>
    </div>
    <div>
      <a href="/main/theorem/ax-sep" class="inline-block mt-4">Example theorem 2</a>
    </div>
    <div>
      <a href="/main/theorem/ax-rep" class="inline-block mt-4">Example theorem 3</a>
    </div>
    <div>
      <a href="/main/editor/ax-rep" class="inline-block mt-4">Editor</a>
    </div>
  </div>
</main>
