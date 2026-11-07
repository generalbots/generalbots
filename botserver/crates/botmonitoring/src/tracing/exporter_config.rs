use super::*;

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            // Outbound tracing is opt-in: nothing leaves the machine until an
            // operator explicitly selects an exporter and endpoint.
            exporter_type: ExporterType::None,
            endpoint: String::new(),
            headers: HashMap::new(),
            batch_size: 512,
            flush_interval_ms: 5000,
            timeout_ms: 10000,
            compression: true,
            enabled: false,
        }
    }
}
