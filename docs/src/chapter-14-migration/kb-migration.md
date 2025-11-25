# Knowledge Base Migration

Converting documents from cloud storage to General Bots knowledge bases.

## Overview

Knowledge base migration involves moving documents from various sources into `.gbkb` packages where they become searchable through General Bots.

## Source Systems

Common document sources:
- SharePoint document libraries
- Google Drive folders
- OneDrive/Dropbox
- Confluence spaces
- File servers

## Document Types

Supported formats:
- PDF files
- Office documents (Word, Excel, PowerPoint)
- Text files
- Markdown files
- HTML pages

## Migration Process

### 1. Export
- Download documents from source
- Preserve folder structure
- Maintain metadata where possible

### 2. Organize
- Group related documents
- Create logical collections
- Remove duplicates

### 3. Import
- Place in `.gbkb` folders
- General Bots indexes automatically
- Documents become searchable

## Considerations

### Volume
- Large document sets take time to index
- Consider staged migration
- Monitor disk space

### Quality
- Clean up outdated content first
- Remove duplicate documents
- Fix broken files

### Structure
- Maintain logical organization
- Use meaningful folder names
- Group by topic or department

## Format Conversion

Some formats may need conversion:
- Web pages → PDF or Markdown
- Databases → CSV exports
- Proprietary formats → Standard formats

## Testing

After migration:
- Verify search works
- Check document access
- Test with sample queries

## Next Steps

- [Overview](./overview.md) - Migration concepts
- [Validation](./validation.md) - Testing procedures