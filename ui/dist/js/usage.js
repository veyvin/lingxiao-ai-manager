import { getLocalAccounts, getUsageEvents } from './api.js';
import { showToast } from './ui.js';
import { logError } from './logger.js';

let currentAccount = null;
let allEvents = [];

function restoreButton(btnId) {
    const button = document.getElementById(btnId);
    if (button) {
        button.classList.remove('loading');
        button.disabled = false;
    }
}

export function initUsage() {
    document.getElementById('refresh-usage-btn')?.addEventListener('click', () => {
        loadUsageData(true);
    });

    if (!document.getElementById('model-detail-overlay')) {
        document.body.appendChild(createModelDetailDialog());
    }

    loadUsageData(false);
}

async function loadUsageData(forceRefresh = false) {
    const button = document.getElementById('refresh-usage-btn');
    if (!button || button.disabled) return;

    button.disabled = true;
    button.classList.add('loading');

    try {
        const accounts = await getLocalAccounts(forceRefresh);
        if (!accounts || accounts.length === 0) {
            showNoAccount();
            return;
        }

        currentAccount = accounts[0];
        displayAccountInfo(currentAccount);

        try {
            const usage = await getUsageEvents();
            allEvents = usage.events || [];
            displayUsageStats(usage);
        } catch (error) {
            logError('Failed to fetch usage events:', error);
            allEvents = [];
            showNoUsageData('No usage data is available.');
        }
    } catch (error) {
        logError('Failed to load account data:', error);
        showToast(`Failed to load data: ${error}`, 'error');
        showNoUsageData('Unable to load usage data.');
    } finally {
        restoreButton('refresh-usage-btn');
    }
}

function displayAccountInfo(account) {
    const emailEl = document.getElementById('current-account-email');
    const badgeEl = document.getElementById('account-type-badge');
    const daysBadgeEl = document.getElementById('days-remaining-badge');

    if (emailEl) emailEl.textContent = account.email || 'Signed-in account';
    if (!badgeEl) return;

    const displayType = account.account_type || '--';
    badgeEl.textContent = displayType;
    badgeEl.className = 'account-type-badge';
    badgeEl.removeAttribute('style');

    if (daysBadgeEl) {
        const days = Number(account.days_remaining);
        if (Number.isFinite(days) && days > 0) {
            daysBadgeEl.textContent = `${days} days left`;
            daysBadgeEl.style.display = '';
            daysBadgeEl.className = 'days-remaining-badge';
        } else {
            daysBadgeEl.style.display = 'none';
        }
    }
}

