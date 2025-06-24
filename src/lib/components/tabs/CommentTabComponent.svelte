<script lang="ts" module>
  import type { Comment, CommentPageData, HeaderPath } from "$lib/sharedState/model.svelte";
  import CommentTabComponent from "$lib/components/tabs/CommentTabComponent.svelte";

  export class CommentTab extends Tab {
    component = CommentTabComponent;

    #headerPath: HeaderPath; // = $state({ path: [] });
    #commentNum: number;
    #comment: Comment = $state({ text: "" });

    constructor(headerPath: HeaderPath, commentNum: number) {
      super();
      this.#headerPath = headerPath;
      this.#commentNum = commentNum;
    }

    async loadData(): Promise<void> {
      this.#comment = (await invoke("get_comment_local", { headerPath: this.#headerPath, commentNum: this.#commentNum })) as Comment;
    }

    unloadData(): void {
      this.#comment = { text: "" };
    }

    name(): string {
      return "Comment " + util.headerPathToStringRep(this.#headerPath) + "#" + (this.#commentNum + 1);
    }

    sameTab(tab: Tab): boolean {
      return tab instanceof CommentTab && tab.headerPath.path == this.#headerPath.path && tab.commentNum == this.#commentNum;
    }

    get headerPath() {
      return this.#headerPath;
    }
    get commentNum() {
      return this.#commentNum;
    }
    get comment() {
      return this.#comment;
    }
  }
</script>

<script lang="ts">
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { util } from "$lib/sharedState/util.svelte";
  import CommentPage from "../pages/CommentPage.svelte";

  let { tab }: { tab: Tab } = $props();

  let commentTab: CommentTab = $derived.by(() => {
    if (tab instanceof CommentTab) {
      return tab;
    }
    throw Error("Wrong Tab Type!");
  });

  let pageData: CommentPageData = $derived({
    comment: commentTab.comment,
    commentPath: util.headerPathToStringRep(commentTab.headerPath) + "#" + (commentTab.commentNum + 1),
    discriminator: "CommentPageData",
  });
</script>

<CommentPage {pageData}></CommentPage>
