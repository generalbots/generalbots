storage "file" {
  path = "/home/a404010/src/generalbots/botserver/botserver-stack/data/vault/vault"
}

listener "tcp" {
  address       = "0.0.0.0:8200"
  tls_disable   = false
  tls_cert_file = "/home/a404010/src/generalbots/botserver/botserver-stack/conf/system/certificates/vault/server.crt"
  tls_key_file  = "/home/a404010/src/generalbots/botserver/botserver-stack/conf/system/certificates/vault/server.key"
  tls_client_ca_file = "/home/a404010/src/generalbots/botserver/botserver-stack/conf/system/certificates/ca/ca.crt"
}

api_addr = "https://localhost:8200"
cluster_addr = "https://localhost:8201"
ui = true
disable_mlock = true
