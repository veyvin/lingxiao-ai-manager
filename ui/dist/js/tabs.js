export function initTabs() {
    const tabButtons = document.querySelectorAll('.tab-btn');
    const tabContents = document.querySelectorAll('.tab-content');

    tabButtons.forEach((button) => {
        button.addEventListener('click', () => {
            const tabId = button.dataset.tab;

            tabButtons.forEach((item) => {
                item.classList.remove('active');
                item.setAttribute('aria-selected', 'false');
            });
            button.classList.add('active');
            button.setAttribute('aria-selected', 'true');

            tabContents.forEach((content) => {
                content.classList.remove('active');
            });
            document.getElementById(tabId)?.classList.add('active');
        });
    });
}
