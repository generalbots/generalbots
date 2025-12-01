# Vector Collections

This chapter explains how BotServer organizes knowledge into vector collections, the searchable units that power semantic retrieval. Understanding how collections work helps you structure documents effectively and optimize the knowledge your bots can access.

<img src="../assets/chapter-03/storage-breakdown.svg" alt="Storage Breakdown" style="max-height: 400px; width: 100%; object-fit: contain;">

## From Folders to Collections

Vector collections emerge automatically from the folder structure within your .gbkb directory. Each folder you create becomes a distinct collection, indexed separately and activated independently. This direct mapping between physical organization and logical collections makes knowledge management intuitive—organize files into folders by topic, and those folders become the collections you reference in your scripts.

When BotServer encounters a .gbkb folder, it scans for documents in supported formats including PDF, DOCX, TXT, HTML, and Markdown. Each file's content is extracted, split into manageable chunks, converted to vector embeddings, and stored in the vector database. The folder name becomes the collection identifier you use with the USE KB keyword.

This automatic process means no manual indexing configuration is required. Add files to a folder, and they become searchable. Remove files, and they disappear from search results. The system tracks file changes through hash comparisons, triggering reindexing only when content actually changes.

## The Indexing Pipeline

Understanding the indexing pipeline helps diagnose issues and optimize performance. When a folder is processed, the system first detects which files are new or modified since the last indexing run. This incremental approach avoids reprocessing unchanged content.

For each file requiring processing, text extraction pulls readable content from the document regardless of its format. PDF extraction handles complex layouts, DOCX processing unwraps the underlying XML, and plain text formats are read directly. The extracted text preserves paragraph structure and meaningful breaks.

The chunking phase splits long documents into smaller pieces suitable for embedding and retrieval. Each chunk contains approximately 500 tokens with overlap between adjacent chunks to preserve context across boundaries. This sizing balances granularity—enabling precise matches—against coherence—keeping related information together.

Embedding generation converts each text chunk into a numerical vector representation. BotServer uses the BGE embedding model by default, producing 384-dimensional vectors that capture semantic meaning. These embeddings enable the similarity comparisons that power semantic search.

Finally, the vectors and their associated metadata are stored in the vector database, organized by collection. Each entry includes the embedding vector, the original text chunk, the source file path, and position information enabling reconstruction of context.

## Working with Collections

Activating a collection for use in conversations requires only the USE KB statement with the collection name matching the folder. Once activated, the collection becomes part of the knowledge available when answering questions.

Multiple collections can be active simultaneously, and the system searches across all of them when looking for relevant content. This capability allows bots to draw on diverse knowledge sources. A comprehensive assistant might activate employee policies, product documentation, and procedural guides, answering questions that span any combination of these areas.

The CLEAR KB keyword deactivates collections, either removing all active collections at once or targeting specific ones by name. Clearing collections frees memory and focuses search results on remaining active knowledge. Scripts that handle diverse topics might activate and clear collections as the conversation shifts between subject areas.

Collections operate at the session level, meaning activation persists until the session ends or the collection is explicitly cleared. Users can ask follow-up questions that build on retrieved knowledge without requiring reactivation between each query.

## Website Indexing

Beyond static documents, collections can include content crawled from websites. The USE WEBSITE keyword registers a URL for crawling, with the retrieved content becoming searchable alongside document-based collections.

For content that changes over time, scheduled crawling keeps the collection current. A script with SET SCHEDULE can periodically re-crawl websites, ensuring that the bot's knowledge reflects recent updates. This approach works well for documentation sites, knowledge bases, or any web content relevant to your bot's domain.

Website content goes through the same indexing pipeline as documents—text extraction, chunking, embedding, and storage. The resulting collection is indistinguishable in use from document-based collections.

## How Search Utilizes Collections

When a user asks a question and collections are active, the search process finds relevant content automatically. The system embeds the query using the same model that indexed the documents, ensuring that queries and content exist in the same semantic space.

Vector similarity search identifies chunks whose embeddings are closest to the query embedding. The system retrieves the top matches from each active collection, then combines and ranks them by relevance. This process typically completes in milliseconds even for large collections.

The most relevant chunks become part of the context provided to the language model when generating a response. The model sees both the user's question and the retrieved information, enabling it to produce answers grounded in your organization's actual documentation.

This entire process happens transparently. Developers don't write search queries or handle result sets. Users don't know that retrieval is occurring. The system simply provides knowledgeable responses informed by the activated collections.

## Embedding Configuration

The embedding model determines how meaning is captured in vectors and significantly influences search quality. BotServer uses a locally-running BGE model by default, configured through the embedding URL and model path settings in config.csv.

The default model provides good general-purpose performance for English content. Organizations with specialized vocabulary or multilingual requirements might benefit from alternative models. The embedding infrastructure supports any compatible model, allowing customization for specific domains.

Changing embedding models requires reindexing existing collections since embeddings from different models aren't comparable. Plan model changes carefully, accounting for the reprocessing time required for large document collections.

## Collection Management Practices

Effective collection organization follows the principle of coherent groupings. Each folder should contain documents about a related topic area, enabling targeted activation. Overly broad collections that mix unrelated content produce noisier search results than focused collections containing cohesive material.

Clear naming conventions help scripts remain readable and maintainable. Collection names should indicate their content clearly enough that someone reading a script understands what knowledge is being activated without examining the folder contents.

Regular content maintenance keeps collections valuable. Remove outdated documents that might produce incorrect answers. Update files when information changes. Schedule website re-crawls frequently enough that cached content doesn't become stale.

Monitoring collection usage helps identify optimization opportunities. If certain collections are rarely activated, consider whether they should exist separately or merge into related collections. If search results frequently miss relevant content, examine whether documents are organized in ways that match how users think about topics.

## Performance Considerations

Collection size affects both memory usage and search performance. Larger collections require more storage for their embeddings and take longer to search, though the impact is usually modest given vector database optimizations. Very large collections might benefit from subdivision into more focused subcollections.

Active collection count influences context-building overhead. Each active collection contributes potential results that must be ranked and filtered. Activating only relevant collections for each conversation keeps search focused and efficient.

Embedding generation represents the primary indexing cost. Initial indexing of large document sets takes time proportional to total content size. Incremental updates process only changed files, making ongoing maintenance much faster than initial setup.

Caching at multiple levels improves performance for common patterns. Frequently accessed chunks remain in memory. Repeated queries benefit from result caching. The system automatically manages these caches without requiring configuration.

## Summary

Vector collections bridge the gap between static documents and dynamic conversation knowledge. The automatic indexing pipeline transforms folder contents into searchable collections without requiring manual configuration. Simple activation through USE KB makes knowledge available, while the underlying vector search finds relevant content based on meaning rather than keywords. Thoughtful organization of documents into focused collections maximizes the value of this powerful capability.