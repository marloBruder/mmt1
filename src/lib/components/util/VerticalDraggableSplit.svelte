<script lang="ts">
  import { createInstanceId } from "$lib/sharedState/idGenerator.svelte";
  import { onDestroy, onMount, type Snippet } from "svelte";

  let { first, second, startPercent = 0.2, onDrag = () => {} }: { first: Snippet; second: Snippet; startPercent?: number; onDrag?: () => void } = $props();

  let position = $state(320);
  let containerId = "container-id-" + createInstanceId();
  let firstId = "first-id-" + createInstanceId();
  let secondId = "second-id-" + createInstanceId();
  let buttonContainerId = "button-container-id-" + createInstanceId();

  let moving = $state(false);
  let hovering = $state(false);

  let handleMouseMove = (e: MouseEvent) => {
    let containerElement = document.getElementById(containerId)!;

    let newPosition = e.clientX - containerElement.offsetLeft;
    position = Math.max(200, Math.min(containerElement.clientWidth - 200, newPosition));

    onDrag();
  };

  let handleWindowResize = () => {
    let containerElement = document.getElementById(containerId)!;
    let firstElement = document.getElementById(firstId)!;
    let secondElement = document.getElementById(secondId)!;
    let buttonContainerElement = document.getElementById(buttonContainerId)!;

    position = Math.max(200, Math.min(containerElement.clientWidth - 200, position));

    firstElement.style.width = position + "px";
    secondElement.style.width = Math.max(containerElement.clientWidth - position, 0) + "px";

    buttonContainerElement.style.left = position - 2 + containerElement.offsetLeft + "px";

    buttonContainerElement.style.height = containerElement.clientHeight + "px";
  };

  let onmousedown = () => {
    moving = true;

    document.addEventListener("mousemove", handleMouseMove);
  };

  let onmouseenter = () => {
    hovering = true;
  };

  let onmouseleave = () => {
    hovering = false;
  };

  onMount(() => {
    document.addEventListener("mouseup", () => {
      moving = false;
      document.removeEventListener("mousemove", handleMouseMove);
    });

    window.addEventListener("resize", handleWindowResize);

    let containerElement = document.getElementById(containerId)!;
    let firstElement = document.getElementById(firstId)!;
    let secondElement = document.getElementById(secondId)!;
    let buttonContainerElement = document.getElementById(buttonContainerId)!;

    position = containerElement.clientWidth * startPercent;

    firstElement.style.width = position + "px";
    secondElement.style.width = Math.max(containerElement.clientWidth - position, 0) + "px";

    buttonContainerElement.style.left = position - 2 + containerElement.offsetLeft + "px";

    buttonContainerElement.style.top = containerElement.offsetTop + "px";
    buttonContainerElement.style.height = containerElement.clientHeight + "px";
  });

  onDestroy(() => {
    window.removeEventListener("resize", handleWindowResize);
  });

  $effect(() => {
    let containerElement = document.getElementById(containerId)!;
    let firstElement = document.getElementById(firstId)!;
    let secondElement = document.getElementById(secondId)!;
    let buttonContainerElement = document.getElementById(buttonContainerId)!;

    firstElement.style.width = position + "px";
    secondElement.style.width = Math.max(containerElement.clientWidth - position, 0) + "px";

    buttonContainerElement.style.left = position - 2 + containerElement.offsetLeft + "px";
  });
</script>

<div id={containerId} class="h-full w-full flex flex-row">
  <div id={firstId} class="">
    {@render first()}
  </div>
  <div id={secondId} class="overflow-hidden">
    {@render second()}
  </div>
</div>

<div id={buttonContainerId} class={"fixed " + (moving || hovering ? " bg-opacity-50 " : " bg-opacity-0 ") + " bg-slate-600 w-1 "}>
  <button class={"w-full h-full " + (moving ? "cursor-grabbing" : "cursor-grab")} {onmousedown} {onmouseenter} {onmouseleave} aria-label="dragButton"></button>
</div>
