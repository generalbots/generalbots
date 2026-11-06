            // Carregar o modal New Project dinamicamente (Issue #524)
            fetch('/suite/partials/new-project.html')
                .then(response => response.text())
                .then(html => {
                    const div = document.createElement('div');
                    div.innerHTML = html;
                    document.body.appendChild(div);
                    
                    // Executar scripts embutidos no modal carregado
                    const scripts = div.querySelectorAll('script');
                    scripts.forEach(script => {
                        const newScript = document.createElement('script');
                        newScript.textContent = script.textContent;
                        document.body.appendChild(newScript);
                    });
                })
                .catch(err => console.error('Failed to load new-project wizard:', err));
        
