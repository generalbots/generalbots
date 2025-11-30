# RUN PYTHON / RUN JAVASCRIPT / RUN BASH

Executes code in a sandboxed environment. Enables safe execution of dynamic code for data processing, calculations, and automation tasks.

## Syntax

```basic
result = RUN PYTHON "code"
result = RUN JAVASCRIPT "code"
result = RUN BASH "code"
```

```basic
result = RUN PYTHON WITH FILE "script.py"
result = RUN JAVASCRIPT WITH FILE "script.js"
result = RUN BASH WITH FILE "script.sh"
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | String | Inline code to execute |
| `filepath` | String | Path to script file (with `WITH FILE` variant) |

## Returns

The output (stdout) from the executed code as a string.

## Description

The `RUN` keywords execute code in isolated, sandboxed environments. This provides:

- **Security** - Code runs in isolated containers (LXC, Docker, or Firecracker)
- **Flexibility** - Use the right language for the task
- **Safety** - Resource limits prevent runaway processes
- **Integration** - Pass data between BASIC and other languages

The sandbox prevents:
- File system access outside designated areas
- Network access (unless explicitly enabled)
- System calls and privilege escalation
- Excessive CPU or memory usage

## Examples

### Basic Python Execution

```basic
' Simple calculation
result = RUN PYTHON "print(2 + 2)"
TALK "2 + 2 = " + result

' Data processing
code = "
import json
data = [1, 2, 3, 4, 5]
print(json.dumps({'sum': sum(data), 'avg': sum(data)/len(data)}))
"
stats = RUN PYTHON code
TALK "Statistics: " + stats
```

### JavaScript for JSON Processing

```basic
' Parse and transform JSON
jsonData = GET "https://api.example.com/data"
code = "
const data = JSON.parse('" + jsonData + "');
const transformed = data.items.map(i => ({
    id: i.id,
    name: i.name.toUpperCase()
}));
console.log(JSON.stringify(transformed));
"
result = RUN JAVASCRIPT code
TALK result
```

### Bash for System Tasks

```basic
' List files and get disk usage
result = RUN BASH "ls -la /data && df -h"
TALK "System info:\n" + result
```

### Run Script from File

```basic
' Execute a Python script from .gbdrive
result = RUN PYTHON WITH FILE "scripts/analyze_data.py"
TALK "Analysis complete: " + result

' Run a bash script
output = RUN BASH WITH FILE "scripts/backup.sh"
PRINT "Backup output: " + output
```

### Data Pipeline

```basic
' Fetch data, process with Python, store result
rawData = GET "https://api.example.com/sales"

pythonCode = "
import json
import statistics

data = json.loads('''" + rawData + "''')
sales = [item['amount'] for item in data]

result = {
    'total': sum(sales),
    'average': statistics.mean(sales),
    'median': statistics.median(sales),
    'std_dev': statistics.stdev(sales) if len(sales) > 1 else 0
}
print(json.dumps(result))
"

analysis = RUN PYTHON pythonCode
SAVE "sales_analysis.csv", analysis
TALK "Sales analysis saved!"
```

### Machine Learning Inference

```basic
' Run ML model for prediction
inputData = #{ features: [1.5, 2.3, 4.1, 0.8] }

code = "
import json
import pickle

# Load pre-trained model (stored in sandbox)
with open('/data/model.pkl', 'rb') as f:
    model = pickle.load(f)

input_data = " + JSON(inputData) + "
prediction = model.predict([input_data['features']])[0]
print(json.dumps({'prediction': float(prediction)}))
"

result = RUN PYTHON code
prediction = JSON_PARSE(result)
TALK "Predicted value: " + prediction.prediction
```

### Image Processing

```basic
' Process an uploaded image
imagePath = UPLOAD userImage, "uploads/"

code = "
from PIL import Image
import json

img = Image.open('/data/" + imagePath + "')
width, height = img.size
format = img.format

# Resize if too large
if width > 1920:
    ratio = 1920 / width
    new_size = (1920, int(height * ratio))
    img = img.resize(new_size)
    img.save('/data/resized_" + imagePath + "')

print(json.dumps({
    'original_size': [width, height],
    'format': format,
    'resized': width > 1920
}))
"

result = RUN PYTHON code
TALK "Image processed: " + result
```

### Multi-Language Pipeline

```basic
' Use different languages for different strengths
data = GET "https://api.example.com/raw-data"

