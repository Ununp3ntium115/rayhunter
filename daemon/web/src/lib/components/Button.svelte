<script lang="ts">
    import type { Snippet } from 'svelte';

    let {
        variant = 'blue',
        disabled = false,
        onclick,
        'aria-label': ariaLabel,
        children,
    }: {
        variant?: 'blue' | 'red' | 'green';
        disabled?: boolean;
        onclick?: () => void;
        'aria-label'?: string;
        children: Snippet;
    } = $props();

    const variantClasses = {
        blue: {
            enabled: 'bg-blue-500 hover:bg-blue-700',
            disabled: 'bg-blue-500 opacity-50 cursor-not-allowed',
        },
        red: {
            enabled: 'bg-red-500 hover:bg-red-700',
            disabled: 'bg-red-500 opacity-50 cursor-not-allowed',
        },
        green: {
            enabled: 'bg-green-500 hover:bg-green-700',
            disabled: 'bg-green-500 opacity-50 cursor-not-allowed',
        },
    };

    let buttonClasses = $derived(
        disabled ? variantClasses[variant].disabled : variantClasses[variant].enabled
    );
</script>

<button
    class="text-white font-bold py-2 px-2 sm:px-4 rounded-md flex flex-row items-center gap-1 {buttonClasses}"
    {disabled}
    {onclick}
    aria-label={ariaLabel}
>
    {@render children()}
</button>
