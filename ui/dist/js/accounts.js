import {
    addManagedAccount,
    deleteManagedAccount,
    listManagedAccounts,
    switchManagedAccount,
} from './api.js';
import { logError } from './logger.js';
import { showToast } from './ui.js';
import { loadUsageData } from './usage.js';

export function initAccounts() {
    document.getElementById('account-form')?.addEventListener('submit', handleSubmit);
    document.getElementById('refresh-accounts-btn')?.addEventListener('click', loadAccounts);
    loadAccounts();
}

async function handleSubmit(event) {
    event.preventDefault();
    const label = document.getElementById('account-label')?.value.trim() || null;
    const session = document.getElementById('account-session')?.value.trim();

    if (!session) {
        showToast('Session value is required.', 'error');
        return;
    }

    try {
        await addManagedAccount(label, session);
        event.target.reset();
        showToast('Account added locally.', 'success');
        await loadAccounts();
    } catch (error) {
        logError('Failed to add account:', error);
        showToast(`Failed to add account: ${error}`, 'error');
    }
}

async function loadAccounts() {
    const body = document.getElementById('accounts-body');
    if (!body) return;

    body.textContent = '';
    appendEmptyRow(body, 'Loading accounts...');

    try {
        const accounts = await listManagedAccounts();
        renderAccounts(accounts || []);
    } catch (error) {
        logError('Failed to load accounts:', error);
        body.textContent = '';
        appendEmptyRow(body, 'Unable to load local accounts.');
    }
}

function renderAccounts(accounts) {
    const body = document.getElementById('accounts-body');
    if (!body) return;

    body.textContent = '';
    if (!accounts.length) {
        appendEmptyRow(body, 'No saved accounts yet.');
        return;
    }

    accounts.forEach((account) => body.appendChild(createAccountRow(account)));
}

function createAccountRow(account) {
    const row = document.createElement('tr');
    appendCell(row, account.label || `Account ${account.index + 1}`);
    appendCell(row, account.email || account.subject_hint || 'Private account');
    appendCell(row, account.status || 'unknown', `status-${account.status || 'unknown'}`);
    appendCell(row, account.is_current ? 'Current' : 'Saved');

    const actions = document.createElement('td');
    actions.className = 'account-actions';
    actions.appendChild(actionButton('Use', () => switchAccount(account.index), account.is_current));
    actions.appendChild(actionButton('Delete', () => removeAccount(account.index), false, 'danger'));
    row.appendChild(actions);
    return row;
}

async function switchAccount(index) {
    try {
        const message = await switchManagedAccount(index);
        showToast(message, 'success');
        await Promise.all([loadAccounts(), loadUsageData(true)]);
    } catch (error) {
        logError('Failed to switch account:', error);
        showToast(`Failed to switch account: ${error}`, 'error');
    }
}

async function removeAccount(index) {
    try {
        await deleteManagedAccount(index);
        showToast('Account removed locally.', 'success');
        await loadAccounts();
    } catch (error) {
        logError('Failed to remove account:', error);
        showToast(`Failed to remove account: ${error}`, 'error');
    }
}

function actionButton(label, onClick, disabled, variant = '') {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = `btn-text ${variant}`.trim();
    button.textContent = label;
    button.disabled = disabled;
    button.addEventListener('click', onClick);
    return button;
}

function appendCell(row, value, className) {
    const cell = document.createElement('td');
    if (className) cell.className = className;
    cell.textContent = String(value);
    row.appendChild(cell);
}

function appendEmptyRow(body, message) {
    const row = document.createElement('tr');
    const cell = document.createElement('td');
    cell.colSpan = 5;
    cell.className = 'loading-row';
    cell.textContent = message;
    row.appendChild(cell);
    body.appendChild(row);
}
