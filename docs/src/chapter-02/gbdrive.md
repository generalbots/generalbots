# .gbdrive File Storage

The .gbdrive system provides centralized file storage for all bot packages, leveraging S3-compatible object storage to deliver reliable, scalable storage infrastructure. This chapter explains how file storage works, how files are organized, and how to interact with stored content.

## Understanding File Storage in General Bots

Every bot requires storage for its various components—scripts, documents, configuration files, user uploads, and generated content. Rather than managing files across disparate locations, General Bots consolidates storage through the .gbdrive system, which provides a consistent interface regardless of the underlying storage backend.

The storage system builds on S3-compatible object storage, meaning it works with self-hosted solutions like MinIO as well as cloud providers like AWS S3, Backblaze B2, or DigitalOcean Spaces. This flexibility allows deployments to choose storage solutions that match their requirements for cost, performance, and data residency.

Beyond simple file storage, the system provides versioning capabilities, access control, automatic synchronization, and integration with other bot components like knowledge bases and themes.

## Storage Organization

Files are organized using a bucket-per-bot structure that keeps each bot's content isolated and manageable. Within a bot's storage bucket, the familiar package structure appears: .gbdialog for scripts, .gbkb for knowledge base collections, .gbot for configuration, and .gbtheme for interface customization.

Additionally, each bot has space for user-uploaded files, generated content, and other runtime data. This organization mirrors the logical structure you work with during development, making it intuitive to understand where files reside and how they relate to bot functionality.

The system maintains this structure automatically when bots are deployed or updated, ensuring that the storage state reflects the current bot configuration without manual intervention.

## Working with Files

File operations in General Bots happen through several interfaces depending on your needs. The BASIC scripting language provides keywords for reading file content directly into scripts, enabling bots to process documents, load data, or access configuration dynamically.

Files can also be managed through the administrative API for bulk operations, migrations, or integration with external systems. The web interface provides user-facing upload and download capabilities where appropriate.

When files change in storage, the system detects modifications and triggers appropriate responses. Script changes cause recompilation, document changes trigger knowledge base reindexing, and configuration changes reload bot settings. This hot-reloading capability accelerates development and enables runtime updates without service interruption.

## Integration with Bot Components

The storage system integrates deeply with other bot components, serving as the foundation for several capabilities.

Knowledge bases draw their source documents from storage, with the indexing system monitoring for changes and updating embeddings accordingly. When you add a document to a .gbkb folder, it automatically becomes part of the bot's searchable knowledge.

Theme assets including CSS files and images are served from storage, with appropriate caching to ensure good performance. Changes to theme files take effect quickly without requiring restarts.

Tool scripts in .gbdialog folders are loaded from storage, parsed, and made available for execution. The compilation system tracks dependencies and rebuilds as needed when source files change.

## Access Control

Different files require different access levels, and the storage system enforces appropriate controls. Public files can be accessed without authentication, suitable for shared resources or publicly downloadable content. Authenticated access requires valid user credentials, protecting user-specific uploads and downloads.

Bot-internal files remain accessible only to the bot system itself, protecting scripts and configuration from unauthorized access. Administrative files containing sensitive configuration require elevated privileges to access or modify.

These access levels work in conjunction with the broader authentication and authorization system, ensuring that file access respects organizational security policies.

## Storage Backend Options

The storage system supports multiple backends to accommodate different deployment scenarios. The default configuration uses self-hosted S3-compatible object storage, providing full control over where data resides. Any S3-compatible service works as an alternative, including major cloud providers.

For development and testing, local filesystem storage offers simplicity and easy inspection of files. Production deployments might use hybrid configurations with multiple backends providing redundancy or geographic distribution.

Backend selection happens through configuration, and the rest of the system interacts with storage through a consistent interface regardless of which backend is active. This abstraction allows deployments to change storage strategies without modifying bot code.

## Summary

The .gbdrive storage system provides the foundation for all file-based operations in General Bots. Through S3-compatible object storage, organized bucket structures, automatic synchronization, and deep integration with other components, it delivers reliable file management that supports both development workflows and production operation. Understanding how storage works helps you organize bot content effectively and leverage the automatic capabilities the system provides.