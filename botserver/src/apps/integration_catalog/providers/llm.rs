use super::super::actions::AI_PLATFORM_ACTIONS;
use super::super::auth::{API_KEY, KEYLESS, OPENAI_COMPATIBLE};
use super::super::types::{Category, Priority, ProviderSeed, Status, Strategy};
use super::{provider, ProviderSpec};

/// LLM inference providers. These seed the integration catalog so users can
/// connect a provider in Settings (Integrations → LLM Providers) and then
/// assign providers to the three Vibe agent types (Reasoning / Agentic /
/// Fast) in the Vibe settings section.
///
/// The free tier ships the Kilo gateway only (`api.kilo.ai`): every slug below
/// maps to a model that is currently served with the `:free` suffix, and each
/// one was checked against the gateway model list at build time. LLM7 was
/// removed from the catalog because its free tier requires an account key and
/// its keyless request returns 401, and the OVHcloud endpoint was removed from
/// the product selection because it answers plain questions in prose instead of
/// following the native `vm_exec` tool protocol.
pub(super) const PROVIDERS: &[ProviderSeed] = &[
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kilo-dots",
            "Kilo · Dots 3 Note",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kilocode.ai/docs"),
                auth: &KEYLESS,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kilo-nemotron-ultra",
            "Kilo · Nemotron 3 Ultra 550B",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kilocode.ai/docs"),
                auth: &KEYLESS,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kilo-nemotron-super",
            "Kilo · Nemotron 3.5 Super VL",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kilocode.ai/docs"),
                auth: &KEYLESS,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kilo-nemotron-lightning",
            "Kilo · Nemotron 3.5 Lightning",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kilocode.ai/docs"),
                auth: &KEYLESS,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kilo-nano",
            "Kilo · Nemotron 3 Nano Omni",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kilocode.ai/docs"),
                auth: &KEYLESS,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kilo-cohere",
            "Kilo · Cohere North Mini",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kilocode.ai/docs"),
                auth: &KEYLESS,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kilo-openrouter",
            "Kilo · OpenRouter Free",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kilocode.ai/docs"),
                auth: &KEYLESS,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kilo-flint",
            "Kilo · Flint 1.0",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kilocode.ai/docs"),
                auth: &KEYLESS,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kilo-xing",
            "Kilo · Xing 4.0 29B",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kilocode.ai/docs"),
                auth: &KEYLESS,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "openai",
            "OpenAI",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://platform.openai.com/api-keys"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "claude",
            "Claude",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://console.anthropic.com/settings/keys"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "azure-claude",
            "Azure Claude",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some(
                    "https://learn.microsoft.com/azure/ai-foundry/openai/how-to/create-resource",
                ),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "azure-gpt5",
            "Azure GPT-5",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some(
                    "https://learn.microsoft.com/azure/ai-foundry/openai/how-to/create-resource",
                ),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "glm",
            "GLM",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://docs.z.ai/guides/develop/http/introduction"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "bedrock",
            "Bedrock",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some(
                    "https://docs.aws.amazon.com/bedrock/latest/userguide/getting-started.html",
                ),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "vertex",
            "Vertex",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some(
                    "https://cloud.google.com/vertex-ai/generative-ai/docs/start/quickstarts/quickstart-multimodal",
                ),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "kiro",
            "Kiro",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://kiro.dev/docs/"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "deepinfra",
            "DeepInfra",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://deepinfra.com/docs/getting-started"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "cerebras",
            "Cerebras",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://inference-docs.cerebras.ai/introduction"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "fireworks",
            "Fireworks AI",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://docs.fireworks.ai/getting-started/introduction"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "together",
            "Together AI",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://docs.together.ai/docs/quickstart"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "openrouter",
            "OpenRouter",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://openrouter.ai/docs/quickstart"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "requesty",
            "Requesty",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://docs.requesty.ai/"),
                auth: &API_KEY,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "openai-compatible",
            "OpenAI Compatible",
            ProviderSpec {
                category: Category::Llm,
                strategy: Strategy::Improve,
                status: Status::Built,
                priority: Priority::Must,
                module: None,
                official_docs: Some("https://platform.openai.com/docs/api-reference/chat"),
                auth: &OPENAI_COMPATIBLE,
                actions: AI_PLATFORM_ACTIONS,
            },
        )
    },
];
