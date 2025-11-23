## LLM Strategy & Workflow

### Fallback Strategy (After 3 attempts / 10 minutes):
When initial attempts fail, sequentially try these LLMs:
1. **DeepSeek-V3-0324** (good architect, adventure, reliable, let little errors just to be fixed by gpt-*)
1. **gpt-5-chat** (slower, let warnings...)
1. **gpt-oss-120b**
1. **Claude (Web)**: Copy only the problem statement and create unit tests. Create/extend UI.

### Development Workflow:
- **One requirement at a time** with sequential commits.
- Start editing docs before any code, explain user behaviour in docs first with LLM, before code in rust. 
- Spend time on design and architecture before coding. Code structure of packages and skeltons with ideas concretized from documentation. But focus in docs first, because LLM can help with design and architecture, and code better.
- **On unresolved error**: Stop and use add-req.sh, and consult Claude for guidance.  with DeepThining in DeepSeek also, with Web turned on.
- **Change progression**: Start with DeepSeek, conclude with gpt-oss-120b
- If a big req. fail, specify a @code file that has similar pattern or sample from official docs.
- **Warning removal**: Last task before commiting, create a task list of warning removal and work with cargo check.
- **Final validation**: Use prompt "cargo check" with gpt-oss-120b
- Be humble, one requirement, one commit. But sometimes, freedom of caos is welcome - when no deadlines are set.
- Fix manually in case of dangerous trouble.
- Keep in the source codebase only deployed and tested source, no lab source code in main project. At least, use optional features to introduce new behaviour gradually in PRODUCTION.
- Transform good articles into prompts for the coder.
- Switch to libraries that have LLM affinity (LLM knows the library, was well trained).
- Ensure 'continue' on LLMs, they can EOF and say are done, but got more to output.
