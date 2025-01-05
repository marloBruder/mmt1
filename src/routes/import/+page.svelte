<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { save, confirm, open } from "@tauri-apps/plugin-dialog";
  import { resetApp } from "../+page.svelte";

  let mmFilePath = $state("");
  let dbFilePath = $state("");

  let selectMetamathFilePath = async () => {
    const filePath = await open({ multiple: false, directory: false, filters: [{ name: "Metamath File", extensions: ["mm"] }] });
    if (filePath) {
      mmFilePath = filePath;
    }
  };

  let selectDatabaseFilePath = async () => {
    const filePath = await save({ filters: [{ name: "Metamath SQLite Database", extensions: ["mm.sqlite", "mm.db"] }] });
    if (filePath) {
      dbFilePath = filePath;
    }
  };

  let createDatabase = async () => {
    invoke("import_database", { mmFilePath, dbFilePath })
      .then(() => {
        resetApp();
        goto("/main");
      })
      .catch(async (error) => {
        if (error == "DatabaseExistsError") {
          let confirmed = await confirm("You are about to override and delete an existing database. Are you sure?", { title: "Warning (mmdbt)", kind: "warning" });
          if (confirmed) {
            invoke("import_and_override_database", { mmFilePath, dbFilePath }).then(() => {
              resetApp();
              goto("/main");
            });
          }
        }
      });
  };
</script>

<div class="mt-40 text-center">
  <h1 class="text-2xl">Import Metamath (.mm) file</h1>
  <div class="pt-4">
    <label for="mm_path">Metamath file path</label>
    <br />
    <input id="mm_path" bind:value={mmFilePath} class="border border-black rounded" size="50" />
    <br />
    <button onclick={selectMetamathFilePath}>Select</button>
  </div>
  <div class="pt-4">
    <label for="mm_path">New database file path</label>
    <br />
    <input id="mm_path" bind:value={dbFilePath} class="border border-black rounded" size="50" />
    <br />
    <button onclick={selectDatabaseFilePath}>Select</button>
  </div>
  <div class="pt-8">
    <button onclick={createDatabase} class="text-xl">Create</button>
  </div>
  <div class="pt-4">
    <a href="/">Back</a>
  </div>
</div>
