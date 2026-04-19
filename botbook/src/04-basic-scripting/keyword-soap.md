# SOAP

The `SOAP` keyword enables bots to communicate with legacy SOAP/XML web services, allowing integration with enterprise systems, government APIs, and older corporate infrastructure that still relies on SOAP protocols.

---

## Syntax

```basic
result = SOAP "wsdl_url", "operation", params
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `wsdl_url` | String | URL to the WSDL file or SOAP endpoint |
| `operation` | String | Name of the SOAP operation to call |
| `params` | Object | Parameters to pass to the operation |

---

## Description

`SOAP` sends a SOAP (Simple Object Access Protocol) request to a web service, automatically building the XML envelope and parsing the response. This enables integration with legacy enterprise systems that haven't migrated to REST APIs.

Use cases include:
- Connecting to government tax and fiscal systems
- Integrating with legacy ERP systems (SAP, Oracle)
- Communicating with banking and payment systems
- Accessing healthcare HL7/SOAP interfaces
- Interfacing with older CRM systems

---

## Examples

### Basic SOAP Request

```basic
' Call a simple SOAP service
result = SOAP "https://api.example.com/service?wsdl", "GetUserInfo", #{
    "userId": "12345"
}

TALK "User name: " + result.name
```

### Tax Calculation Service

```basic
' Brazilian NF-e fiscal service example
nfe_params = #{
    "CNPJ": company_cnpj,
    "InvoiceNumber": invoice_number,
    "Items": invoice_items,
    "TotalValue": total_value
}

result = SOAP "https://nfe.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx?wsdl",
    "NfeAutorizacao",
    nfe_params

IF result.status = "Authorized" THEN
    TALK "Invoice authorized! Protocol: " + result.protocol
ELSE
    TALK "Authorization failed: " + result.errorMessage
END IF
```

### Currency Exchange Service

```basic
' Get exchange rates from central bank
params = #{
    "fromCurrency": "USD",
    "toCurrency": "BRL",
    "date": FORMAT(NOW(), "YYYY-MM-DD")
}

result = SOAP "https://www.bcb.gov.br/webservice/cotacao.asmx?wsdl",
    "GetCotacao",
    params

rate = result.cotacao.valor
TALK "Today's USD/BRL rate: " + rate
```

### Weather Service (Legacy)

```basic
' Access legacy weather SOAP service
weather_params = #{
    "city": city_name,
    "country": "BR"
}

result = SOAP "https://weather.example.com/service.asmx?wsdl",
    "GetWeather",
    weather_params

TALK "Weather in " + city_name + ": " + result.description
TALK "Temperature: " + result.temperature + "°C"
```

### SAP Integration

```basic
' Query SAP for material information
sap_params = #{
    "MaterialNumber": material_code,
    "Plant": "1000"
}

result = SOAP "https://sap.company.com:8443/sap/bc/srt/wsdl/MATERIAL_INFO?wsdl",
    "GetMaterialDetails",
    sap_params

material = result.MaterialData
TALK "Material: " + material.Description
TALK "Stock: " + material.AvailableStock + " units"
TALK "Price: $" + material.StandardPrice
```

---

## Working with Complex Types

### Nested Objects

```basic
' SOAP request with nested structure
customer_data = #{
    "Customer": #{
        "Name": customer_name,
        "Address": #{
            "Street": street,
            "City": city,
            "ZipCode": zipcode,
            "Country": "BR"
        },
        "Contact": #{
            "Email": email,
            "Phone": phone
        }
    }
}

result = SOAP "https://crm.company.com/CustomerService.asmx?wsdl",
    "CreateCustomer",
    customer_data

TALK "Customer created with ID: " + result.CustomerId
```

### Array Parameters

```basic
' Send multiple items in SOAP request
order_items = [
    #{ "SKU": "PROD-001", "Quantity": 2, "Price": 99.99 },
    #{ "SKU": "PROD-002", "Quantity": 1, "Price": 49.99 },
    #{ "SKU": "PROD-003", "Quantity": 5, "Price": 19.99 }
]

