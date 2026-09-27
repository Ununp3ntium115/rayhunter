<script lang="ts">
    import { user_action_req } from '$lib/utils.svelte';
    import TrashIcon from './TrashIcon.svelte';
    import Button from './Button.svelte';

    let {
        text,
        url,
        prompt,
        name,
    }: {
        text?: string;
        url: string;
        prompt: string;
        name: string;
    } = $props();

    function confirm_delete() {
        if (window.confirm(prompt)) {
            user_action_req('POST', url, 'Unable to delete recording ' + name);
        }
    }
</script>

<Button variant="red" onclick={confirm_delete} aria-label="delete">
    <!-- eslint-disable-next-line svelte/no-useless-children-snippet -->
    {#snippet children()}
        <p>{text}</p>
        <TrashIcon class="w-6 h-6 text-white" />
    {/snippet}
</Button>
