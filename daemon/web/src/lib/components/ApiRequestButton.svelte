<script lang="ts">
    import { user_action_req } from '$lib/utils.svelte';
    import Spinner from './Spinner.svelte';
    import Button from './Button.svelte';

    let {
        url,
        method = 'POST',
        label,
        loadingLabel,
        disabled = false,
        variant = 'blue',
        icon,
        onclick,
        ariaLabel,
        errorMessage,
        jsonBody,
    }: {
        url: string;
        method?: string;
        label: string;
        loadingLabel?: string;
        disabled?: boolean;
        variant?: 'blue' | 'red' | 'green';
        icon?: any; // Svelte snippet
        onclick?: () => void | Promise<void>;
        ariaLabel?: string;
        errorMessage?: string;
        jsonBody?: unknown;
    } = $props();

    let is_requesting = $state(false);
    let is_disabled = $derived(disabled || is_requesting);

    async function handle_click() {
        if (is_disabled) return;

        is_requesting = true;
        try {
            await user_action_req(
                method,
                url,
                errorMessage ? errorMessage : 'Error performing action',
                jsonBody
            );
            if (onclick) {
                await onclick();
            }
        } catch (err) {
            console.error(`Failed to ${method} ${url}:`, err);
            alert(`Request failed. Please try again.`);
        } finally {
            is_requesting = false;
        }
    }
</script>

<Button {variant} disabled={is_disabled} onclick={handle_click} aria-label={ariaLabel || label}>
    <!-- eslint-disable-next-line svelte/no-useless-children-snippet -->
    {#snippet children()}
        <span>{is_requesting && loadingLabel ? loadingLabel : label}</span>
        {#if is_requesting}
            <Spinner class="w-4 h-4 text-white" />
        {:else if icon}
            {@render icon()}
        {/if}
    {/snippet}
</Button>