order_params = #{
    "OrderHeader": #{
        "CustomerId": customer_id,
        "OrderDate": FORMAT(NOW(), "YYYY-MM-DD")
    },
    "OrderItems": order_items
}

result = SOAP "https://erp.company.com/OrderService?wsdl",
    "CreateOrder",
    order_params

TALK "Order " + result.OrderNumber + " created successfully!"
```

---

## Response Handling

### Parsing Complex Responses

```basic
' Handle structured SOAP response
result = SOAP "https://api.example.com/InvoiceService?wsdl",
    "GetInvoices",
    #{ "CustomerId": customer_id, "Year": 2024 }

' Access nested response data
FOR EACH invoice IN result.Invoices.Invoice
    TALK "Invoice #" + invoice.Number + " - $" + invoice.Total
    TALK "  Date: " + invoice.Date
    TALK "  Status: " + invoice.Status
END FOR
```

### Checking Response Status

```basic
result = SOAP service_url, operation, params

IF result.ResponseCode = "0" OR result.Success = true THEN
    TALK "Operation completed successfully"
    ' Process result data
ELSE
    TALK "Operation failed: " + result.ErrorMessage
END IF
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = SOAP "https://legacy.system.com/service.asmx?wsdl",
    "ProcessPayment",
    payment_params

IF ERROR THEN
    error_msg = ERROR_MESSAGE
    
    IF INSTR(error_msg, "timeout") > 0 THEN
        TALK "The service is taking too long. Please try again."
    ELSE IF INSTR(error_msg, "WSDL") > 0 THEN
        TALK "Cannot connect to the service. It may be down."
    ELSE IF INSTR(error_msg, "authentication") > 0 THEN
        TALK "Authentication failed. Please check credentials."
    ELSE
        TALK "Service error: " + error_msg
    END IF
ELSE
    IF result.TransactionId THEN
        TALK "Payment processed! Transaction: " + result.TransactionId
    END IF
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `WSDL_PARSE_ERROR` | Invalid WSDL format | Verify WSDL URL and format |
| `SOAP_FAULT` | Service returned fault | Check error message from service |
| `TIMEOUT` | Request took too long | Increase timeout or retry |
| `CONNECTION_ERROR` | Cannot reach service | Check network and URL |
| `AUTHENTICATION_ERROR` | Invalid credentials | Verify authentication headers |

---

## Authentication

SOAP services commonly use several authentication methods. General Bots supports all major approaches.

### Basic Authentication

The simplest form of authentication, sending username and password with each request:

```basic
' Basic HTTP authentication
SET HEADER "Authorization", "Basic " + BASE64(username + ":" + password)

result = SOAP service_url, operation, params

CLEAR HEADERS
```

### WS-Security (Username Token)

WS-Security adds security tokens directly to the SOAP envelope. Configure in `config.csv`:

```csv
name,value
soap-wsse-enabled,true
soap-wsse-username,your_username
soap-wsse-password,your_password
soap-wsse-password-type,PasswordDigest
```

**Password Types:**
- `PasswordText` - Password sent in plain text (use only with HTTPS)
- `PasswordDigest` - Password hashed with nonce and timestamp (recommended)

**Usage:**

```basic
' WS-Security is applied automatically when configured
result = SOAP "https://secure.service.com/api?wsdl", "SecureOperation", params

' The SOAP envelope will include:
' <wsse:Security>
'   <wsse:UsernameToken>
'     <wsse:Username>your_username</wsse:Username>
'     <wsse:Password Type="...">hashed_password</wsse:Password>
'     <wsse:Nonce>...</wsse:Nonce>
'     <wsu:Created>...</wsu:Created>
'   </wsse:UsernameToken>
' </wsse:Security>
```

### WS-Security with Timestamp

Add timestamp validation to prevent replay attacks:

```csv
name,value
soap-wsse-enabled,true
soap-wsse-username,your_username
soap-wsse-password,your_password
soap-wsse-timestamp,true
soap-wsse-timestamp-ttl,300
```

The `timestamp-ttl` sets validity in seconds (default: 300 = 5 minutes).

### Certificate-Based Authentication (Mutual TLS)

For services requiring client certificates:

