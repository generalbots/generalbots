continue this style: generate a docusaurs using mdBook for the application in a point of view of the user, a BASIC person basic knowledge wwkring with LLM.

GeneralBots User Documentation (mdBook)
Code 
only use real keywords 
generate a docusaurs using mdBook for the application in a point of view of the user, a BASIC person basic knowledge wwkring with LLM. do not talk a line of rust outside .gbapp chapter!!! you are doing gretae
do not invent anything.
enhance that wil more text by your analysis of source code enterily 
Table of Contents
Chapter 01 - Run and Talk
Chapter 02 - About Packages
Chapter 03 - gbkb Reference (each colletion is a vectordb collectoin that can be choosen to participate in conversation context - compacted and with cache)
Chapter 04 - gbtheme Reference (change Ui themes with css and html (web/index.html)
Chapter 05 - gbdialog Reference (basic tools found in @/templates..... .bas files as REAL EXAMPLES) + keywords.rs mod keywords...
Chapter 06 - gbapp Reference (now here, you teach how to build the botserver rust and extend new crates... RUST ONLY APPEARS HERE)
Chapter 07 - gbot Reference (show @src/shard/models/ bot_config) user can put in drive .gbot with config.csv and configure various bot params. like anserMode, bot descrptoion, eec.)
Chapter 08 - Tooling (Talke about the BASIC AND HOW GET keyword call call external tools integratis features.
Chapter 09 - Feature-Matrix (List categories all features of this source code)
Chapter 10 - Contributing (
Apendix I - Database Model
Glossary
the only keywords that exists are in @/src/basic/keywords/on.rs  for .gbdialog!!!! FUCK; the only thikg .gbkb does is to store collections that can be use in the way ADD_KB keyword like @/templates/announcements.gbai/announcements.gbdialog  scripts!!! FUCK.. .gbot will contain a config.csv that mimic the @/src/shared/models.rs  bot_config so people can edit it from their DRIVE (minio bot bucket) that contains .gbkb, .gbdialog, .gbot, .gbdrive, .gbtheme!
1. **gbapp** are folder extensions within the GeneralBots package type, responsible for GeneralBots services in a back-end development environment. A General Bot App comprises four components: dialogs, models, services, and tests.
2. The **.gbui** extension pertains to the GeneralBots front-end development environment. Angular, Riot, React, and HTML enable dynamic functionality for GeneralBots services.
3. The **.gtheme** extension is utilized by GeneralBots designers using CSS and PS technologies. A theme consists of CSS files and images.
4. **.gbDialog** is an extension responsible for GeneralBots' communication with end-users through bot services. **.gbkb** is a package for executing various types of media.
5. The **.gbot** extension refers to the GeneralBots service product.
6. **.gbai** denotes the architecture construction of the GeneralBots application.

GeneralBots User Documentation
I'll create a comprehensive mdBook documentation based on the actual source code, using only real keywords and features.

GeneralBots Documentation (mdBook)
Code 
do not write any BASIC example more than templates folder start.bas auth.bas and generate-summary.bas, and include a ingle example of TOOL:  PARAM name AS string          LIKE "Abreu Silva"                DESCRIPTION "Required full name of the individual."
PARAM birthday AS date        LIKE "23/09/2001"                 DESCRIPTION "Required birth date of the individual in DD/MM/YYYY format."
PARAM email AS string         LIKE "abreu.silva@example.com"    DESCRIPTION "Required email address for contact purposes."
PARAM personalid AS integer   LIKE "12345678900"                DESCRIPTION "Required Personal ID number of the individual (only numbers)."
PARAM address AS string       LIKE "Rua das Flores, 123 - SP"   DESCRIPTION "Required full address of the individual."
DESCRIPTION  "This is a the enrollment process, called when the user wants to enrol. Once all information is collected, confirm the details and inform them that their enrollment request has been successfully submitted. Provide a polite and professional tone throughout the interaction."
SAVE "enrollments.csv", id, name, birthday, email, personalid, address that is in termos of basic samples, but show all keywords with one with independant page. inside .gbdialog section, more text

I'll create a comprehensive mdBook documentation focusing on the actual templates and all real keywords from the source code.

# Summary

[Introduction](./introduction.md)

# Part I - Getting Started

- [Chapter 01: Run and Talk](./chapter-01/README.md)
  - [Installation](./chapter-01/installation.md)
  - [First Conversation](./chapter-01/first-conversation.md)
  - [Understanding Sessions](./chapter-01/sessions.md)

# Part II - Package System

- [Chapter 02: About Packages](./chapter-02/README.md)
  - [.gbai Architecture](./chapter-02/gbai.md)
  - [.gbdialog Dialogs](./chapter-02/gbdialog.md)
  - [.gbkb Knowledge Base](./chapter-02/gbkb.md)
  - [.gbot Bot Configuration](./chapter-02/gbot.md)
  - [.gbtheme UI Theming](./chapter-02/gbtheme.md)
  - [.gbdrive File Storage](./chapter-02/gbdrive.md)

# Part III - Knowledge Base

- [Chapter 03: gbkb Reference](./chapter-03/README.md)
  - [Vector Collections](./chapter-03/vector-collections.md)
  - [Document Indexing](./chapter-03/indexing.md)
  - [Qdrant Integration](./chapter-03/qdrant.md)
  - [Semantic Search](./chapter-03/semantic-search.md)
  - [Context Compaction](./chapter-03/context-compaction.md)
  - [Semantic Caching](./chapter-03/caching.md)

# Part IV - Themes and UI

- [Chapter 04: gbtheme Reference](./chapter-04/README.md)
  - [Theme Structure](./chapter-04/structure.md)
  - [Web Interface](./chapter-04/web-interface.md)
  - [CSS Customization](./chapter-04/css.md)
  - [HTML Templates](./chapter-04/html.md)

# Part V - BASIC Dialogs

- [Chapter 05: gbdialog Reference](./chapter-05/README.md)
  - [Dialog Basics](./chapter-05/basics.md)
  - [Template Examples](./chapter-05/templates.md)
    - [start.bas](./chapter-05/template-start.md)
    - [auth.bas](./chapter-05/template-auth.md)
    - [generate-summary.bas](./chapter-05/template-summary.md)
    - [enrollment Tool Example](./chapter-05/template-enrollment.md)
  - [Keyword Reference](./chapter-05/keywords.md)
    - [TALK](./chapter-05/keyword-talk.md)
    - [HEAR](./chapter-05/keyword-hear.md)
    - [SET_USER](./chapter-05/keyword-set-user.md)
    - [SET_CONTEXT](./chapter-05/keyword-set-context.md)
    - [LLM](./chapter-05/keyword-llm.md)
    - [GET_BOT_MEMORY](./chapter-05/keyword-get-bot-memory.md)
    - [SET_BOT_MEMORY](./chapter-05/keyword-set-bot-memory.md)
    - [SET_KB](./chapter-05/keyword-set-kb.md)
    - [ADD_KB](./chapter-05/keyword-add-kb.md)
    - [ADD_WEBSITE](./chapter-05/keyword-add-website.md)
    - [ADD_TOOL](./chapter-05/keyword-add-tool.md)
    - [LIST_TOOLS](./chapter-05/keyword-list-tools.md)
    - [REMOVE_TOOL](./chapter-05/keyword-remove-tool.md)
    - [CLEAR_TOOLS](./chapter-05/keyword-clear-tools.md)
    - [GET](./chapter-05/keyword-get.md)
    - [FIND](./chapter-05/keyword-find.md)
    - [SET](./chapter-05/keyword-set.md)
    - [ON](./chapter-05/keyword-on.md)
    - [SET_SCHEDULE](./chapter-05/keyword-set-schedule.md)
    - [CREATE_SITE](./chapter-05/keyword-create-site.md)
    - [CREATE_DRAFT](./chapter-05/keyword-create-draft.md)
    - [WEBSITE OF](./chapter-05/keyword-website-of.md)
    - [PRINT](./chapter-05/keyword-print.md)
    - [WAIT](./chapter-05/keyword-wait.md)
    - [FORMAT](./chapter-05/keyword-format.md)
    - [FIRST](./chapter-05/keyword-first.md)
    - [LAST](./chapter-05/keyword-last.md)
    - [FOR EACH](./chapter-05/keyword-for-each.md)
    - [EXIT FOR](./chapter-05/keyword-exit-for.md)

# Part VI - Extending BotServer

- [Chapter 06: gbapp Reference](./chapter-06/README.md)
  - [Rust Architecture](./chapter-06/architecture.md)
  - [Building from Source](./chapter-06/building.md)
  - [Crate Structure](./chapter-06/crates.md)
  - [Service Layer](./chapter-06/services.md)
  - [Creating Custom Keywords](./chapter-06/custom-keywords.md)
  - [Adding Dependencies](./chapter-06/dependencies.md)

# Part VII - Bot Configuration

- [Chapter 07: gbot Reference](./chapter-07/README.md)
  - [config.csv Format](./chapter-07/config-csv.md)
  - [Bot Parameters](./chapter-07/parameters.md)
  - [Answer Modes](./chapter-07/answer-modes.md)
  - [LLM Configuration](./chapter-07/llm-config.md)
  - [Context Configuration](./chapter-07/context-config.md)
  - [MinIO Drive Integration](./chapter-07/minio.md)

# Part VIII - Tools and Integration

- [Chapter 08: Tooling](./chapter-08/README.md)
  - [Tool Definition](./chapter-08/tool-definition.md)
  - [PARAM Declaration](./chapter-08/param-declaration.md)
  - [Tool Compilation](./chapter-08/compilation.md)
  - [MCP Format](./chapter-08/mcp-format.md)
  - [OpenAI Tool Format](./chapter-08/openai-format.md)
  - [GET Keyword Integration](./chapter-08/get-integration.md)
  - [External APIs](./chapter-08/external-apis.md)

# Part IX - Feature Reference

- [Chapter 09: Feature Matrix](./chapter-09/README.md)
  - [Core Features](./chapter-09/core-features.md)
  - [Conversation Management](./chapter-09/conversation.md)
  - [AI and LLM](./chapter-09/ai-llm.md)
  - [Knowledge Base](./chapter-09/knowledge-base.md)
  - [Automation](./chapter-09/automation.md)
  - [Email Integration](./chapter-09/email.md)
  - [Web Automation](./chapter-09/web-automation.md)
  - [Storage and Data](./chapter-09/storage.md)
  - [Multi-Channel Support](./chapter-09/channels.md)

# Part X - Community

- [Chapter 10: Contributing](./chapter-10/README.md)
  - [Development Setup](./chapter-10/setup.md)
  - [Code Standards](./chapter-10/standards.md)
  - [Testing](./chapter-10/testing.md)
  - [Pull Requests](./chapter-10/pull-requests.md)
  - [Documentation](./chapter-10/documentation.md)

# Appendices

- [Appendix I: Database Model](./appendix-i/README.md)
  - [Schema Overview](./appendix-i/schema.md)
  - [Tables](./appendix-i/tables.md)
  - [Relationships](./appendix-i/relationships.md)

[Glossary](./glossary.md)
EOF

