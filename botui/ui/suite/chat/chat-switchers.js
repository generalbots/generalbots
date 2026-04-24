// Chat Switchers Module - Manages format switchers (tables, infographic, cards, etc.)
// Uses event delegation for better reliability with dynamic content

var activeSwitchers = new Set();
var switcherDefinitions = [];

function renderBotSwitchers(switchers) {
    if (!switchers || switchers.length === 0) return;
    
    var existingIds = {};
    switcherDefinitions.forEach(function(sw) { existingIds[sw.id] = true; });
    
    switchers.forEach(function(sw) {
        if (!existingIds[sw.id]) {
            switcherDefinitions.push({
                id: sw.id,
                label: sw.label || sw.id,
                icon: sw.icon || '🔀',
                color: sw.color || '#666'
            });
            existingIds[sw.id] = true;
        }
    });
    
    renderSwitchers();
    
    var container = document.getElementById("switchers");
    if (container && switcherDefinitions.length > 0) {
        container.style.display = '';
    }
}

function renderSwitchers() {
    var container = document.getElementById("switcherChips");
    if (!container) return;
    
    container.innerHTML = switcherDefinitions.map(function(sw) {
        var isActive = activeSwitchers.has(sw.id);
        return (
            '<div class="switcher-chip' + (isActive ? ' active' : '') + '" ' +
            'data-switch-id="' + sw.id + '" ' +
            'style="--switcher-color: ' + sw.color + '; ' +
            (isActive ? 'color: ' + sw.color + ' background: ' + sw.color + '; ' : '') + '">' +
            '<span class="switcher-chip-icon">' + sw.icon + '</span>' +
            '<span>' + sw.label + '</span>' +
            '</div>'
        );
    }).join('');
    
    // Event delegation - attach once to parent
    if (!container.dataset.hasClickHandler) {
        container.addEventListener('click', function(e) {
            var chip = e.target.closest('.switcher-chip');
            if (chip) {
                var switcherId = chip.getAttribute('data-switch-id');
                if (switcherId) {
                    toggleSwitcher(switcherId);
                }
            }
        });
        container.dataset.hasClickHandler = 'true';
    }
}

function toggleSwitcher(switcherId) {
    console.log('toggleSwitcher called with:', switcherId);
    console.log('activeSwitchers before:', Array.from(activeSwitchers));
    if (activeSwitchers.has(switcherId)) {
        activeSwitchers.delete(switcherId);
        console.log('Deleted switcher:', switcherId);
    } else {
        activeSwitchers.add(switcherId);
        console.log('Added switcher:', switcherId);
    }
    console.log('activeSwitchers after:', Array.from(activeSwitchers));
    
    // Re-render to show active state
    renderSwitchers();
}

function getActiveSwitcherIds() {
    var ids = Array.from(activeSwitchers);
    console.log('getActiveSwitcherIds returning:', ids);
    return ids;
}

function clearSwitchers() {
    activeSwitchers.clear();
    renderSwitchers();
}
