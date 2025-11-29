#!/bin/bash

# TLS/HTTPS Setup Script for BotServer
# This script sets up a complete TLS infrastructure with internal CA and certificates for all services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CERT_DIR="./certs"
CA_DIR="$CERT_DIR/ca"
VALIDITY_DAYS=365
COUNTRY="BR"
STATE="SP"
LOCALITY="São Paulo"
ORGANIZATION="BotServer"
COMMON_NAME_SUFFIX="botserver.local"

# Services that need certificates
SERVICES=(
    "api:8443:localhost,api.botserver.local,127.0.0.1"
    "llm:8444:localhost,llm.botserver.local,127.0.0.1"
    "embedding:8445:localhost,embedding.botserver.local,127.0.0.1"
    "qdrant:6334:localhost,qdrant.botserver.local,127.0.0.1"
    "redis:6380:localhost,redis.botserver.local,127.0.0.1"
    "postgres:5433:localhost,postgres.botserver.local,127.0.0.1"
    "minio:9001:localhost,minio.botserver.local,127.0.0.1"
    "directory:8446:localhost,directory.botserver.local,127.0.0.1"
    "email:465:localhost,email.botserver.local,127.0.0.1"
    "meet:7881:localhost,meet.botserver.local,127.0.0.1"
)

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}BotServer TLS/HTTPS Setup${NC}"
echo -e "${BLUE}========================================${NC}"

# Function to check if OpenSSL is installed
check_openssl() {
    if ! command -v openssl &> /dev/null; then
        echo -e "${RED}OpenSSL is not installed. Please install it first.${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ OpenSSL found${NC}"
}

# Function to create directory structure
create_directories() {
    echo -e "${YELLOW}Creating certificate directories...${NC}"

    mkdir -p "$CA_DIR"
    mkdir -p "$CA_DIR/private"
    mkdir -p "$CA_DIR/certs"
    mkdir -p "$CA_DIR/crl"
    mkdir -p "$CA_DIR/newcerts"

    # Create directories for each service
    for service_config in "${SERVICES[@]}"; do
        IFS=':' read -r service port sans <<< "$service_config"
        mkdir -p "$CERT_DIR/$service"
    done

    # Create CA database files
    touch "$CA_DIR/index.txt"
    echo "1000" > "$CA_DIR/serial"
    echo "1000" > "$CA_DIR/crlnumber"

    echo -e "${GREEN}✓ Directories created${NC}"
}

# Function to create CA configuration
create_ca_config() {
    echo -e "${YELLOW}Creating CA configuration...${NC}"

    cat > "$CA_DIR/ca.conf" << EOF
[ ca ]
default_ca = CA_default

[ CA_default ]
dir               = $CA_DIR
certs             = \$dir/certs
crl_dir           = \$dir/crl
new_certs_dir     = \$dir/newcerts
database          = \$dir/index.txt
serial            = \$dir/serial
crlnumber         = \$dir/crlnumber
crl               = \$dir/crl.pem
certificate       = \$dir/ca.crt
private_key       = \$dir/private/ca.key
RANDFILE          = \$dir/private/.rand
x509_extensions   = usr_cert
name_opt          = ca_default
cert_opt          = ca_default
default_days      = $VALIDITY_DAYS
default_crl_days  = 30
default_md        = sha256
preserve          = no
policy            = policy_loose

[ policy_loose ]
countryName             = optional
stateOrProvinceName     = optional
localityName            = optional
organizationName        = optional
organizationalUnitName  = optional
commonName              = supplied
emailAddress            = optional

[ req ]
default_bits        = 4096
default_keyfile     = privkey.pem
distinguished_name  = req_distinguished_name
attributes          = req_attributes
x509_extensions     = v3_ca
string_mask         = utf8only
default_md          = sha256

[ req_distinguished_name ]
countryName                     = Country Name (2 letter code)
countryName_default             = $COUNTRY
stateOrProvinceName             = State or Province Name (full name)
stateOrProvinceName_default     = $STATE
localityName                    = Locality Name (eg, city)
localityName_default            = $LOCALITY
organizationName                = Organization Name (eg, company)
organizationName_default        = $ORGANIZATION
organizationalUnitName          = Organizational Unit Name (eg, section)
commonName                      = Common Name (e.g. server FQDN or YOUR name)
emailAddress                    = Email Address

[ req_attributes ]
challengePassword               = A challenge password
challengePassword_min           = 4
challengePassword_max           = 20
unstructuredName                = An optional company name

[ v3_ca ]
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid:always,issuer
basicConstraints = critical,CA:true
keyUsage = critical, digitalSignature, cRLSign, keyCertSign

[ v3_intermediate_ca ]
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid:always,issuer
basicConstraints = critical, CA:true, pathlen:0
keyUsage = critical, digitalSignature, cRLSign, keyCertSign

[ usr_cert ]
basicConstraints = CA:FALSE
nsCertType = client, email
nsComment = "OpenSSL Generated Client Certificate"
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer
keyUsage = critical, nonRepudiation, digitalSignature, keyEncipherment
extendedKeyUsage = clientAuth, emailProtection

[ server_cert ]
basicConstraints = CA:FALSE
nsCertType = server
nsComment = "OpenSSL Generated Server Certificate"
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer:always
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
EOF

    echo -e "${GREEN}✓ CA configuration created${NC}"
}

