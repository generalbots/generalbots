# The gbapp Philosophy: Let Machines Do Machine Work

## Core Principle: Automation First

In 2025, the gbapp philosophy is simple and powerful: if a machine can do the work, let it do the work. This principle guides every decision about how to build and extend General Bots. Rather than writing code manually, you describe what you need and let AI handle the implementation details.


## The Hierarchy of Development

The development approach in General Bots follows a clear hierarchy based on what percentage of work falls into each category.

### LLM First (90% of cases)

The vast majority of work should be handled by letting AI write the code for you. Instead of implementing complex logic yourself, describe what you want in natural language and let the LLM generate the solution.

```basic
' Don't write complex logic - describe what you want
result = LLM "Generate a function that validates email addresses and returns true/false: " + email
```

### BASIC for Flow Control (9% of cases)

BASIC serves as the orchestration layer that connects AI calls together. Think of it as glue code that manages the flow between different operations. The logic itself lives in LLM calls while BASIC handles sequencing and data flow.

```basic
' BASIC is just glue between AI calls
data = GET "api/data"
processed = LLM "Process this: " + data
SET "results", processed
```

### Rust for Core Only (1% of cases)

Writing Rust code should be reserved for rare situations where you are contributing new keywords to the core platform, building fundamental infrastructure that many bots will use, or optimizing critical performance paths where every millisecond matters. Most developers will never need to write Rust because BASIC and LLM calls handle nearly every use case.


## What gbapp Really Is

Understanding what gbapp is and is not helps clarify the development model.

The gbapp concept is not about external plugin packages that you download separately. It is not about separate npm modules or package managers. It is not a way to bypass BASIC and write custom code. It is not about runtime extensions that modify behavior dynamically.

Instead, gbapp represents virtual crates inside the `src/` directory that are Rust modules compiling together into a single binary. The concept serves as a bridge between older plugin-based thinking and the modern integrated approach. It provides a familiar mental model for developers who want to contribute to the platform. Most importantly, gbapp embodies a mindset of coding through automation rather than manual implementation.


## Real-World Examples

The contrast between traditional development and the General Bots approach becomes clear through examples.

### Traditional Approach

In the old way of thinking, you might write hundreds of lines of custom Node.js, Python, or C# code for data validation. A function like validateComplexBusinessRules would contain extensive logic handling edge cases, format checking, and business rule verification. This code requires maintenance, testing, and documentation.

### The General Bots Approach

With the automation-first philosophy, the same task takes three lines. You fetch your business rules from a file, ask the LLM to validate data against those rules, and handle the result. The AI understands the rules and applies them correctly without you implementing the validation logic.

```basic
' 3 lines - let AI handle complexity
rules = GET "business-rules.txt"
validation = LLM "Validate this data against these rules: " + data + " Rules: " + rules
IF validation CONTAINS "valid" THEN TALK "Approved" ELSE TALK "Rejected: " + validation
```


## The Multi-SDK Reality

You do not need separate SDKs or plugins for different services. Everything integrates through BASIC combined with LLM calls.

### Integrating Any API

When you need to work with an external API, you do not need to find and install an SDK. Just fetch the data and let the LLM interpret and format it.

```basic
' No SDK needed - just describe what you want
data = GET "https://server/data"
answer = LLM "Do a good report from this json: " + data
TALK answer
```

### Working with Any Database

Database operations do not require an ORM or query builder. The AI understands SQL and can generate queries from natural language descriptions.

```basic
' No ORM needed - AI understands SQL
results = FIND "users", "all users who logged in today"
```

### Processing Any Format

You do not need parser libraries for different file formats. The LLM can transform data between formats based on your description.

```basic
' No parser library needed
xml_data = GET "complex.xml"
json = LLM "Convert this XML to JSON: " + xml_data
SET BOT MEMORY "processed_data", json
```


## When to Write Code

Understanding when each approach applies helps you work efficiently.

### Use LLM When

LLM calls are appropriate for processing unstructured data, implementing business logic, transforming between formats, making decisions, generating content, and analyzing patterns. This covers roughly ninety percent of everything you might want to do.

### Use BASIC When

BASIC code handles orchestrating AI calls in sequence, simple flow control with conditionals and loops, managing state and variables, and connecting different systems together. Think of BASIC as the glue that holds everything together.

### Use Rust When

Rust development is only necessary when building new keywords that will become part of the core platform, creating a new gbapp module in the `src/` directory, performing system-level optimization for critical paths, or contributing new features that will benefit all users. Almost no one needs to write Rust for their bots.


## The gbapp Mindset

Shifting your thinking is the most important part of adopting this philosophy.

Stop thinking about how to code a solution, what library you need to import, or how to extend the system with plugins. Start thinking about how to describe what you want to AI, what the simplest BASIC flow looks like, and how your patterns could help everyone using the platform.


## Data Enrichment Example

Consider a data enrichment task that pulls information about companies from their websites.

