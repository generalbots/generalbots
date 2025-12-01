# Storage and Data

This chapter explains how BotServer organizes and manages data across its multiple storage layers. Understanding this architecture helps you make informed decisions about where to store different types of information and how to optimize storage performance for your deployment.

## Understanding the Storage Architecture

BotServer employs a multi-layered storage architecture where each layer serves specific purposes and data types. Rather than forcing all data into a single storage system, this design allows each component to use the storage technology best suited to its access patterns and requirements.

PostgreSQL serves as the primary database for all structured data, including user accounts, session information, bot configurations, and message history. Its relational model excels at maintaining data integrity and supporting complex queries across related entities.

The Drive component provides S3-compatible object storage for files and documents. This includes uploaded files, knowledge base documents, BASIC scripts, and media assets. Object storage handles large files efficiently and integrates well with content delivery networks.

Valkey (the cache layer) maintains session state and temporary data that benefits from extremely fast access. Cached data might be lost during restarts, but the performance benefits for frequently accessed information justify this trade-off.

Qdrant stores vector embeddings that power semantic search. These high-dimensional numerical representations capture the meaning of documents and queries, enabling similarity-based retrieval that goes beyond keyword matching.

Local filesystem storage handles temporary working directories, log files, and operational caches that don't require persistence across system restarts.

## PostgreSQL: Structured Data Storage

PostgreSQL anchors the storage architecture by maintaining all structured information that requires durability and relational integrity. User accounts, their associations with sessions, and the relationships between users and bots all live in PostgreSQL tables protected by transactions and foreign key constraints.

The database schema evolves through managed migrations stored in the migrations directory. Diesel ORM provides type-safe database access from Rust code, catching many potential errors at compile time rather than runtime. When the system bootstraps, it automatically applies pending migrations to bring the schema up to date.

Connection pooling ensures efficient database access under load. The pool maintains a configurable number of connections ready for use, eliminating the overhead of establishing new connections for each query. Automatic retry logic handles transient connection failures, and timeout protection prevents runaway queries from consuming resources indefinitely.

Message history accumulates in the database, creating a permanent record of all conversations. Session data persists across server restarts, allowing users to resume conversations even after maintenance windows. Bot configurations stored in the database take effect immediately across all running instances.

## Drive: Object Storage for Files

The Drive component implements S3-compatible object storage, organizing files into buckets that typically correspond to individual bots. Within each bucket, the familiar package structure appears: .gbdialog folders for scripts, .gbkb folders for knowledge base documents, and .gbot folders for configuration.

File operations follow standard patterns. Uploads place files into specified bucket and key combinations. Downloads retrieve files by their bucket and key. Listing operations enumerate bucket contents for browsing or processing. Deletion removes objects when necessary, though this operation is relatively rare in normal operation.

The storage system supports any S3-compatible backend, including self-hosted solutions like MinIO for development and cloud services like AWS S3 for production. This flexibility allows deployments to choose storage solutions that match their requirements for cost, performance, geographic distribution, and data residency.

Beyond static storage, Drive integrates with the knowledge base system. Documents uploaded to .gbkb folders trigger indexing pipelines that extract text, generate embeddings, and make content searchable. Changes to stored files can trigger reprocessing, keeping knowledge bases current as source documents evolve.

## Valkey: High-Speed Caching

The cache layer accelerates access to frequently used data by keeping it in memory. Session tokens validate quickly against cached values. Recently accessed conversation state retrieves without database queries. Rate limiting counters update with minimal latency.

Cached data follows patterns that maximize effectiveness. Session data uses keys combining the session identifier with the data type, enabling targeted retrieval. Rate limiting keys incorporate user identifiers and endpoint paths to track request rates per user per endpoint. Temporary data keys clearly indicate their transient nature.

Cache entries include time-to-live values that automatically expire stale data. Session caches might persist for 24 hours of inactivity. Rate limiting counters reset after their tracking windows. Temporary computation results expire after configurable periods.

The cache operates as a performance optimization rather than a primary data store. If cached data is lost, the system regenerates it from authoritative sources in PostgreSQL or Drive. This approach simplifies operations since cache failures cause performance degradation rather than data loss.

## Qdrant: Vector Storage for Semantic Search

Qdrant provides the specialized storage that makes semantic search possible. Each document chunk from knowledge bases generates a high-dimensional vector embedding that captures its semantic content. These vectors live in Qdrant collections organized by bot, enabling fast similarity searches.

The vector storage structure includes collections for different content types. Document embeddings enable knowledge base search. Conversation embeddings support finding similar past interactions. Cached query results accelerate repeated searches.

Vector operations differ from traditional database operations. Insertion adds new embeddings along with their associated metadata. Search finds vectors most similar to a query vector, returning the closest matches based on distance metrics. Updates modify the metadata associated with existing vectors. Deletion removes outdated content when source documents change.

