<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { save, confirm, open } from "@tauri-apps/plugin-dialog";
  import type { MetamathData } from "$lib/sharedState/model.svelte";
  import { inProgressTheoremData } from "$lib/sharedState/metamathData/inProgressTheoremData.svelte";
  import { tabManager } from "$lib/sharedState/tabData.svelte";

  let resetApp = () => {
    inProgressTheoremData.resetTheoremsLocal();
    tabManager.resetTabs();
  };

  let createNewDB = async () => {
    // Allow user to select file location
    const filePath = await save({ filters: [{ name: "Metamath SQLite Database", extensions: ["mm.sqlite", "mm.db"] }] });
    if (filePath) {
      invoke("create_database", { filePath })
        .then(() => {
          resetApp();
          goto("/main");
        })
        .catch(async (error) => {
          if (error == "DatabaseExistsError") {
            let confirmed = await confirm("You are about to override and delete an existing database. Are you sure?", { title: "Warning (mmdbt)", kind: "warning" });
            if (confirmed) {
              invoke("create_or_override_database", { filePath }).then(() => {
                resetApp();
                goto("/main");
              });
            }
          }
        });
    }
  };

  let openDB = async () => {
    const filePath = await open({ multiple: false, directory: false, filters: [{ name: "Metamath SQLite Database", extensions: ["mm.sqlite", "mm.db"] }] });

    if (filePath) {
      invoke("open_database", { filePath }).then((metamathDataUnknown) => {
        let metamathData = metamathDataUnknown as MetamathData;
        resetApp();
        for (let theorem of metamathData.in_progress_theorems) {
          inProgressTheoremData.addTheoremLocal(theorem.name, theorem.text);
        }
        goto("/main");
      });
    }
  };
</script>

<main>
  <div class="m-40 text-center">
    <h1 class="text-4xl">Welcome to mmdbt!</h1>
    <div>
      <button onclick={createNewDB} class="inline-block mt-4">Create new Metamath database</button>
    </div>
    <div>
      <button onclick={openDB} class="inline-block mt-4">Open Metamath database</button>
    </div>
  </div>
</main>