' Step 1: Clean data with Python (pandas)
cleanCode = "
import pandas as pd
import json
df = pd.read_json('''" + data + "''')
df = df.dropna()
df = df[df['value'] > 0]
print(df.to_json(orient='records'))
"
cleanedData = RUN PYTHON cleanCode

' Step 2: Transform with JavaScript (fast JSON manipulation)
transformCode = "
const data = JSON.parse('" + cleanedData + "');
const result = data.reduce((acc, item) => {
    acc[item.category] = (acc[item.category] || 0) + item.value;
    return acc;
}, {});
console.log(JSON.stringify(result));
"
aggregated = RUN JAVASCRIPT transformCode

TALK "Results: " + aggregated
```

## Sandbox Configuration

### Runtime Options

The sandbox supports multiple isolation backends:

| Runtime | Security | Performance | Requirements |
|---------|----------|-------------|--------------|
| `LXC` | High | Excellent | LXC installed |
| `Docker` | High | Good | Docker daemon |
| `Firecracker` | Highest | Good | Firecracker binary |
| `Process` | Low | Best | None (fallback) |

### Config.csv Options

```csv
name,value
sandbox-runtime,lxc
sandbox-timeout,30
sandbox-memory-mb,512
sandbox-cpu-percent,50
sandbox-network,false
sandbox-python-packages,numpy,pandas,pillow
sandbox-allowed-paths,/data,/tmp
```

| Option | Default | Description |
|--------|---------|-------------|
| `sandbox-runtime` | `lxc` | Isolation backend to use |
| `sandbox-timeout` | `30` | Maximum execution time (seconds) |
| `sandbox-memory-mb` | `512` | Memory limit in MB |
| `sandbox-cpu-percent` | `50` | CPU usage limit |
| `sandbox-network` | `false` | Allow network access |
| `sandbox-python-packages` | (none) | Pre-installed Python packages |
| `sandbox-allowed-paths` | `/data,/tmp` | Accessible filesystem paths |

## Security Considerations

### What's Blocked

- Direct file system access outside sandbox
- Network connections (unless `sandbox-network=true`)
- System calls (fork, exec, etc.)
- Environment variable access
- Process spawning

### What's Allowed

- Standard library operations
- File I/O within `/data` and `/tmp`
- Computation up to resource limits
- Pre-approved packages

### Input Sanitization

```basic
' IMPORTANT: Always sanitize user input before embedding in code
userInput = HEAR input
' Remove potential code injection
safeInput = REPLACE(userInput, "'", "\'")
safeInput = REPLACE(safeInput, '"', '\"')

code = "print('User said: " + safeInput + "')"
result = RUN PYTHON code
```

## Error Handling

```basic
' Handle execution errors
ON ERROR RESUME NEXT

result = RUN PYTHON "
import nonexistent_module
print('hello')
"

IF ERROR THEN
    TALK "Code execution failed: " + ERROR_MESSAGE
    ' Fall back to alternative approach
ELSE
    TALK result
END IF
```

## Resource Limits

| Resource | Default | Maximum |
|----------|---------|---------|
| Execution time | 30s | 300s |
| Memory | 512 MB | 4096 MB |
| CPU | 50% | 100% |
| Output size | 1 MB | 10 MB |
| File writes | 10 MB | 100 MB |

## Related Keywords

| Keyword | Description |
|---------|-------------|
| [`LLM`](./keyword-llm.md) | AI-generated code execution |
| [`GET`](./keyword-get.md) | Fetch data for processing |
| [`SAVE`](./keyword-save.md) | Store processed results |

## Best Practices

1. **Keep code snippets small** - Large scripts should use `WITH FILE`
2. **Sanitize all inputs** - Never trust user data in code strings
3. **Set appropriate timeouts** - Match timeout to expected execution time
4. **Use the right language** - Python for data, JS for JSON, Bash for files
5. **Handle errors gracefully** - Code can fail for many reasons
6. **Pre-install packages** - Don't pip install in every execution
7. **Log execution times** - Monitor for performance issues

## Limitations

- No persistent state between executions
- No GPU access (use dedicated ML endpoints instead)
- No interactive input (stdin)
- No graphical output (use file output instead)
- Package installation not allowed at runtime

## See Also

- [Code Sandbox Architecture](../chapter-07-gbapp/containers.md) - Technical details
- [Security Features](../chapter-12-auth/security-features.md) - Sandbox security model
- [Data Operations](./keywords-data.md) - Alternative data processing keywords