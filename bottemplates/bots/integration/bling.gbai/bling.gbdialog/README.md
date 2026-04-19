# Bling ERP Integration (.gbdialog)

This package provides complete integration with [Bling ERP](https://www.bling.com.br/) for data synchronization and conversational commerce.

## Scripts

| File | Description |
|------|-------------|
| `start.bas` | Welcome message and system prompt configuration |
| `tables.bas` | Database schema definitions for all synced entities |
| `sync-erp.bas` | Main ERP synchronization (products, orders, contacts, vendors) |
| `sync-accounts.bas` | Accounts payable and receivable synchronization |
| `sync-inventory.bas` | Stock/inventory levels synchronization |
| `sync-suppliers.bas` | Supplier/vendor data synchronization |
| `add-stock.bas` | Manual stock adjustment tool |
| `data-analysis.bas` | LLM-powered data analysis and reporting |
| `refresh-llm.bas` | Scheduled LLM context refresh |

## Configuration

Configure the integration in `bling.gbot/config.csv`:

| Parameter | Description |
|-----------|-------------|
| `param-blingClientID` | Bling API Client ID |
| `param-blingClientSecret` | Bling API Client Secret |
| `param-blingHost` | Bling API base URL |
| `param-host` | API endpoint (default: `https://api.bling.com.br/Api/v3`) |
| `param-limit` | Records per page for API calls |
| `param-pages` | Maximum pages to sync |
| `param-admin1` | Primary admin email for notifications |
| `param-admin2` | Secondary admin email for notifications |

## Synchronized Entities

### Products (`maria.Produtos`)
- Product details, SKU, pricing
- Product variations and hierarchy
- Product images (`maria.ProdutoImagem`)

### Orders (`maria.Pedidos`)
- Sales orders with line items (`maria.PedidosItem`)
- Payment parcels (`maria.Parcela`)

### Contacts (`maria.Contatos`)
- Customers and suppliers
- Address and billing information

### Vendors (`maria.Vendedores`)
- Sales representatives
- Commission and discount limits

### Financial
- Accounts Receivable (`maria.ContasAReceber`)
- Accounts Payable (`maria.ContasAPagar`)
- Payment Methods (`maria.FormaDePagamento`)
- Revenue Categories (`maria.CategoriaReceita`)

### Inventory
- Stock by Warehouse (`maria.Depositos`)
- Product Suppliers (`maria.ProdutoFornecedor`)
- Price History (`maria.HistoricoPreco`)

## Scheduled Jobs

The following schedules are configured:

| Job | Schedule | Description |
|-----|----------|-------------|
| `sync-erp.bas` | Daily at 22:30 | Full ERP synchronization |
| `sync-accounts.bas` | Every 2 days at midnight | Financial accounts sync |
| `sync-inventory.bas` | Daily at 23:30 | Stock levels update |
| `refresh-llm.bas` | Daily at 21:00 | Refresh LLM context |

## Data Analysis

The `data-analysis.bas` script enables natural language queries against synced data:

**Example queries:**
- "Which products have excess stock that can be transferred?"
- "What are the top 10 best-selling products?"
- "What is the average ticket for each store?"
- "Which products need restocking?"

## Usage

### Manual Stock Adjustment

Vendors can adjust stock via conversation:

```basic
REM User provides SKU and quantity
REM Stock is updated in Bling and local database
```

### Running Sync Manually

```basic
RUN "sync-erp.bas"
RUN "sync-accounts.bas"
RUN "sync-inventory.bas"
```

## API Integration

All API calls use the Bling v3 REST API with pagination support:

- Products: `GET /produtos`
- Orders: `GET /pedidos/vendas`
- Contacts: `GET /contatos`
- Inventory: `GET /estoques/saldos`
- Accounts: `GET /contas/receber`, `GET /contas/pagar`

## Related Documentation

- [Bling API Documentation](https://developer.bling.com.br/)
- [General Bots BASIC Reference](../../docs/src/chapter-06-gbdialog/README.md)
- [Template Guide](../../README.md)