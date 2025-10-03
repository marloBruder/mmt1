import { load, Store } from "@tauri-apps/plugin-store";
import { util } from "./util.svelte";

export interface Settings {
  definitionsStartWith: string;
  colorUnicodePreview: boolean;
}

let defaultSettings: Settings = {
  definitionsStartWith: "df-",
  colorUnicodePreview: true,
};

class SettingsData {
  settings: Settings = $state(util.clone(defaultSettings) as Settings);
  settingsStore: Store | null = null;

  async setupSettings() {
    this.settingsStore = await load("settings.json");
    let settingsIfExists = (await this.settingsStore.get("settings")) as Settings | undefined;
    if (settingsIfExists !== undefined) {
      this.settings = settingsIfExists;
    } else {
      this.settingsStore.set("settings", defaultSettings);
    }
  }

  cloneSettings(): Settings {
    return util.clone(this.settings) as Settings;
  }
}

let settingsData = new SettingsData();

export { defaultSettings, settingsData };
