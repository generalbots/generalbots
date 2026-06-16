# Issue #005: PRODUCTS — Missing partial file (prefix inconsistency resolved)

**Severity:** LOW
**Components:** `botui/ui/suite/products/`
**Type:** Ghost functionality (partial)

## Description

The Products app has one real issue and one that was resolved by the crate audit.

---

### 1. Missing Partial File ⚠️ REAL
**File:** `products.html` line 181 references `/suite/products/partials/pricelist-form.html`
**Reality:** The file **does not exist** on disk.

When users click "New Price List", the modal will attempt to load this partial and get a 404.

### 2. Prefix Inconsistency — RESOLVED (after crate audit)

**Reportado inicialmente:** GET usa `/api/ui/products/items` e POST usa `/api/products/items` — prefixos inconsistentes.

**Realidade:** O crate `botproducts` registra **ambos os conjuntos de rotas** no servidor:
- `configure_products_routes()` → `/api/ui/products/items`, `/api/ui/products/services`, `/api/ui/products/stats/*`, etc. (8 rotas)
- `configure_products_api_routes()` → `/api/products/items`, `/api/products/services`, `/api/products/categories`, `/api/products/price-lists`, etc. (23 rotas)

Ambas são chamadas em `server.rs` sob `#[cfg(feature = "billing")]`. Portanto, **GET em `/api/ui/products/items` E POST em `/api/products/items` ambos funcionam**. A inconsistência é cosmética (design) e não funcional.

**No entanto**, isso ainda é uma má prática: dois prefixos para o mesmo recurso criam confusão. Deveria haver um padrão único.

---

## Impact (Revised)

- **Price List creation está quebrado** (partial file ausente).
- Prefixos funcionam ambos, mas a duplicação confunde desenvolvedores.
- Feature gate `billing` pode não ser intuitivo para Products.

## Suggested Fix

1. **Criar** o partial `pricelist-form.html` ausente ou remover a referência.
2. **Escolher** um prefixo canônico para Products e remover o outro.
3. **Renomear** a feature flag de `billing` para `products` se Products não for só billing.