# Function to generate Root CA
generate_root_ca() {
    echo -e "${YELLOW}Generating Root CA...${NC}"

    if [ -f "$CA_DIR/ca.crt" ] && [ -f "$CA_DIR/private/ca.key" ]; then
        echo -e "${YELLOW}Root CA already exists, skipping...${NC}"
        return
    fi

    # Generate Root CA private key
    openssl genrsa -out "$CA_DIR/private/ca.key" 4096
    chmod 400 "$CA_DIR/private/ca.key"

    # Generate Root CA certificate
    openssl req -config "$CA_DIR/ca.conf" \
        -key "$CA_DIR/private/ca.key" \
        -new -x509 -days 7300 -sha256 -extensions v3_ca \
        -out "$CA_DIR/ca.crt" \
        -subj "/C=$COUNTRY/ST=$STATE/L=$LOCALITY/O=$ORGANIZATION/CN=BotServer Root CA"

    # Copy CA cert to main cert directory for easy access
    cp "$CA_DIR/ca.crt" "$CERT_DIR/ca.crt"

    echo -e "${GREEN}✓ Root CA generated${NC}"
}

# Function to generate Intermediate CA
generate_intermediate_ca() {
    echo -e "${YELLOW}Generating Intermediate CA...${NC}"

    if [ -f "$CA_DIR/intermediate.crt" ] && [ -f "$CA_DIR/private/intermediate.key" ]; then
        echo -e "${YELLOW}Intermediate CA already exists, skipping...${NC}"
        return
    fi

    # Generate Intermediate CA private key
    openssl genrsa -out "$CA_DIR/private/intermediate.key" 4096
    chmod 400 "$CA_DIR/private/intermediate.key"

    # Generate Intermediate CA CSR
    openssl req -config "$CA_DIR/ca.conf" \
        -new -sha256 \
        -key "$CA_DIR/private/intermediate.key" \
        -out "$CA_DIR/intermediate.csr" \
        -subj "/C=$COUNTRY/ST=$STATE/L=$LOCALITY/O=$ORGANIZATION/CN=BotServer Intermediate CA"

    # Sign Intermediate CA certificate with Root CA
    openssl ca -config "$CA_DIR/ca.conf" \
        -extensions v3_intermediate_ca \
        -days 3650 -notext -md sha256 \
        -in "$CA_DIR/intermediate.csr" \
        -out "$CA_DIR/intermediate.crt" \
        -batch

    chmod 444 "$CA_DIR/intermediate.crt"

    # Create certificate chain
    cat "$CA_DIR/intermediate.crt" "$CA_DIR/ca.crt" > "$CA_DIR/ca-chain.crt"

    echo -e "${GREEN}✓ Intermediate CA generated${NC}"
}

