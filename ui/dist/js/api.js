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

function ensureTauriLoaded() {
    const invoke = getTauriInvoke();
    if (!invoke) {
        throw new Error('Tauri runtime is not available. Run this app through Tauri.');
    }
    return invoke;
}

let localAccountsCache = null;
let localAccountsCacheTime = 0;
let localAccountsPromise = null;
const LOCAL_ACCOUNTS_CACHE_TTL = 2000;

export async function getLocalAccounts(forceRefresh = false) {
    const now = Date.now();
    const cacheFresh = now - localAccountsCacheTime < LOCAL_ACCOUNTS_CACHE_TTL;

    if (!forceRefresh && localAccountsCache && cacheFresh) {
        return localAccountsCache;
    }
    if (localAccountsPromise) {
        return await localAccountsPromise;
    }

    const invoke = ensureTauriLoaded();
    localAccountsPromise = invoke('get_local_accounts')
        .then((result) => {
            localAccountsCache = result;
            localAccountsCacheTime = Date.now();
            localAccountsPromise = null;
            return result;
        })
        .catch((error) => {
            localAccountsPromise = null;
            throw error;
        });

    return await localAccountsPromise;
}

export function clearLocalAccountsCache() {
    localAccountsCache = null;
    localAccountsCacheTime = 0;
    localAccountsPromise = null;
}

export async function getUsageEvents() {
    return await ensureTauriLoaded()('get_usage_events');
}

export async function listManagedAccounts() {
    return await ensureTauriLoaded()('list_managed_accounts');
}

export async function addManagedAccount(label, session) {
    return await ensureTauriLoaded()('add_managed_account', {
        request: { label, session },
    });
}

export async function deleteManagedAccount(index) {
    return await ensureTauriLoaded()('delete_managed_account', { index });
}

export async function switchManagedAccount(index) {
    const result = await ensureTauriLoaded()('switch_managed_account', { index });
    clearLocalAccountsCache();
    return result;
}
