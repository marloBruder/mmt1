<script lang="ts">
  import { onMount } from "svelte";

  let { title, buttons }: { title: string; buttons: string[] } = $props();

  let buttonID = $derived("titleBarDropdownButton-" + title);
  let dropdownID = $derived("titleBarDropdown-" + title);

  let open: boolean = $state(false);

  let onclick = () => {
    open = !open;
  };

  // $effect(() => {
  //   if (open) {
  //     let button = document.getElementById(buttonID);
  //     let dropdown = document.getElementById(dropdownID);

  //     dropdown.style.x = button?.style.x;
  //     console.log("Was here");
  //     console.log(button?.);
  //     dropdown.style.y = button?.style.y + button?.style.height;
  //   }
  // });

  let onfocusout = () => {
    setTimeout(() => {
      open = false;
    }, 100);
  };
</script>

<div>
  <button id={buttonID} {onclick} {onfocusout} class={"px-1 rounded " + (open ? "bg-gray-200 " : "")}>
    {title}
  </button>
  {#if open}
    <div id={dropdownID} class="fixed bg-white border border-black p-2">
      {#each buttons as button}
        <div>
          <button>{button}</button>
        </div>
      {/each}
    </div>
  {/if}
</div>