Qdrant's specialized architecture handles the mathematical operations behind similarity search efficiently. Unlike general-purpose databases that would struggle with high-dimensional vector comparisons, Qdrant implements optimized algorithms that scale to millions of vectors while maintaining sub-second query response times.

## Local Storage: Working Directories

The local filesystem serves operational needs that don't require distributed storage. Working directories hold files during processing, such as documents being indexed or uploads being validated. Log files capture operational events for debugging and monitoring. Local caches store computed values that benefit from filesystem-level caching.

Directory structure follows conventions that keep different content types organized. The work directory contains per-bot working files during processing. Logs accumulate in dedicated directories with rotation policies that prevent unbounded growth. Upload directories receive incoming files temporarily before they move to permanent storage.

Automatic cleanup processes remove files that no longer serve purposes. Old temporary files delete after their processing completes. Log rotation archives and eventually removes old log files. Cache invalidation clears stale computed values when source data changes.

## Data Persistence and Backup

Reliable data storage requires comprehensive backup strategies that protect against various failure modes. BotServer's multi-layer architecture requires coordinating backups across storage systems.

PostgreSQL backups capture the authoritative state of all structured data. Daily dumps create recovery points. Point-in-time recovery capabilities protect against accidental data modifications. Backup verification ensures that recovery would actually work when needed.

Drive storage benefits from built-in replication and versioning capabilities. S3-compatible storage systems maintain multiple copies across availability zones. Object versioning preserves previous states even after modifications. Cross-region replication protects against regional failures for critical deployments.

Configuration versioning through source control provides another protection layer. Environment-specific configurations store separately from shared defaults. Secret encryption protects sensitive values in backups.

Retention policies balance storage costs against recovery needs. Message history might retain for 90 days before archival. Session data expires after 30 days of inactivity. Temporary files clean up within 24 hours. Log retention follows regulatory requirements and debugging needs. Backup retention provides sufficient history for recovery scenarios.

## Storage Operations in BASIC Scripts

Scripts interact with storage through dedicated keywords that abstract the underlying complexity. The SAVE keyword writes data to CSV files or other formats, handling the details of file creation and formatting. The GET keyword retrieves content from storage, automatically determining the appropriate storage layer based on the path specified.

These abstractions allow script authors to work with storage without understanding the full architecture. A script saving customer data doesn't need to know whether that data ultimately resides in PostgreSQL or Drive. The system routes operations appropriately based on data types and configurations.

## Security and Access Control

Data security spans all storage layers with appropriate protections for each. Encryption at rest protects stored data from unauthorized physical access. Database encryption covers PostgreSQL storage. Object storage encryption protects Drive contents. Transport encryption using TLS secures all network communication between components.

Access control ensures users and processes only reach data they're authorized to access. Role-based permissions govern database operations. Bucket policies control object storage access. Bot isolation prevents cross-bot data leakage. Audit logging creates accountability trails for sensitive operations.

Sensitive data receives additional protection. Passwords never store in BotServer systems since Zitadel handles authentication. API keys and secrets encrypt with AES-GCM before storage. Personally identifiable information follows data protection regulations applicable to the deployment jurisdiction.

## Monitoring and Maintenance

Storage systems require ongoing attention to maintain performance and reliability. Monitoring tracks resource utilization across all storage layers. Database size growth reveals capacity planning needs. Drive bucket usage indicates document accumulation rates. Cache memory utilization guides sizing decisions. Qdrant index size affects search performance.

Health checks verify that storage systems remain accessible and responsive. Database connectivity tests confirm query capability. Drive availability checks verify object operations work. Cache response time measurements identify performance degradation. Qdrant query tests validate search functionality.

Regular maintenance keeps storage systems performing well. PostgreSQL vacuum operations reclaim space and update statistics. Drive cleanup removes orphaned objects. Cache pruning maintains working set size. Qdrant optimization improves query performance as indexes grow.

## Troubleshooting Common Issues

Storage problems manifest in recognizable patterns that guide resolution. Space exhaustion causes write failures across storage layers. Resolution involves cleaning temporary files, archiving old data, or expanding storage allocation.

Performance degradation often traces to storage layer issues. Slow queries might indicate missing indexes or excessive table sizes. Slow file access might reveal network or disk bottlenecks. Cache misses might suggest insufficient cache sizing or inappropriate eviction policies.

Connection failures require systematic investigation. Service status checks confirm components are running. Credential verification ensures authentication succeeds. Network configuration review identifies routing or firewall issues.

## Summary

BotServer's storage architecture distributes data across specialized systems optimized for different access patterns. PostgreSQL handles structured data with transactional integrity. Drive provides scalable object storage for files and documents. Valkey accelerates access to frequently used information. Qdrant enables semantic search through vector storage. Understanding this architecture helps you configure storage appropriately, implement effective backup strategies, and troubleshoot issues when they arise. The result is a storage foundation that supports the diverse requirements of conversational AI applications while maintaining performance and reliability.