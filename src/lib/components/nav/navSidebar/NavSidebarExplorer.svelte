<script>
  import { goto } from "$app/navigation";
</script>

<div>
  <p>This is the Explorer Component</p>

  <div>
    <button
      class="border border-black ml-2 mt-2 rounded p-1"
      onclick={() => {
        goto("/");
      }}>Back to main menu (temp)</button
    >
  </div>
</div>
