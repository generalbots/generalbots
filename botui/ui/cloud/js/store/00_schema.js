  (async function() {
    try {
      const res = await fetch('/api/catalog/prices.json');
      if (res.ok) {
        const data = await res.json();
        const el = document.createElement('script');
        el.type = 'application/ld+json';
        el.id = 'product-schema';
        el.textContent = JSON.stringify(data);
        document.head.appendChild(el);
      }
    } catch(e) { /* JSON-LD not critical */ }
  })();
