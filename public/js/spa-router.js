// spa-router.js - Simple, robust vanilla SPA router
// Requires: <div id="page-content"> inside the layout,
// and links with either data-spa attribute or inside #page-content.

(function() {
    // DOM elements
    const contentEl = document.getElementById('page-content');
    if (!contentEl) {
        console.warn('SPA: #page-content not found, router disabled');
        return;
    }

    // Helper: get full URL for relative href
    function getFullUrl(href) {
        if (!href || href.startsWith('http') || href.startsWith('//')) return href;
        // handle relative paths like /en/post/1
        return new URL(href, window.location.origin).href;
    }

    // Helper: is this an internal link we should handle?
    function isInternalLink(link) {
        const href = link.getAttribute('href');
        if (!href) return false;
        // skip anchors, external, protocol-relative, javascript:, mailto:, tel:
        if (href.startsWith('#') || href.startsWith('javascript:') || href.startsWith('mailto:') || href.startsWith('tel:')) return false;
        // external check
        const url = getFullUrl(href);
        return url.startsWith(window.location.origin);
    }

    // Core navigation function
    async function navigateTo(url, pushState = true) {
        // Prevent duplicate navigation to same URL
        if (url === window.location.href && url !== window.location.pathname) return;

        console.debug('[SPA] navigating to', url);
        try {
            const response = await fetch(url, {
                headers: { 'X-Requested-With': 'spa' },
                credentials: 'same-origin'   // include cookies if needed
            });
            if (!response.ok) {
                console.warn('[SPA] fetch failed, fallback to full reload');
                window.location.href = url;
                return;
            }

            const html = await response.text();
            const parser = new DOMParser();
            const doc = parser.parseFromString(html, 'text/html');

            const newContent = doc.getElementById('page-content');
            if (!newContent) {
                console.warn('[SPA] #page-content missing, fallback');
                window.location.href = url;
                return;
            }

            // Swap content
            contentEl.innerHTML = newContent.innerHTML;

            // Update title
            const newTitle = doc.querySelector('title');
            if (newTitle) document.title = newTitle.textContent;

            // Update URL bar
            if (pushState) {
                history.pushState({ url: url }, '', url);
            }

            // Scroll to top
            contentEl.scrollIntoView({ behavior: 'instant', block: 'start' });

            // Re-run syntax highlighting if needed
            if (typeof hljs !== 'undefined' && hljs.highlightAll) {
                hljs.highlightAll();
            }

            console.debug('[SPA] navigation completed');
        } catch (error) {
            console.error('[SPA] exception:', error);
            window.location.href = url;
        }
    }

    // Global click delegation
    document.body.addEventListener('click', (e) => {
        // Find closest anchor
        const link = e.target.closest('a[href]');
        if (!link) return;

        // Determine if we should handle it
        let shouldHandle = false;
        if (link.hasAttribute('data-spa')) {
            shouldHandle = true;
        } else if (contentEl.contains(link) && isInternalLink(link)) {
            shouldHandle = true;
        }

        if (!shouldHandle) return;

        const href = link.getAttribute('href');
        const url = getFullUrl(href);
        // Prevent full reload
        e.preventDefault();
        navigateTo(url);
    });

    // Popstate (back/forward)
    window.addEventListener('popstate', (e) => {
        const url = e.state?.url || window.location.pathname;
        console.debug('[SPA] popstate', url);
        navigateTo(url, false);
    });

    // Initialize history state on first load
    history.replaceState({ url: window.location.href }, '', window.location.href);

    console.debug('[SPA] router initialized');
})();