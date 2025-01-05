<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { save, confirm, open } from "@tauri-apps/plugin-dialog";
  import { tabManager } from "$lib/sharedState/tabData.svelte";
  import { nameListData } from "$lib/sharedState/nameListData.svelte";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";

  let resetApp = () => {
    nameListData.resetLists();
    explorerData.resetExplorer();
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
      invoke("open_database", { filePath }).then(async (metamathDataUnknown) => {
        resetApp();
        nameListData
          .load()
          .then(() => {
            goto("/main");
          })
          .catch(() => {
            console.log("Error while loading data");
          });
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
