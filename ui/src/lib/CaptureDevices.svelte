<script lang="ts">
    import { type Device, cap, db } from "../bindings";

    let device: Device | null = $state(null);

    $effect(() => {
        if (
            cap.state.status == null ||
            typeof cap.state.status != "object" ||
            cap.state.status.devices.length == 0
        )
            return;

        if (device == null) {
            device = cap.state.status.devices[0];
        } else {
            device =
                cap.state.status.devices.find((d) => d.name == device?.name) ??
                null;
        }

        if (cap.state.status.capture != null) {
            for (const d of cap.state.status.devices) {
                if (d.name == cap.state.status.capture.name) {
                    device = d;
                    break;
                }
            }
        }
    });
</script>

{#if typeof cap.state.status == "string"}
    <p>Couldn't load <code>libpcap</code>: <code>{cap.state}</code></p>
{:else if cap.state.status != null}
    <p>Loaded <code>{cap.state.status.version}</code></p>

    <div class="join join-horizontal">
        <select
            class="select join-item"
            disabled={cap.state.status.capture != null}
            bind:value={device}
        >
            {#each cap.state.status.devices as device}
                <option value={device} selected>
                    {device.name}
                    {#if device.description}
                        : ({device.description})
                    {/if}
                </option>
            {/each}
        </select>

        {#if cap.state.status.capture == null}
            <button
                class="join-item btn btn-primary"
                onclick={() => cap.startCapture(device)}
                disabled={device == null}>Start Capture</button
            >
        {:else}
            <button class="join-item btn btn-error" onclick={cap.stopCapture}
                >Stop Capture</button
            >
        {/if}
    </div>
{/if}
