export function showToast(message, type = 'info', duration = 3000) {
    const container = document.getElementById('toast-container');
    if (!container) {
        alert(message);
        return;
    }

    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;

    const icon = document.createElement('span');
    icon.className = 'toast-icon';
    icon.textContent = type === 'success' ? 'OK' : type === 'error' ? '!' : 'i';

    const content = document.createElement('span');
    content.className = 'toast-content';
    content.textContent = String(message);

    const close = document.createElement('button');
    close.className = 'toast-close';
    close.type = 'button';
    close.textContent = 'x';
    close.addEventListener('click', () => toast.remove());

    toast.append(icon, content, close);
    container.appendChild(toast);

    setTimeout(() => {
        toast.classList.add('hide');
        toast.addEventListener('transitionend', () => toast.remove(), { once: true });
    }, duration);
}
