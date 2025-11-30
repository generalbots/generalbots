# TABLE Keyword

The `TABLE` keyword defines database tables directly in your `.bas` files. Tables are automatically created on the specified database connection when the script is compiled.

## Syntax

```bas
TABLE TableName ON connection
    FieldName dataType[(length[,precision])] [key] [references OtherTable]
    ...
END TABLE
```

## Parameters

| Parameter | Description |
|-----------|-------------|
| `TableName` | Name of the table to create |
| `connection` | Connection name defined in config.csv (e.g., `maria`, `sales_db`) |
| `FieldName` | Name of the field/column |
| `dataType` | Data type (see supported types below) |
| `length` | Optional length for string/number types |
| `precision` | Optional decimal precision for number types |
| `key` | Marks field as primary key |
| `references` | Creates a foreign key reference to another table |

## Supported Data Types

| Type | Description | SQL Mapping |
|------|-------------|-------------|
| `string(n)` | Variable-length string | VARCHAR(n) |
| `number` | Integer | INTEGER |
| `number(n)` | Big integer | BIGINT |
| `number(n,p)` | Decimal with precision | DECIMAL(n,p) |
| `integer` | Integer | INTEGER |
| `double` | Double precision float | DOUBLE PRECISION |
| `double(n,p)` | Decimal | DECIMAL(n,p) |
| `date` | Date only | DATE |
| `datetime` | Date and time | TIMESTAMP/DATETIME |
| `boolean` | True/false | BOOLEAN |
| `text` | Long text | TEXT |
| `guid` | UUID | UUID/CHAR(36) |

## Connection Configuration

External database connections are configured in `config.csv` with the following format:

| Key | Description |
|-----|-------------|
| `conn-{name}-Server` | Database server hostname or IP |
| `conn-{name}-Name` | Database name |
| `conn-{name}-Username` | Username for authentication |
| `conn-{name}-Password` | Password for authentication |
| `conn-{name}-Port` | Port number (optional, uses default) |
| `conn-{name}-Driver` | Database driver: `mysql`, `mariadb`, `postgres`, `mssql` |

### Example config.csv

```csv
conn-maria-Server,192.168.1.100
conn-maria-Name,sales_database
conn-maria-Username,app_user
conn-maria-Password,secure_password
conn-maria-Port,3306
conn-maria-Driver,mariadb
```

## Examples

### Basic Table Definition

```bas
TABLE Contacts ON maria
    Id number key
    Nome string(150)
    Email string(255)
    Telefone string(20)
    DataCadastro date
END TABLE
```

### Table with Multiple Field Types

```bas
TABLE Produtos ON maria
    Id number key
    Nome string(150)
    Sku string(20)
    Preco double(10,2)
    Estoque integer
    Ativo boolean
    DescricaoCurta string(4000)
    DataValidade date
    Categoria_id integer
END TABLE
```

### Table with Foreign Key References

```bas
TABLE Pedidos ON maria
    Id number key
    Numero integer
    Data date
    Total double(15,2)
    Contato_id number
    Situacao_id integer
    Vendedor_id number
END TABLE

TABLE PedidosItem ON maria
    Id number key
    Pedido_id number
    Produto_id number
    Quantidade integer
    Valor double(10,2)
    Desconto double(5,2)
END TABLE
```

### Complete CRM Tables Example

```bas
' Contact management tables
TABLE Contatos ON maria
    Id number key
    Nome string(150)
    Codigo string(50)
    Situacao string(5)
    NumeroDocumento string(25)
    Telefone string(20)
    Celular string(20)
    Email string(50)
    Endereco_geral_endereco string(100)
    Endereco_geral_cep string(10)
    Endereco_geral_bairro string(50)
    Endereco_geral_municipio string(50)
    Endereco_geral_uf string(5)
    Vendedor_id number
    DadosAdicionais_dataNascimento date
    Financeiro_limiteCredito double
END TABLE

' Payment methods
TABLE FormaDePagamento ON maria
    Id number key
    Descricao string(255)
    TipoPagamento integer
    Situacao integer
    Padrao integer
    Taxas_aliquota double
    Taxas_valor double
END TABLE

' Accounts receivable
TABLE ContasAReceber ON maria
    Id number key
    Situacao integer
    Vencimento date
    Valor double
    Contato_id number
    FormaPagamento_id number
    Saldo double
    DataEmissao date
    NumeroDocumento string(50)
END TABLE
```

## Using Tables After Creation

Once tables are defined, you can use standard BASIC keywords to work with the data:

### Inserting Data

```bas
data = NEW OBJECT
data.Nome = "João Silva"
data.Email = "joao@example.com"
data.Telefone = "11999999999"
INSERT "Contatos", data
```

### Finding Data

```bas
contacts = FIND "Contatos", "Situacao='A'"
FOR EACH contact IN contacts
    TALK "Name: " + contact.Nome
NEXT
```

### Updating Data

```bas
UPDATE "Contatos", "Id=123", "Telefone='11988888888'"
```

### Deleting Data

```bas
DELETE "Contatos", "Id=123"
```

## Notes

1. **Automatic Table Creation**: Tables are created automatically when the `.bas` file is compiled. If the table already exists, no changes are made.

2. **Connection Required**: The connection name must be configured in `config.csv` before using it in TABLE definitions.

3. **Primary Keys**: Fields marked with `key` become the primary key. Multiple fields can be marked as key for composite primary keys.

4. **Default Connection**: If `ON connection` is omitted, the table is created on the default (internal) PostgreSQL database.

5. **SQL Injection Protection**: All identifiers are sanitized to prevent SQL injection attacks.

## See Also

- [FIND](./keyword-find.md) - Query data from tables
- [SAVE](./keyword-save.md) - Insert or update data
- [INSERT](./keyword-insert.md) - Insert new records
- [UPDATE](./keyword-update.md) - Update existing records
- [DELETE](./keyword-delete.md) - Delete records
- [config.csv](../chapter-08-config/config-csv.md) - Connection configuration