# Function to generate service certificate
generate_service_cert() {
    local service=$1
    local port=$2
    local sans=$3

    echo -e "${YELLOW}Generating certificates for $service...${NC}"

    local cert_dir="$CERT_DIR/$service"

    # Create SAN configuration
    cat > "$cert_dir/san.conf" << EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req

[req_distinguished_name]

[v3_req]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
subjectAltName = @alt_names

[alt_names]
EOF

    # Add SANs
    IFS=',' read -ra SAN_ARRAY <<< "$sans"
    local san_index=1
    for san in "${SAN_ARRAY[@]}"; do
        if [[ $san =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            echo "IP.$san_index = $san" >> "$cert_dir/san.conf"
        else
            echo "DNS.$san_index = $san" >> "$cert_dir/san.conf"
        fi
        ((san_index++))
    done

    # Generate server private key
    openssl genrsa -out "$cert_dir/server.key" 2048
    chmod 400 "$cert_dir/server.key"

    # Generate server CSR
    openssl req -new -sha256 \
        -key "$cert_dir/server.key" \
        -out "$cert_dir/server.csr" \
        -config "$cert_dir/san.conf" \
        -subj "/C=$COUNTRY/ST=$STATE/L=$LOCALITY/O=$ORGANIZATION/CN=$service.$COMMON_NAME_SUFFIX"

    # Sign server certificate with CA
    openssl x509 -req -in "$cert_dir/server.csr" \
        -CA "$CA_DIR/ca.crt" \
        -CAkey "$CA_DIR/private/ca.key" \
        -CAcreateserial \
        -out "$cert_dir/server.crt" \
        -days $VALIDITY_DAYS \
        -sha256 \
        -extensions v3_req \
        -extfile "$cert_dir/san.conf"

    # Generate client certificate for mTLS
    openssl genrsa -out "$cert_dir/client.key" 2048
    chmod 400 "$cert_dir/client.key"

    openssl req -new -sha256 \
        -key "$cert_dir/client.key" \
        -out "$cert_dir/client.csr" \
        -subj "/C=$COUNTRY/ST=$STATE/L=$LOCALITY/O=$ORGANIZATION/CN=$service-client.$COMMON_NAME_SUFFIX"

    openssl x509 -req -in "$cert_dir/client.csr" \
        -CA "$CA_DIR/ca.crt" \
        -CAkey "$CA_DIR/private/ca.key" \
        -CAcreateserial \
        -out "$cert_dir/client.crt" \
        -days $VALIDITY_DAYS \
        -sha256

    # Copy CA certificate to service directory
    cp "$CA_DIR/ca.crt" "$cert_dir/ca.crt"

    # Create full chain certificate
    cat "$cert_dir/server.crt" "$CA_DIR/ca.crt" > "$cert_dir/fullchain.crt"

    # Clean up CSR files
    rm -f "$cert_dir/server.csr" "$cert_dir/client.csr" "$cert_dir/san.conf"

    echo -e "${GREEN}✓ Certificates generated for $service (port $port)${NC}"
}

# Function to generate all service certificates
generate_all_service_certs() {
    echo -e "${BLUE}Generating certificates for all services...${NC}"

    for service_config in "${SERVICES[@]}"; do
        IFS=':' read -r service port sans <<< "$service_config"
        generate_service_cert "$service" "$port" "$sans"
    done

    echo -e "${GREEN}✓ All service certificates generated${NC}"
}

# Function to create TLS configuration file
create_tls_config() {
    echo -e "${YELLOW}Creating TLS configuration file...${NC}"

    cat > "$CERT_DIR/tls-config.toml" << EOF
# TLS Configuration for BotServer
# Generated on $(date)

[tls]
enabled = true
mtls_enabled = true
auto_generate_certs = true
renewal_threshold_days = 30

[ca]
ca_cert_path = "$CA_DIR/ca.crt"
ca_key_path = "$CA_DIR/private/ca.key"
intermediate_cert_path = "$CA_DIR/intermediate.crt"
intermediate_key_path = "$CA_DIR/private/intermediate.key"
validity_days = $VALIDITY_DAYS
organization = "$ORGANIZATION"
country = "$COUNTRY"
state = "$STATE"
locality = "$LOCALITY"

# Service configurations
EOF

    for service_config in "${SERVICES[@]}"; do
        IFS=':' read -r service port sans <<< "$service_config"
        cat >> "$CERT_DIR/tls-config.toml" << EOF

[[services]]
name = "$service"
port = $port
cert_path = "$CERT_DIR/$service/server.crt"
key_path = "$CERT_DIR/$service/server.key"
client_cert_path = "$CERT_DIR/$service/client.crt"
client_key_path = "$CERT_DIR/$service/client.key"
ca_cert_path = "$CERT_DIR/$service/ca.crt"
sans = "$sans"
EOF
    done

    echo -e "${GREEN}✓ TLS configuration file created${NC}"
}

# Function to display certificate information
display_cert_info() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Certificate Information${NC}"
    echo -e "${BLUE}========================================${NC}"

    echo -e "${YELLOW}Root CA:${NC}"
    openssl x509 -in "$CA_DIR/ca.crt" -noout -subject -dates

    echo ""
    echo -e "${YELLOW}Service Certificates:${NC}"
    for service_config in "${SERVICES[@]}"; do
        IFS=':' read -r service port sans <<< "$service_config"
        echo -e "${GREEN}$service (port $port):${NC}"
        openssl x509 -in "$CERT_DIR/$service/server.crt" -noout -subject -dates
    done
}

# Function to create environment variables file
create_env_vars() {
    echo -e "${YELLOW}Creating environment variables file...${NC}"

    cat > "$CERT_DIR/tls.env" << EOF
# TLS Environment Variables for BotServer
# Source this file to set TLS environment variables

export TLS_ENABLED=true
export MTLS_ENABLED=true
export CA_CERT_PATH="$CA_DIR/ca.crt"
export CA_KEY_PATH="$CA_DIR/private/ca.key"

# Service-specific TLS settings
export API_TLS_PORT=8443
export API_CERT_PATH="$CERT_DIR/api/server.crt"
export API_KEY_PATH="$CERT_DIR/api/server.key"

export LLM_TLS_PORT=8444
export LLM_CERT_PATH="$CERT_DIR/llm/server.crt"
export LLM_KEY_PATH="$CERT_DIR/llm/server.key"

export EMBEDDING_TLS_PORT=8445
export EMBEDDING_CERT_PATH="$CERT_DIR/embedding/server.crt"
export EMBEDDING_KEY_PATH="$CERT_DIR/embedding/server.key"

export QDRANT_TLS_PORT=6334
export QDRANT_CERT_PATH="$CERT_DIR/qdrant/server.crt"
export QDRANT_KEY_PATH="$CERT_DIR/qdrant/server.key"

export REDIS_TLS_PORT=6380
export REDIS_CERT_PATH="$CERT_DIR/redis/server.crt"
export REDIS_KEY_PATH="$CERT_DIR/redis/server.key"

export POSTGRES_TLS_PORT=5433
export POSTGRES_CERT_PATH="$CERT_DIR/postgres/server.crt"
export POSTGRES_KEY_PATH="$CERT_DIR/postgres/server.key"

export MINIO_TLS_PORT=9001
export MINIO_CERT_PATH="$CERT_DIR/minio/server.crt"
export MINIO_KEY_PATH="$CERT_DIR/minio/server.key"
EOF

    echo -e "${GREEN}✓ Environment variables file created${NC}"
}

# Function to test certificate validity
test_certificates() {
    echo -e "${BLUE}Testing certificate validity...${NC}"

    local all_valid=true

    for service_config in "${SERVICES[@]}"; do
        IFS=':' read -r service port sans <<< "$service_config"

        # Verify certificate chain
        if openssl verify -CAfile "$CA_DIR/ca.crt" "$CERT_DIR/$service/server.crt" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ $service server certificate is valid${NC}"
        else
            echo -e "${RED}✗ $service server certificate is invalid${NC}"
            all_valid=false
        fi

        if openssl verify -CAfile "$CA_DIR/ca.crt" "$CERT_DIR/$service/client.crt" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ $service client certificate is valid${NC}"
        else
            echo -e "${RED}✗ $service client certificate is invalid${NC}"
            all_valid=false
        fi
    done

    if $all_valid; then
        echo -e "${GREEN}✓ All certificates are valid${NC}"
    else
        echo -e "${RED}Some certificates failed validation${NC}"
        exit 1
    fi
}

# Main execution
main() {
    check_openssl
    create_directories
    create_ca_config
    generate_root_ca
    # Intermediate CA is optional but recommended
    # generate_intermediate_ca
    generate_all_service_certs
    create_tls_config
    create_env_vars
    test_certificates
    display_cert_info

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}TLS Setup Complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Source the environment variables: source $CERT_DIR/tls.env"
    echo "2. Update your service configurations to use HTTPS/TLS"
    echo "3. Restart all services with TLS enabled"
    echo ""
    echo -e "${YELLOW}Important files:${NC}"
    echo "  CA Certificate: $CA_DIR/ca.crt"
    echo "  TLS Config: $CERT_DIR/tls-config.toml"
    echo "  Environment Variables: $CERT_DIR/tls.env"
    echo ""
    echo -e "${YELLOW}To trust the CA certificate on your system:${NC}"
    echo "  Ubuntu/Debian: sudo cp $CA_DIR/ca.crt /usr/local/share/ca-certificates/botserver-ca.crt && sudo update-ca-certificates"
    echo "  RHEL/CentOS: sudo cp $CA_DIR/ca.crt /etc/pki/ca-trust/source/anchors/ && sudo update-ca-trust"
    echo "  macOS: sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain $CA_DIR/ca.crt"
}

# Run main function
main
