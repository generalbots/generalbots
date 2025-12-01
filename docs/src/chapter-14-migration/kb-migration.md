# Knowledge Base Migration

Converting documents from cloud storage to General Bots knowledge bases.

## Overview

Knowledge base migration involves moving documents from various sources into `.gbkb` packages where they become searchable through General Bots.

## Source Systems

Common document sources include SharePoint document libraries, Google Drive folders, OneDrive and Dropbox storage, Confluence spaces, and traditional file servers.

## Document Types

General Bots supports a variety of document formats for knowledge base ingestion. These include PDF files, Office documents such as Word, Excel, and PowerPoint, plain text files, Markdown files, and HTML pages.

## Migration Process

### 1. Export

Begin by downloading documents from the source system. Preserve the folder structure to maintain organizational context, and retain metadata where possible for future reference.

### 2. Organize

Group related documents into logical collections. Create meaningful organizational structures and remove any duplicate documents that would clutter the knowledge base.

### 3. Import

Place the organized documents in `.gbkb` folders within your bot package. General Bots indexes these documents automatically, making them searchable for RAG-powered responses.

## Considerations

### Volume

Large document sets require additional time to index. Consider staging the migration in batches rather than importing everything at once. Monitor disk space throughout the process to ensure adequate storage remains available.

### Quality

Before migration, clean up outdated content that no longer reflects current information. Remove duplicate documents to avoid confusing the AI with conflicting information. Fix any broken or corrupted files that would fail during indexing.

### Structure

Maintain logical organization within your knowledge base. Use meaningful folder names that describe the content within. Group documents by topic or department to improve retrieval accuracy.

## Format Conversion

Some formats require conversion before import. Web pages should be converted to PDF or Markdown for reliable indexing. Database content should be exported to CSV format. Proprietary formats from specialized applications need conversion to standard formats that the indexing system can process.

## Testing

After migration, verify the knowledge base functions correctly. Test that search works across the imported documents. Check that users can access all migrated content. Run sample queries to ensure the AI provides accurate responses based on the imported knowledge.

## Next Steps

Review the [Overview](./overview.md) for general migration concepts. See [Validation](./validation.md) for detailed testing procedures to verify your migration succeeded.