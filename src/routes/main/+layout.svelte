<script>
	let { children } = $props();
</script>

<div class="h-screen w-screen">
  <div class="h-full w-56 float-left border-8 border-red-600">
    <p>This is the side bar</p>
  </div>
  <div class="h-full ml-56">
    {@render children()}
  </div>
</div>