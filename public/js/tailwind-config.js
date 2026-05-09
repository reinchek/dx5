tailwind.config = {
    theme: {
        extend: {
            colors: {
                dusk: {
                    900: '#120d1a', // Sfondo profondo
                    800: '#1a1325', // Sidebar / Card
                    700: '#251b36', // Hover
                },
                rust: {
                    500: '#b7410e', // Accento principale
                    400: '#d95d26',
                },
                terminal: {
                    green: '#4ade80',
                    red: '#f87171',
                }
            },
            fontFamily: {
                mono: ['"JetBrains Mono"', 'monospace'],
            },
            animation: {
                'glitch-top': 'glitchTop 4s steps(1) infinite',
                'glitch-bot': 'glitchBot 4s steps(1) infinite',
                'scanlines':  'scanroll 8s linear infinite',
            },
            keyframes: {
                glitchTop: {
                    '0%, 8%':  { transform: 'translate(0,0)', opacity: '0' },
                    '9%':      { transform: 'translate(-4px,-2px)', opacity: '0.9' },
                    '11%':     { transform: 'translate(0,0)', opacity: '0' },
                    // ...
                }
            }
        }
    }
}