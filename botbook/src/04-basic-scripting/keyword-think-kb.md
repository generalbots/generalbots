# THINK KB

Perform explicit knowledge base reasoning with structured results.

## Syntax

```basic
results = THINK KB "query_text"
results = THINK KB query_variable
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `query_text` | String | The question or search query to execute |
| `query_variable` | Variable | Variable containing the search query |

## Description

Unlike automatic KB search (USE KB), THINK KB provides explicit control over knowledge base queries with structured results for analysis and decision-making.

## Return Structure

```basic
{
  "results": [
    {
      "content": "Relevant text content",
      "source": "document.pdf", 
      "kb_name": "knowledge_base_name",
      "relevance": 0.85,
      "tokens": 150
    }
  ],
  "summary": "Brief summary of findings",
  "confidence": 0.78,
  "total_results": 5,
  "sources": ["doc1.pdf", "doc2.md"],
  "query": "original search query",
  "kb_count": 2
}
```

## Examples

### Basic Usage

```basic
USE KB "policies"
results = THINK KB "What is the remote work policy?"

TALK results.summary
PRINT "Confidence: " + results.confidence

FOR i = 0 TO results.results.length - 1
  result = results.results[i]
  PRINT "Source: " + result.source
  PRINT "Content: " + result.content
NEXT i
```

### Decision Making with Confidence

```basic
USE KB "technical_docs"
results = THINK KB "How to fix database errors?"

IF results.confidence > 0.8 THEN
  TALK "I found reliable information: " + results.summary
  top_result = results.results[0]
  TALK "From: " + top_result.source
  TALK top_result.content
ELSE IF results.confidence > 0.5 THEN
  TALK "Found some information, but not completely certain"
ELSE
  TALK "Couldn't find reliable information. Consult additional resources."
END IF
```

### Multi-Stage Reasoning

```basic
USE KB "research_papers"

' Stage 1: General search
general = THINK KB "machine learning applications"

' Stage 2: Specific search based on findings
IF general.confidence > 0.6 THEN
  specific_query = "deep learning " + general.results[0].content.substring(0, 50)
  specific = THINK KB specific_query
  
  TALK "Overview: " + general.summary
  TALK "Details: " + specific.summary
END IF
```

### Source Filtering

```basic
results = THINK KB "contract clauses"

pdf_results = []
FOR i = 0 TO results.results.length - 1
  result = results.results[i]
  IF result.source CONTAINS ".pdf" THEN
    pdf_results.push(result)
  END IF
NEXT i

TALK "Found " + pdf_results.length + " PDF results"
```

## Key Differences from USE KB

| Feature | USE KB (Automatic) | THINK KB (Explicit) |
|---------|-------------------|-------------------|
| **Trigger** | Automatic on user questions | Explicit keyword execution |
| **Control** | Behind-the-scenes | Full programmatic control |
| **Results** | Injected into LLM context | Structured data for processing |
| **Confidence** | Not exposed | Explicit confidence scoring |
| **Filtering** | Not available | Full result filtering |

## Best Practices

1. **Activate KBs First**: Use `USE KB` to activate knowledge bases
2. **Check Confidence**: Use thresholds for decision making
3. **Handle Empty Results**: Check `total_results` before accessing array
4. **Filter by Relevance**: Consider filtering results below 0.5 relevance
5. **Cache Results**: Store in variables for multiple uses

## Error Handling

```basic
TRY
  results = THINK KB user_query
  IF results.total_results = 0 THEN
    TALK "No information found for: " + user_query
  END IF
CATCH error
  TALK "Search failed: " + error.message
END TRY
```

## Performance

- **Search Time**: 100-500ms depending on KB size
- **Memory**: Results cached for session
- **Token Limits**: Respects 2000 token default limit
- **Concurrent**: Searches all active KBs in parallel

## See Also

- [USE KB](./keyword-use-kb.md) - Activate knowledge bases
- [CLEAR KB](./keyword-clear-kb.md) - Deactivate knowledge bases
- [KB Statistics](./keyword-kb-statistics.md) - Knowledge base metrics
