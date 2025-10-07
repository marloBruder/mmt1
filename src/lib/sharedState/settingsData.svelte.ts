import { load, Store } from "@tauri-apps/plugin-store";
import { util } from "./util.svelte";
import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  definitionsStartWith: string;
  colorUnicodePreview: boolean;
  showUnifyResultInUnicodePreview: boolean;
  defaultShowAll: boolean;
}

let defaultSettings: Settings = {
  definitionsStartWith: "df-",
  colorUnicodePreview: true,
  showUnifyResultInUnicodePreview: true,
  defaultShowAll: false,
};

class SettingsData {
  settings: Settings = $state(util.clone(defaultSettings) as Settings);
  settingsStore: Store | null = null;

  async setupSettings() {
    this.settingsStore = await load("settings.json");
    let settingsIfExists = (await this.settingsStore.get("settings")) as Settings | undefined;
    if (settingsIfExists !== undefined) {
      this.settings = { ...util.clone(defaultSettings), ...settingsIfExists };
    } else {
      this.settingsStore.set("settings", defaultSettings);
    }

    await invoke("set_settings", { settings: this.settings });
  }

  cloneSettings(): Settings {
    return util.clone(this.settings) as Settings;
  }
}

let settingsData = new SettingsData();

export { defaultSettings, settingsData };