function formatTokens(value) {
    const n = Number(value) || 0;
    if (Math.abs(n) >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
    if (Math.abs(n) >= 1_000) return `${(n / 1_000).toFixed(2)}K`;
    return String(n);
}

function displayUsageStats(usage) {
    setText('total-cost', `$${Number(usage.total_cost || 0).toFixed(2)}`);
    setText('total-tokens', formatTokens(usage.total_tokens));
    setText('input-tokens', formatTokens(usage.input_tokens));
    setText('output-tokens', formatTokens(usage.output_tokens));
    setText('cache-write', formatTokens(usage.cache_write_tokens));
    setText('cache-read', formatTokens(usage.cache_read_tokens));
    displayModelDetails(usage.models || [], usage);
}

function getModelName(name) {
    if (!name || !name.trim()) return 'unknown';
    const lower = name.toLowerCase();
    return lower === 'default' || lower === 'auto' ? 'auto' : name;
}

function displayModelDetails(models, usage) {
    const tbody = document.getElementById('model-details-body');
    const tfoot = document.getElementById('model-details-footer');
    if (!tbody) return;

    tbody.textContent = '';
    if (!models.length) {
        appendEmptyRow(tbody, 5, 'No model usage data.');
        return;
    }

    let totalRequests = 0;
    models.forEach((model) => {
        totalRequests += model.request_count;
        tbody.appendChild(createModelRow(model));
    });

    if (tfoot) {
        tfoot.style.display = '';
        setText('total-requests', totalRequests);
        setText('total-tokens-footer', formatTokens(usage.total_tokens));
        setText('total-cost-footer', `$${Number(usage.total_cost || 0).toFixed(2)}`);
    }
}

function createModelRow(model) {
    const name = getModelName(model.name);
    const row = document.createElement('tr');
    appendCell(row, name, 'model-name');
    appendCell(row, model.request_count);
    appendCell(row, formatTokens(model.total_tokens), 'token-value');
    appendCell(row, `$${Number(model.cost || 0).toFixed(2)}`, 'cost-value');

    const action = document.createElement('td');
    const link = document.createElement('a');
    link.href = '#';
    link.className = 'link-btn';
    link.textContent = 'View details';
    link.addEventListener('click', (event) => {
        event.preventDefault();
        showModelDetailDialog(name);
    });
    action.appendChild(link);
    row.appendChild(action);
    return row;
}

function showNoAccount() {
    setText('current-account-email', 'No signed-in Cursor account found');
    setText('account-type-badge', '--');
    const daysBadgeEl = document.getElementById('days-remaining-badge');
    if (daysBadgeEl) daysBadgeEl.style.display = 'none';
    showNoUsageData('Sign in to Cursor locally, then refresh this view.');
}

function showNoUsageData(message) {
    const values = {
        'total-cost': '$0.00',
        'total-tokens': '0',
        'input-tokens': '0',
        'output-tokens': '0',
        'cache-write': '0',
        'cache-read': '0',
    };
    Object.entries(values).forEach(([id, value]) => setText(id, value));

    const tbody = document.getElementById('model-details-body');
    if (tbody) {
        tbody.textContent = '';
        appendEmptyRow(tbody, 5, message);
    }
}

function showModelDetailDialog(modelName) {
    const target = modelName.toLowerCase();
    const modelEvents = allEvents.filter((event) => {
        const model = getModelName(event.model || '').toLowerCase();
        return model === target;
    });

    if (!modelEvents.length) {
        showToast('No usage records for this model.', 'info');
        return;
    }

    const overlay = document.getElementById('model-detail-overlay');
    const nameEl = document.getElementById('model-detail-name');
    if (!overlay || !nameEl) return;

    overlay.style.display = 'flex';
    nameEl.textContent = modelName;
    attachDialogHandlers(overlay, modelEvents);
}

function attachDialogHandlers(overlay, modelEvents) {
    let page = 1;
    const size = 10;
    const totalPages = Math.max(1, Math.ceil(modelEvents.length / size));
    const prev = resetButton('model-detail-prev');
    const next = resetButton('model-detail-next');
    const close = document.getElementById('model-detail-close');

    const render = () => renderDetailPage(modelEvents, page, size, totalPages);
    prev?.addEventListener('click', () => { if (page > 1) { page--; render(); } });
    next?.addEventListener('click', () => { if (page < totalPages) { page++; render(); } });
    if (close) close.onclick = () => { overlay.style.display = 'none'; };
    overlay.onclick = (event) => { if (event.target.id === 'model-detail-overlay') overlay.style.display = 'none'; };

    render();
}

function renderDetailPage(modelEvents, page, size, totalPages) {
    const tbody = document.getElementById('model-detail-body');
    if (!tbody) return;
    tbody.textContent = '';

    modelEvents.slice((page - 1) * size, page * size).forEach((event) => {
        const row = document.createElement('tr');
        const totalTokens = event.input_tokens + event.output_tokens + event.cache_write_tokens + event.cache_read_tokens;
        appendCell(row, new Date(event.timestamp).toLocaleString());
        appendCell(row, formatTokens(totalTokens), 'token-value');
        appendCell(row, formatTokens(event.input_tokens), 'token-value');
        appendCell(row, formatTokens(event.output_tokens), 'token-value');
        appendCell(row, formatTokens(event.cache_write_tokens + event.cache_read_tokens), 'token-value');
        appendCell(row, `$${(event.total_cents / 100).toFixed(4)}`, 'cost-value');
        tbody.appendChild(row);
    });

    setText('model-detail-page-info', `${page} / ${totalPages}`);
    setText('model-detail-total', `${modelEvents.length} records`);
    document.getElementById('model-detail-prev').disabled = page <= 1;
    document.getElementById('model-detail-next').disabled = page >= totalPages;
}

function createModelDetailDialog() {
    const overlay = document.createElement('div');
    overlay.className = 'dialog-overlay';
    overlay.id = 'model-detail-overlay';
    overlay.style.display = 'none';
    overlay.innerHTML = `<div class="dialog-container" style="max-width:900px;max-height:80vh">
        <div class="dialog-header"><h3>Model Details - <span id="model-detail-name"></span></h3>
        <button class="dialog-close" id="model-detail-close">&times;</button></div>
        <div class="dialog-content" style="overflow-y:auto;max-height:60vh">
        <div style="margin-bottom:12px;color:#666;font-size:14px"><span id="model-detail-total">0 records</span></div>
        <table class="model-table" style="width:100%"><thead><tr><th>Time</th><th>Total Tokens</th><th>Input</th>
        <th>Output</th><th>Cache</th><th>Cost</th></tr></thead>
        <tbody id="model-detail-body"><tr><td colspan="6" class="loading-row">Loading...</td></tr></tbody></table>
        <div class="pagination" style="margin-top:16px;justify-content:center">
        <button class="page-btn" id="model-detail-prev">&lt;</button>
        <span class="page-info" id="model-detail-page-info">1 / 1</span>
        <button class="page-btn" id="model-detail-next">&gt;</button></div></div></div>`;
    return overlay;
}

function setText(id, value) {
    const element = document.getElementById(id);
    if (element) element.textContent = String(value);
}

function appendCell(row, value, className) {
    const cell = document.createElement('td');
    if (className) cell.className = className;
    cell.textContent = String(value);
    row.appendChild(cell);
}

function appendEmptyRow(tbody, colspan, message) {
    const row = document.createElement('tr');
    const cell = document.createElement('td');
    cell.colSpan = colspan;
    cell.className = 'loading-row';
    cell.textContent = message;
    row.appendChild(cell);
    tbody.appendChild(row);
}

function resetButton(id) {
    const button = document.getElementById(id);
    if (!button) return null;
    const clone = button.cloneNode(true);
    button.parentNode.replaceChild(clone, button);
    return clone;
}

export function getCurrentAccount() {
    return currentAccount;
}

export { loadUsageData };
