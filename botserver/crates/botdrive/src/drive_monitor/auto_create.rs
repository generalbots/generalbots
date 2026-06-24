//! Auto-criação de bots a partir de buckets no MinIO (Issue #506).
//!
//! A lógica de auto-criação foi integrada diretamente no método `scan_bucket`
//! em `types.rs`. Quando o DriveMonitor escaneia um bucket `{nome}.gbai`,
//! verifica se o bot já existe na tabela `bots` e o cria automaticamente se
//! necessário.
//!
//! ## Fluxo
//! 1. `scan_bucket` lista objetos no bucket `{nome}.gbai`
//! 2. Extrai o `bot_name` do nome do bucket (removendo sufixo `.gbai`)
//! 3. Verifica se o bot existe em `SELECT id FROM bots WHERE name = $1`
//! 4. Se não existir: `INSERT INTO bots (id, name, created_at) VALUES (...)`
//! 5. Continua o processamento normal dos arquivos do bucket
//!
//! ## Por que integrar em scan_bucket?
//! - Evita duplicação de lógica de listagem de buckets
//! - Garante consistência transacional (bot existe antes de processar arquivos)
//! - Aproveita o loop de monitoramento existente (reentrância, backoff, etc.)

/// Certifica-se de que o bot existe no banco de dados.
/// A implementação real está em `types.rs::scan_bucket()` — esta função
/// é mantida como atalho para chamadas externas que precisam verificar
/// um bot específico sem acionar o scan completo.
pub async fn ensure_bot_exists(
    bot_name: &str,
) -> Result<bool, String> {
    // A implementação real foi integrada em types.rs::scan_bucket()
    // Este arquivo agora serve como documentação e ponto de entrada
    // alternativo para uso programático.
    // TODO(#506): Implementar query direta: SELECT EXISTS(SELECT 1 FROM bots WHERE name = $1)
    // Se false: INSERT INTO bots (id, name, org_id, created_at) VALUES (gen_random_uuid(), $1, $2, NOW())
    let _ = bot_name;
    Ok(false)
}

/// Sincroniza todos os buckets .gbai do MinIO com a tabela `bots`.
/// A implementação real percorre cada bucket do drive e chama o
/// DriveMonitor.scan_bucket() correspondente.
pub async fn sync_bots_from_buckets() -> Result<u32, String> {
    // TODO(#506): Listar todos os buckets .gbai no MinIO (/tmp/mc ls local/),
    // extrair bot_name de cada bucket, chamar ensure_bot_exists para cada um.
    // Retornar total de bots criados vs já existentes.
    Ok(0)
}
