<script lang="ts">
    import StylizedButton from '$lib/StylizedButton.svelte';
    import type { InstallerSubcommand } from '$lib/types.svelte';

    let {
        subcommand,
        go_back,
        continue_to_args,
    }: {
        subcommand: InstallerSubcommand;
        go_back: () => void;
        continue_to_args: () => void;
    } = $props();
</script>

<div class="flex flex-col gap-4 items-center max-w-lg mx-auto px-4">
    <h1 class="font-bold text-2xl text-center">Connect Your Device</h1>
    <p class="text-gray-600 text-center">
        Follow these steps to connect <strong>{subcommand.label}</strong> to your computer before
        installing Rayhunter.
    </p>
    <ol class="space-y-3 text-base w-full list-decimal list-inside">
        <li>Connect the hotspot to your computer using a USB cable.</li>
        <li>
            Enable USB tethering on the device's screen or settings menu (sometimes called
            "Internet sharing via USB").
            <ul class="list-disc list-inside ml-4 mt-1 text-gray-700 space-y-1">
                <li>
                    Alternatively, connect your computer's WiFi to the device's hotspot network.
                </li>
            </ul>
        </li>
        <li>
            Verify you can reach the device's admin page in a browser:
            <ul class="list-disc list-inside ml-4 mt-1 text-gray-700 space-y-1">
                <li>Orbic: <code class="font-mono text-sm">http://192.168.1.1</code></li>
                <li>TP-Link: <code class="font-mono text-sm">http://192.168.0.1</code></li>
                <li>
                    Other devices: try <code class="font-mono text-sm">http://192.168.1.1</code> or
                    check your device's documentation.
                </li>
            </ul>
        </li>
    </ol>
    <div class="flex justify-evenly w-full mt-2">
        <StylizedButton label="Back" onclick={go_back} />
        <StylizedButton color="blue" label="Continue" onclick={continue_to_args} />
    </div>
</div>
