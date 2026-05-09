(function () {
    document.querySelectorAll('.lang-switch').forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            console.log('clicked');

            const newLang = link.getAttribute('data-lang');
            if (!newLang) return;
            console.log('newLang='+newLang);

            // Current path
            let path = window.location.pathname;
            console.log('currentpath='+path);
            // Match the first segment
            const match = path.match(/^\/([a-z]{2})(\/|$)/);
            if (match) {
                console.log('matched');
                path = path.replace(/^\/[a-z]{2}/, `/${newLang}`);
                console.log('new_path='+path);
            } else {
                path = `/${newLang}${path}`;
                console.log('notmatched path='+path);
            }

            // Navigate using SPA router if available, otherwise full reload.
            if (typeof navigateTo === 'function') {
                navigateTo(path);
            } else {
                window.location.href = path;
            }
        })
    });
})()