```csv
name,value
soap-client-cert,/path/to/client.pem
soap-client-key,/path/to/client.key
soap-client-key-password,optional_key_password
soap-ca-cert,/path/to/ca.pem
soap-verify-ssl,true
```

**Certificate Formats Supported:**
- PEM (`.pem`, `.crt`, `.cer`)
- PKCS#12 (`.p12`, `.pfx`) - set `soap-client-cert-type,p12`

**Example for Brazilian NFe:**

```csv
name,value
soap-client-cert,/certs/certificado_a1.pfx
soap-client-cert-type,p12
soap-client-key-password,cert_password
soap-ca-cert,/certs/cadeia_nfe.pem
```

### OAuth 2.0 Authentication

For modern SOAP services that support OAuth:

```csv
name,value
soap-oauth-enabled,true
soap-oauth-token-url,https://auth.service.com/oauth/token
soap-oauth-client-id,your_client_id
soap-oauth-client-secret,your_client_secret
soap-oauth-scope,soap_api
```

**Or provide token directly:**

```basic
' Get OAuth token first
token_response = POST "https://auth.service.com/oauth/token", #{
    "grant_type": "client_credentials",
    "client_id": client_id,
    "client_secret": client_secret
}

' Use token for SOAP call
SET HEADER "Authorization", "Bearer " + token_response.access_token

result = SOAP service_url, operation, params

CLEAR HEADERS
```

### API Key Authentication

Some SOAP services use API keys:

```basic
' API key in header
SET HEADER "X-API-Key", api_key

result = SOAP service_url, operation, params

CLEAR HEADERS
```

**Or configure in config.csv:**

```csv
name,value
soap-api-key,your_api_key
soap-api-key-header,X-API-Key
```

### SAML Token Authentication

For enterprise SSO with SAML:

```csv
name,value
soap-saml-enabled,true
soap-saml-assertion-url,https://idp.company.com/saml/assertion
soap-saml-issuer,https://your-bot.example.com
```

### Custom SOAP Headers

For services requiring custom security headers:

```basic
' Add custom SOAP header
SET HEADER "SOAPAction", "urn:processPayment"
SET HEADER "X-Custom-Auth", custom_auth_value

result = SOAP service_url, operation, params

CLEAR HEADERS
```

### Authentication Examples by Industry

#### Government/Fiscal Services (NFe, NFS-e)

```csv
name,value
soap-client-cert,/certs/e-cnpj-a1.pfx
soap-client-cert-type,p12
soap-client-key-password,certificate_password
soap-ca-cert,/certs/ac-raiz.pem
soap-wsse-enabled,false
```

#### Banking/Financial Services

```csv
name,value
soap-wsse-enabled,true
soap-wsse-username,bank_user
soap-wsse-password,bank_password
soap-wsse-password-type,PasswordDigest
soap-wsse-timestamp,true
soap-client-cert,/certs/bank-client.pem
soap-client-key,/certs/bank-client.key
```

#### Healthcare (HL7/SOAP)

```csv
name,value
soap-wsse-enabled,true
soap-wsse-username,hl7_system_user
soap-wsse-password,hl7_system_password
soap-timeout,60
```

#### Legacy ERP (SAP, Oracle)

```csv
name,value
soap-auth-type,basic
soap-username,erp_integration_user
soap-password,erp_integration_password
soap-timeout,120
```

### Configuration Reference

| Parameter | Description | Default |
|-----------|-------------|---------|
| `soap-timeout` | Request timeout in seconds | `120` |
| `soap-verify-ssl` | Verify SSL certificates | `true` |
| `soap-wsse-enabled` | Enable WS-Security | `false` |
| `soap-wsse-username` | WS-Security username | Not set |
| `soap-wsse-password` | WS-Security password | Not set |
| `soap-wsse-password-type` | `PasswordText` or `PasswordDigest` | `PasswordDigest` |
| `soap-wsse-timestamp` | Include timestamp | `false` |
| `soap-wsse-timestamp-ttl` | Timestamp validity (seconds) | `300` |
| `soap-client-cert` | Path to client certificate | Not set |
| `soap-client-key` | Path to client private key | Not set |
| `soap-client-key-password` | Password for private key | Not set |
| `soap-client-cert-type` | Certificate type (`pem`, `p12`) | `pem` |
| `soap-ca-cert` | Path to CA certificate | Not set |
| `soap-oauth-enabled` | Enable OAuth authentication | `false` |
| `soap-api-key` | API key value | Not set |
| `soap-api-key-header` | Header name for API key | `X-API-Key` |

