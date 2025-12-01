# Chapter 13: Contributing

Join the General Bots community and help improve the platform.

## Quick Links

| Resource | Purpose |
|----------|---------|
| [GitHub](https://github.com/GeneralBots/botserver) | Source code, issues |
| [Discussions](https://github.com/GeneralBots/botserver/discussions) | Q&A, ideas |
| [Blog](https://pragmatismo.com.br/blog) | Updates, tutorials |

## How to Contribute

### Code Contributions

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write tests
5. Submit a pull request

### Documentation

- Fix typos and errors
- Add examples
- Improve clarity
- Translate content

### Community Support

- Answer questions in discussions
- Share your bots and templates
- Report bugs with reproduction steps
- Suggest features

## Development Setup

```bash
git clone https://github.com/GeneralBots/botserver
cd botserver
cargo build
./target/debug/botserver
```

## What We Accept

✅ Bug fixes with tests  
✅ Performance improvements  
✅ New BASIC keywords (if broadly useful)  
✅ Documentation improvements  
✅ Security enhancements  

## What We Don't Accept

❌ Vendor-specific integrations  
❌ Undocumented code  
❌ Code without tests  
❌ Features achievable with existing BASIC + LLM  

## Chapter Contents

- [Development Setup](./setup.md) - Build environment
- [Testing Guide](./testing.md) - Running tests
- [Documentation](./documentation.md) - Writing docs
- [Pull Requests](./pull-requests.md) - PR process
- [Community Guidelines](./community.md) - Code of conduct
- [IDEs](./ide-extensions.md) - Editor support

## See Also

- [Architecture](../chapter-07-gbapp/README.md) - System design
- [BASIC Reference](../chapter-06-gbdialog/README.md) - Scripting language