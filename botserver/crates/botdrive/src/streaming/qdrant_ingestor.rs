//! Ingestor de streaming para Qdrant — processa arquivos grandes em chunks
//! sem carregar o conteúdo inteiro em memória. Peak memory: chunk (16MB) + batch embeddings.
//!
//! Uso:
//! ```rust,ignore
//! let mut ingestor = QdrantStreamIngestor::new(client, collection, embedding_dim)?;
//! ingestor.ingest_text_file("/caminho/arquivo.txt")?;
//! ```

use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::Arc;

use botqdrant::{PointStruct, Qdrant, UpsertPointsBuilder};

use crate::stream_processor::StreamProcessor;

const CHUNK_SIZE: usize = 16 * 1024 * 1024; // 16MB por chunk
const QDRANT_BATCH_SIZE: usize = 50; // vectors por upsert batch

pub struct QdrantStreamIngestor {
    client: Arc<Qdrant>,
    collection_name: String,
    embedding_dim: u64,
    batch: Vec<PointStruct>,
    total_points: u64,
}

impl QdrantStreamIngestor {
    pub fn new(
        client: Arc<Qdrant>,
        collection_name: String,
        embedding_dim: u64,
    ) -> Self {
        Self {
            client,
            collection_name,
            embedding_dim,
            batch: Vec::with_capacity(QDRANT_BATCH_SIZE),
            total_points: 0,
        }
    }

    /// Processa um arquivo texto em streaming: lê em chunks de 16MB, gera embeddings
    /// por parágrafo e faz upsert em lotes de 50 vectores no Qdrant.
    pub fn ingest_text_file(&mut self, path: &Path) -> Result<u64, String> {
        let file = std::fs::File::open(path)
            .map_err(|e| format!("Falha ao abrir {}: {}", path.display(), e))?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut paragraph = String::new();
        let mut partial_line = String::new();

        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .map_err(|e| format!("Erro de leitura: {}", e))?;
            if bytes_read == 0 {
                break;
            }

            let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
            let mut text = std::mem::take(&mut partial_line);
            text.push_str(&chunk);

            let lines: Vec<&str> = text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() && !paragraph.is_empty() {
                    self.flush_paragraph(&mut paragraph)?;
                } else if !trimmed.is_empty() {
                    if !paragraph.is_empty() {
                        paragraph.push(' ');
                    }
                    paragraph.push_str(trimmed);
                }
                // Se for a última linha e não terminar com newline, guarda para o próximo chunk
                if i == lines.len() - 1 && !chunk.ends_with('\n') {
                    partial_line = line.to_string();
                }
            }
        }

        // Flush parágrafo residual
        if !paragraph.is_empty() {
            self.flush_paragraph(&mut paragraph)?;
        }

        // Flush batch residual
        self.flush_batch()?;

        Ok(self.total_points)
    }

    /// Ingests pre-extracted text (from a format processor) by
    /// splitting into paragraphs and upserting to Qdrant.
    pub fn ingest_text(&mut self, text: &str) -> Result<u64, String> {
        let mut paragraph = String::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() && !paragraph.is_empty() {
                self.flush_paragraph(&mut paragraph)?;
            } else if !trimmed.is_empty() {
                if !paragraph.is_empty() {
                    paragraph.push(' ');
                }
                paragraph.push_str(trimmed);
            }
        }
        if !paragraph.is_empty() {
            self.flush_paragraph(&mut paragraph)?;
        }
        self.flush_batch()?;
        Ok(self.total_points)
    }

    /// Processes a file through a format-specific stream processor
    /// and ingests the extracted text into Qdrant.
    pub fn ingest_with_processor(
        &mut self,
        path: &Path,
        processor: &mut dyn StreamProcessor,
    ) -> Result<u64, String> {
        let text = processor.process_stream(path)?;
        self.ingest_text(&text)
    }

    fn flush_paragraph(&mut self, paragraph: &mut String) -> Result<(), String> {
        let text = std::mem::take(paragraph);
        // Gera embedding placeholder — em produção substituir por chamada real ao serviço de embedding
        let embedding = self.generate_embedding(&text)?;
        let point = self.build_point(&text, embedding);
        self.batch.push(point);
        self.total_points += 1;

        if self.batch.len() >= QDRANT_BATCH_SIZE {
            self.flush_batch()?;
        }
        Ok(())
    }

    fn build_point(&self, text: &str, embedding: Vec<f32>) -> PointStruct {
        let id = uuid::Uuid::new_v4().to_string();
        let payload: botqdrant::Payload = serde_json::json!({
            "text": text,
            "chunk_size": text.len(),
            "ingested_at": chrono::Utc::now().to_rfc3339(),
        })
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<serde_json::Map<String, serde_json::Value>>();

        PointStruct::new(id, embedding, payload)
    }

    fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>, String> {
        // TODO: Substituir por chamada real ao serviço de embedding
        // (ex.: OpenAI Embeddings API ou serviço local)
        // Retorna vector dummy do tamanho correto para validação do pipeline
        Ok(vec![0.0f32; self.embedding_dim as usize])
    }

    fn flush_batch(&mut self) -> Result<(), String> {
        if self.batch.is_empty() {
            return Ok(());
        }

        let points = std::mem::replace(
            &mut self.batch,
            Vec::with_capacity(QDRANT_BATCH_SIZE),
        );

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Falha ao criar runtime: {}", e))?;

        rt.block_on(async {
            UpsertPointsBuilder::new(&self.collection_name, points)
                .build(&self.client)
                .await
                .map_err(|e| format!("Falha no upsert Qdrant: {}", e))
        })?;

        log::info!(
            "Ingestor: flush batch de {} pontos para coleção {}",
            QDRANT_BATCH_SIZE.min(self.batch.capacity()),
            self.collection_name
        );

        Ok(())
    }

    pub fn total_points(&self) -> u64 {
        self.total_points
    }
}
