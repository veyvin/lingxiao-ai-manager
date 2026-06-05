import { initUsage } from './usage.js';
import { initAccounts } from './accounts.js';
import { initTabs } from './tabs.js';
import { showToast } from './ui.js';
import { initLogger, logError } from './logger.js';

function getTauriInvoke() {
    if (window.__TAURI__?.core?.invoke) {
        return window.__TAURI__.core.invoke.bind(window.__TAURI__.core);
    }
    if (window.__TAURI__?.invoke) {
        return window.__TAURI__.invoke.bind(window.__TAURI__);
    }
    if (window.__TAURI_INTERNALS__?.invoke) {
        return window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__);
    }
    if (typeof window.invoke === 'function') {
        return window.invoke;
    }
    return null;
}

async function waitForTauri(maxWaitMs = 10000) {
    const startTime = Date.now();

    while (Date.now() - startTime <= maxWaitMs) {
        const invoke = getTauriInvoke();
        if (invoke) {
            try {
                await invoke('test_logging');
                return;
            } catch {
                // The runtime may be visible before custom commands are ready.
            }
        }
        await new Promise((resolve) => setTimeout(resolve, 50));
    }

    throw new Error(`Tauri runtime did not become ready within ${maxWaitMs}ms`);
}

document.addEventListener('DOMContentLoaded', async () => {
    try {
        initLogger();
        await waitForTauri();
        initTabs();
        initUsage();
        initAccounts();
    } catch (error) {
        const message = error?.message || String(error);
        logError('Initialization failed:', message);
        showToast(`Initialization failed: ${message}`, 'error');
    }
});
