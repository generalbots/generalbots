/* =============================================================================
   MAIL APP - SENTIENT THEME JAVASCRIPT
   ============================================================================= */

(function() {
    'use strict';

    // =============================================================================
    // AI PANEL (Collapsible, Mobile-friendly)
    // =============================================================================

    window.toggleAIPanel = function() {
        const panel = document.getElementById('ai-panel');
        const toggle = document.querySelector('.ai-toggle');
        if (panel) {
            panel.classList.toggle('collapsed');
            if (toggle) {
                toggle.classList.toggle('active', !panel.classList.contains('collapsed'));
            }
            localStorage.setItem('aiPanelCollapsed', panel.classList.contains('collapsed'));
        }
    };

    window.sendAIMessage = function() {
        const input = document.getElementById('ai-input');
        if (!input || !input.value.trim()) return;

        const message = input.value.trim();
        input.value = '';

        addMessage('user', message);
        showTypingIndicator();

        setTimeout(() => {
            hideTypingIndicator();
            addMessage('assistant', `Processando: "${message}". Como posso ajudar mais?`);
        }, 1500);
    };

    window.aiAction = function(action) {
        const actions = {
            'summarize': 'Resumindo o email selecionado...',
            'reply': 'Gerando uma resposta profissional...',
            'translate': 'Traduzindo o conteúdo...',
            'organize': 'Organizando sua caixa de entrada...'
        };
        
        addMessage('assistant', actions[action] || 'Processando...');
        
        setTimeout(() => {
            addMessage('assistant', 'Ação concluída com sucesso! O que mais posso fazer?');
        }, 2000);
    };

    function addMessage(type, content) {
        const container = document.getElementById('ai-messages');
        if (!container) return;

        const messageEl = document.createElement('div');
        messageEl.className = `ai-message ${type}`;
        messageEl.innerHTML = `<div class="ai-message-bubble">${content}</div>`;
        container.appendChild(messageEl);
        container.scrollTop = container.scrollHeight;
    }

    function showTypingIndicator() {
        const container = document.getElementById('ai-messages');
        if (!container) return;

        const indicator = document.createElement('div');
        indicator.className = 'ai-message assistant';
        indicator.id = 'typing-indicator';
        indicator.innerHTML = `
            <div class="ai-typing-indicator">
                <span></span><span></span><span></span>
            </div>
        `;
        container.appendChild(indicator);
        container.scrollTop = container.scrollHeight;
    }

    function hideTypingIndicator() {
        const indicator = document.getElementById('typing-indicator');
        if (indicator) indicator.remove();
    }

    // =============================================================================
    // EMAIL FUNCTIONS
    // =============================================================================

    window.composeEmail = function() {
        addMessage('assistant', 'Abrindo composer de email. Deseja que eu escreva um rascunho?');
    };

    function initEmailList() {
        document.querySelectorAll('.email-item').forEach(item => {
            item.addEventListener('click', function(e) {
                if (e.target.type === 'checkbox' || e.target.classList.contains('email-star')) return;
                
                document.querySelectorAll('.email-item').forEach(i => i.classList.remove('selected'));
                this.classList.add('selected');
                this.classList.remove('unread');
            });
        });

        document.querySelectorAll('.email-star').forEach(star => {
            star.addEventListener('click', function(e) {
                e.stopPropagation();
                this.classList.toggle('starred');
                this.textContent = this.classList.contains('starred') ? '⭐' : '☆';
            });
        });
    }

    // =============================================================================
    // APP NAVIGATION
    // =============================================================================

    function initAppLauncher() {
        document.querySelectorAll('.app-icon').forEach(icon => {
            icon.addEventListener('click', function() {
                const app = this.dataset.app;
                if (app === 'chat') {
                    window.location.href = '/suite/chat/chat-sentient.html';
                } else if (app === 'drive') {
                    window.location.href = '/suite/drive/drive-sentient.html';
                } else if (app === 'tasks') {
                    window.location.href = '/suite/tasks/tasks-sentient.html';
                } else if (app === 'calendar') {
                    window.location.href = '/suite/calendar/calendar-sentient.html';
                } else if (app === 'meet') {
                    window.location.href = '/suite/meet/meet-sentient.html';
                } else if (app === 'paper') {
                    window.location.href = '/suite/paper/paper-sentient.html';
                }
            });
        });
    }

    function initTabs() {
        document.querySelectorAll('.topbar-tab').forEach(tab => {
            tab.addEventListener('click', function() {
                document.querySelectorAll('.topbar-tab').forEach(t => t.classList.remove('active'));
                this.classList.add('active');
            });
        });
    }

    // =============================================================================
    // KEYBOARD SHORTCUTS
    // =============================================================================

    function initKeyboardShortcuts() {
        document.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' && document.activeElement.id === 'ai-input') {
                e.preventDefault();
                sendAIMessage();
            }
            
            // Ctrl+Shift+A to toggle AI panel
            if (e.ctrlKey && e.shiftKey && e.key === 'A') {
                e.preventDefault();
                toggleAIPanel();
            }
        });
    }

    // =============================================================================
    // RESTORE AI PANEL STATE
    // =============================================================================

    function restoreAIPanelState() {
        const collapsed = localStorage.getItem('aiPanelCollapsed') === 'true';
        const panel = document.getElementById('ai-panel');
        const toggle = document.querySelector('.ai-toggle');
        
        if (panel && collapsed) {
            panel.classList.add('collapsed');
        }
        if (toggle) {
            toggle.classList.toggle('active', !collapsed);
        }
    }

    // =============================================================================
    // INITIALIZE
    // =============================================================================

    function init() {
        initEmailList();
        initAppLauncher();
        initTabs();
        initKeyboardShortcuts();
        restoreAIPanelState();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

})();
