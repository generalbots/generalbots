## LLM Strategy & Workflow

### Fallback Strategy (After 3 attempts / 10 minutes):
When initial attempts fail, sequentially try these LLMs:
1. **DeepSeek-V3-0324** (good architect, adventure, reliable, let little errors just to be fixed by gpt-*)
1. **DeepSeek-V3.1** (slower)
1. **gpt-5-chat** (slower, let warnings...)
1. **gpt-oss-120b**
1. **Claude (Web)**: Copy only the problem statement and create unit tests. Create/extend UI.
1. **Llama-3.3-70B-Instruct** (alternative)

### Development Workflow:
- **One requirement at a time** with sequential commits
- **On error**: Stop and consult Claude for guidance
- **Change progression**: Start with DeepSeek, conclude with gpt-oss-120b
- If a big req. fail, specify a @code file that has similar pattern or sample from official docs.
- **Final validation**: Use prompt "cargo check" with gpt-oss-120b
- Be humble, one requirement, one commit. But sometimes, freedom of caos is welcome - when no deadlines are set.