The traditional approach requires over a thousand lines of code spread across multiple npm packages. You need complex error handling for network requests, HTML parsing for different website structures, and a maintenance nightmare as websites change their formats.

The General Bots approach handles the same task in a few lines. You find companies that need enrichment, loop through them, fetch each website, ask the LLM to extract company information, and save the results. The AI handles all the complexity of parsing different website formats.

```basic
items = FIND "companies", "needs_enrichment=true"
FOR EACH item IN items
    website = WEBSITE OF item.company
    page = GET website
    enriched = LLM "Extract company info from: " + page
    SET "companies", "id=" + item.id, "data=" + enriched
NEXT
```


## Report Generation Example

Generating reports traditionally requires a custom reporting engine, template systems, complex formatting logic, and PDF libraries. That infrastructure takes significant development and ongoing maintenance.

With General Bots, you find the relevant data, ask the LLM to create an executive summary, and generate a site with the results. Three lines replace an entire reporting infrastructure.

```basic
data = FIND "sales", "month=current"
report = LLM "Create executive summary from: " + data
CREATE SITE "report", "template", report
```


## The Ultimate Test

Before writing any code, ask yourself three questions in order. First, can the LLM do this? The answer is usually yes. Second, can BASIC orchestrate it? Almost always yes. Third, do you really need Rust? Almost never.

Only proceed to writing custom code if you have genuinely exhausted the first two options. The LLM and BASIC combination handles far more than most developers initially expect.


## Benefits of This Approach

### For Developers

This approach enables development that is roughly one hundred times faster than traditional coding. You have no dependency management headaches and no version conflicts between packages. The maintenance burden drops dramatically because there is no custom code to maintain. You can focus on business logic and what you want to accomplish rather than implementation details.

### For Organizations

Organizations benefit from reduced complexity in their bot deployments. Maintenance costs drop because there is less custom code to support. Iterations happen faster since changes involve modifying descriptions rather than rewriting code. There is no vendor lock-in to specific libraries or frameworks. Anyone in the organization can contribute because they do not need traditional programming skills.

### For the Community

Shared improvements benefit everyone using the platform. There is no fragmentation into incompatible plugin ecosystems. Users experience consistency across different bots and deployments. The community advances collectively rather than each organization maintaining separate extensions.


## The Future is Already Here

In 2025, this approach is not aspirational but reality. Applications built entirely with BASIC and LLM calls run in production today. Most use cases require zero custom code. AI handles complexity better than hand-written algorithms in many domains. Machines do machine work while humans focus on human work like understanding requirements and making decisions.


## Migration Path

### From Extensions to Virtual Crates

If you have existing plugin-style extensions, the migration path consolidates them into the main source tree. An old extension might have been a separate folder with hundreds of lines of JavaScript, a package.json, and complex logic. The new approach places a small Rust module in `src/` that registers BASIC keywords, while the actual logic moves to a few lines of BASIC in your `.gbdialog` folder that leverage LLM calls.

### From Code to Descriptions

Migration from traditional code involves converting algorithms into natural language descriptions. Instead of writing the logic to process data, you describe what processing you need and let the LLM implement it.

### From Libraries to LLM

Instead of importing twenty npm packages for various functionality, you make single LLM calls with descriptions of what you need. The AI has knowledge of countless libraries and formats built into its training.


## Development Guidelines

Follow these practices to work effectively with the automation-first philosophy. Describe problems to the LLM in clear, specific terms. Use BASIC as minimal glue between AI operations. Contribute keywords to the core when you discover patterns that would benefit everyone. Share your patterns with the community so others can learn. Think automation-first for every task you encounter.

Avoid common mistakes that fight against this philosophy. Do not write complex algorithms when a description would suffice. Do not build separate plugins that fragment the ecosystem. Do not create custom frameworks that add unnecessary complexity. Do not maintain separate codebases when everything should be in one place. Do not fight the machine by insisting on manual implementation.


## The Virtual Crate Architecture

Each gbapp is now a module in the `src/` directory. The structure maps conceptually familiar package names to Rust modules. The core gbapp lives in `src/core/`. The BASIC interpreter is `src/basic/`. Channel adapters are in `src/channels/`. Your contribution would go in `src/your_feature/`. This elegant mapping preserves the conceptual model of separate packages while leveraging Rust's module system and compiling everything into a single optimized binary.


## Conclusion

The gbapp concept in 2025 has evolved from external packages to virtual crates. These Rust modules inside `src/` compile into a single, optimized binary while preserving the familiar mental model of separate functional packages.

The philosophy remains constant: machines are better at machine work. Your job is to describe what you want, not implement how to do it. The combination of BASIC for orchestration and LLM for logic eliminates the need for traditional programming in almost all cases.


## Examples Repository

The `/templates/` directory contains real-world examples of applications built entirely with BASIC and LLM calls. A CRM system requires about fifty lines of BASIC. Email automation needs around thirty lines. Data pipelines work in twenty lines. Report generators take about fifteen lines. Each of these would have required thousands of lines of traditional code, demonstrating the dramatic productivity improvement this philosophy enables.