---

## Practical Examples

### Brazilian NFe (Electronic Invoice)

```basic
' Emit electronic invoice to Brazilian tax authority
nfe_data = #{
    "infNFe": #{
        "ide": #{
            "cUF": "35",
            "natOp": "VENDA",
            "serie": "1",
            "nNF": invoice_number
        },
        "emit": #{
            "CNPJ": company_cnpj,
            "xNome": company_name
        },
        "dest": #{
            "CNPJ": customer_cnpj,
            "xNome": customer_name
        },
        "det": invoice_items,
        "total": #{
            "vNF": total_value
        }
    }
}

result = SOAP "https://nfe.fazenda.sp.gov.br/ws/NFeAutorizacao4.asmx?wsdl",
    "nfeAutorizacaoLote",
    nfe_data

IF result.cStat = "100" THEN
    TALK "NFe authorized! Key: " + result.chNFe
ELSE
    TALK "Error: " + result.xMotivo
END IF
```

### Healthcare HL7/SOAP

```basic
' Query patient information from healthcare system
patient_query = #{
    "PatientId": patient_id,
    "IncludeHistory": true
}

result = SOAP "https://hospital.example.com/PatientService?wsdl",
    "GetPatientRecord",
    patient_query

TALK "Patient: " + result.Patient.Name
TALK "DOB: " + result.Patient.DateOfBirth
TALK "Allergies: " + JOIN(result.Patient.Allergies, ", ")
```

### Legacy CRM Integration

```basic
' Update customer in legacy Siebel CRM
update_data = #{
    "AccountId": account_id,
    "AccountName": new_name,
    "PrimaryContact": #{
        "FirstName": first_name,
        "LastName": last_name,
        "Email": email
    },
    "UpdatedBy": bot_user
}

result = SOAP "https://siebel.company.com/eai_enu/start.swe?SWEExtSource=WebService&wsdl",
    "AccountUpdate",
    update_data

TALK "CRM updated. Transaction ID: " + result.TransactionId
```

---

## SOAP vs REST

| Aspect | SOAP | REST |
|--------|------|------|
| Protocol | XML-based | JSON typically |
| Standards | WS-Security, WS-*, WSDL | OpenAPI, OAuth |
| Use Case | Enterprise, legacy | Modern APIs |
| Keyword | `SOAP` | `POST`, `GET` |
| Complexity | Higher | Lower |

**When to use SOAP:**
- Integrating with legacy enterprise systems
- Government/fiscal APIs requiring SOAP
- Systems with strict WS-Security requirements
- Banking and financial services
- Healthcare systems (HL7 SOAP)

---

## Configuration

No specific configuration required. The keyword handles SOAP envelope construction automatically.

For services requiring custom SOAP headers or namespaces, these are inferred from the WSDL.

---

## Implementation Notes

- Implemented in Rust under `src/basic/keywords/http_operations.rs`
- Automatically fetches and parses WSDL
- Builds SOAP envelope from parameters
- Parses XML response into JSON-like object
- Timeout: 120 seconds by default
- Supports SOAP 1.1 and 1.2

---

## Related Keywords

- [POST](keyword-post.md) — For REST API calls
- [GET](keyword-get.md) — For REST GET requests
- [GRAPHQL](keyword-graphql.md) — For GraphQL APIs
- [SET HEADER](keyword-set-header.md) — Set authentication headers

---

## Summary

`SOAP` enables integration with legacy SOAP/XML web services that are still common in enterprise, government, and healthcare sectors. While REST is preferred for modern APIs, SOAP remains essential for connecting to fiscal systems (NFe, tax services), legacy ERPs (SAP, Oracle), and older enterprise infrastructure. The keyword handles XML envelope construction and parsing automatically, making SOAP integration as simple as REST calls.