<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { save, confirm, open } from "@tauri-apps/plugin-dialog";
  import type { MetamathData } from "$lib/sharedState/model.svelte";
  import editorTabs from "$lib/sharedState/mainData.svelte";

  let createNewDB = async () => {
    // Allow user to select file location
    const filePath = await save({ filters: [{ name: "Metamath SQLite Database", extensions: ["mm.sqlite"] }] });
    if (filePath) {
      invoke("create_database", { filePath })
        .then(() => {
          editorTabs.clearTabs();
          goto("/main");
        })
        .catch(async (error) => {
          if (error == "DatabaseExistsError") {
            let confirmed = await confirm("You are about to override and delete an existing database. Are you sure?", { title: "Warning (mmdbt)", kind: "warning" });
            if (confirmed) {
              invoke("create_or_override_database", { filePath }).then(() => {
                editorTabs.clearTabs();
                goto("/main");
              });
            }
          }
        });
    }
  };

  let openDB = async () => {
    const filePath = await open({ multiple: false, directory: false });

    if (filePath) {
      invoke("open_database", { filePath }).then((metamathDataUnknown) => {
        let metamathData = metamathDataUnknown as MetamathData;
        editorTabs.clearTabs();
        for (let theorem of metamathData.in_progress_theorems) {
          editorTabs.addTab(theorem.name, theorem.text);
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
      <a href="/main/editor/0" class="inline-block mt-4">Editor</a>
    </div>
  </div>
</main>
