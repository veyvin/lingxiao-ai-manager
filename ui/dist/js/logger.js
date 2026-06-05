const MAX_LOG_ENTRIES = 300;
let logEntries = [];

function getLogsContainer() {
    return document.getElementById('logs-container');
}

function getTauriInvoke() {
    if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke.bind(window.__TAURI__.core);
    if (window.__TAURI__?.invoke) return window.__TAURI__.invoke.bind(window.__TAURI__);
    if (window.__TAURI_INTERNALS__?.invoke) return window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__);
    if (typeof window.invoke === 'function') return window.invoke;
    return null;
}

function formatTimestamp() {
    const now = new Date();
    return `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}.${String(now.getMilliseconds()).padStart(3, '0')}`;
}

function stringify(value) {
    if (typeof value !== 'object' || value === null) return String(value);
    try {
        return JSON.stringify(value);
    } catch {
        return String(value);
    }
}

function redact(message) {
    return String(message)
        .replace(/(WorkosCursorSessionToken=)[^;\s]+/gi, '$1[redacted]')
        .replace(/(access[_-]?token["']?\s*[:=]\s*["']?)[^"',\s]+/gi, '$1[redacted]')
        .replace(/(refresh[_-]?token["']?\s*[:=]\s*["']?)[^"',\s]+/gi, '$1[redacted]')
        .replace(/(password["']?\s*[:=]\s*["']?)[^"',\s]+/gi, '$1[redacted]');
}

function trimEntries() {
    if (logEntries.length > MAX_LOG_ENTRIES) {
        logEntries = logEntries.slice(-MAX_LOG_ENTRIES);
    }
}

function renderLog(entry) {
    const container = getLogsContainer();
    if (!container) return;

    container.querySelector('.logs-empty')?.remove();

    const row = document.createElement('div');
    row.className = `log-entry log-${entry.level}`;

    const time = document.createElement('span');
    time.className = 'log-time';
    time.textContent = entry.timestamp;

    const level = document.createElement('span');
    level.className = `log-level log-level-${entry.level}`;
    level.textContent = entry.level.toUpperCase();

    const text = document.createElement('span');
    text.className = 'log-message';
    text.textContent = entry.message;

    row.append(time, level, text);
    container.appendChild(row);

    while (container.querySelectorAll('.log-entry').length > MAX_LOG_ENTRIES) {
        container.querySelector('.log-entry')?.remove();
    }
    container.scrollTop = container.scrollHeight;
}

function addLog(level, ...args) {
    const entry = {
        level,
        message: redact(args.map(stringify).join(' ')),
        timestamp: formatTimestamp(),
    };
    logEntries.push(entry);
    trimEntries();
    renderLog(entry);
}

function copyAllLogs() {
    const text = logEntries.map((entry) => `[${entry.timestamp}] ${entry.level.toUpperCase()} ${entry.message}`).join('\n');
    if (!text) {
        addLog('warn', 'No logs to copy.');
        return;
    }
    navigator.clipboard?.writeText(text)
        .then(() => addLog('info', 'Logs copied.'))
        .catch(() => addLog('error', 'Failed to copy logs.'));
}

function clearLogs() {
    logEntries = [];
    const container = getLogsContainer();
    if (container) container.innerHTML = '<div class="logs-empty">No logs yet</div>';
}

function startPolling() {
    if (window._logPollingInterval) return;
    const invoke = getTauriInvoke();
    if (!invoke) return;

    window._logPollingInterval = setInterval(async () => {
        try {
            const logs = await invoke('get_log_events');
            if (!Array.isArray(logs)) return;
            logs.forEach((item) => {
                const entry = {
                    level: item.level || 'info',
                    message: redact(item.message || ''),
                    timestamp: item.timestamp || formatTimestamp(),
                };
                logEntries.push(entry);
                trimEntries();
                renderLog(entry);
            });
        } catch {
            clearInterval(window._logPollingInterval);
            window._logPollingInterval = null;
        }
    }, 500);
}

function setupLogsPage() {
    document.getElementById('copy-logs-btn')?.addEventListener('click', copyAllLogs);
    document.getElementById('clear-logs-btn')?.addEventListener('click', clearLogs);
    logEntries.forEach(renderLog);
}

export function initLogger() {
    if (window._loggerInitialized) return;
    window._loggerInitialized = true;
    setupLogsPage();
    setTimeout(startPolling, 1000);
}

export function logInfo(...args) { addLog('info', ...args); }
export function logError(...args) { addLog('error', ...args); }
export function logWarn(...args) { addLog('warn', ...args); }
export function logDebug(...args) { addLog('debug', ...args); }
