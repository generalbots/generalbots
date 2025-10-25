# Chapter 02: About Packages

GeneralBots uses a package-based architecture where different file extensions define specific components of the bot application. Each package type serves a distinct purpose in the bot ecosystem.

## Package Types

- **.gbai** - Application architecture and structure
- **.gbdialog** - Conversation scripts and dialog flows
- **.gbkb** - Knowledge base collections
- **.gbot** - Bot configuration
- **.gbtheme** - UI theming
- **.gbdrive** - File storage

## Package Structure

Each package is organized in a specific directory structure within the MinIO drive storage:

```
bucket_name.gbai/
├── .gbdialog/
│   ├── start.bas
│   ├── auth.bas
│   └── generate-summary.bas
├── .gbkb/
│   ├── collection1/
│   └── collection2/
├── .gbot/
│   └── config.csv
└── .gbtheme/
    ├── web/
    │   └── index.html
    └── style.css
```

## Package Deployment

Packages are automatically synchronized from the MinIO drive to the local file system when the bot starts. The system monitors for changes and hot-reloads components when possible.
