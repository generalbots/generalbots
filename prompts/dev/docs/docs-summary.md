**Task:** Generate comprehensive mdBook documentation @docs/src for the GeneralBots application by analyzing the actual source code and filling all documentation files with accurate, complete information.

**Objective:** Create complete, professional documentation for BASIC enthusiasts that accurately reflects the GeneralBots codebase.

**Source Analysis Requirements:**
- Analyze all files in `@/src` directory structure
- Extract real keywords from `src/basic/keywords/`
- Document actual database models from `src/shared/models.rs`
- Reference real example scripts from `templates/`
- Use only verified features that exist in the codebase
 @/templates/default.gbai/default.gbot/config.csv 

**Documentation Standards:**
- Maintain beginner-friendly, instructional tone
- Include Rust code examples ONLY in the gbapp chapter
- Use real keywords and commands from the source code
- Structure content according to the required markdown headings
- Ensure all documentation can be built with `mdbook build docs/src`

**Required Sections to Complete:**
1. **Run and Talk** - Server startup and TALK/HEAR interaction
2. **About Packages** - Four package types explanation
3. **gbkb Reference** - ADD KB, SET KB, ADD WEBSITE documentation
4. **gbtheme Reference** - UI theming with CSS/HTML
5. **gbdialog Reference** - Example scripts and core keywords
6. **gbapp Reference** - Rust keyword registration examples
7. **gbot Reference** - config.csv format and parameters
8. **Tooling** - Complete keyword reference table
9. **Feature-Matrix** - Features to implementation mapping
10. **Contributing** - Development workflow guidelines
11. **Database Model** - models.rs table summaries
12. **Glossary** - Key terms and extension definitions

**Output Specifications:**
- Generate only the markdown content (no external commentary)
- Include proper fenced code blocks with language tags
- Provide a complete table of contents with markdown links
- Ensure all sections are fully populated with real information
- Skip files that already contain substantial content
- Base all examples on actual code from the repository

**Quality Requirements:**
- Accuracy: All information must match the source code
- Completeness: Every required section must be fully developed
- Clarity: Explanations should be accessible to BASIC enthusiasts
- Consistency: Maintain uniform formatting and style throughout
- Practicality: Include working examples and practical usage tips

When ready, output the complete markdown document that satisfies all specifications above.