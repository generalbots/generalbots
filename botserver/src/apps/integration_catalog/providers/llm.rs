use super::super::actions::AI_PLATFORM_ACTIONS;
use super::super::auth::API_KEY;
use super::super::types::{Category, Priority, ProviderSeed, Status, Strategy};
use super::provider;

/// LLM inference providers. These seed the integration catalog so users can
/// connect a provider key in Settings (Integrations → LLM Providers) and
/// then assign providers to the three Vibe agent types (Reasoning / Agentic /
/// Fast) in the Vibe settings section. The set mirrors the Cloud store
/// catalog (`/api/cloud/llm-providers`) plus the fast free-tier providers,
/// all consumed through OpenAI-compatible chat-completions endpoints.
pub(super) const PROVIDERS: &[ProviderSeed] = &[
    ProviderSeed {
        llm_available: true,
        ..provider(
            "groq",
            "Groq",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://console.groq.com/docs"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "siliconflow",
            "SiliconFlow",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://docs.siliconflow.cn/en/userguide/introduction"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "gemini",
            "Google Gemini",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://ai.google.dev/gemini-api/docs"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "zai",
            "Z.AI (Zhipu)",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://open.bigmodel.cn/dev/api"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "zhipu",
            "GLM (Zhipu AI)",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://open.bigmodel.cn/dev/api"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "alibaba",
            "Qwen (Alibaba Cloud)",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://help.aliyun.com/zh/model-studio"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "deepseek",
            "DeepSeek",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://platform.deepseek.com"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "minimax",
            "MiniMax",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://platform.minimaxi.com"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "yi",
            "Yi (01.AI)",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://platform.lingyiwanwu.com"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "openai",
            "OpenAI",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://platform.openai.com/docs"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "anthropic",
            "Anthropic",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://docs.anthropic.com"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "google",
            "Google (Vertex AI)",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://cloud.google.com/vertex-ai"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "meta",
            "Meta Llama",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://ai.meta.com/llama"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "mistral",
            "Mistral AI",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://docs.mistral.ai"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
    ProviderSeed {
        llm_available: true,
        ..provider(
            "amazon",
            "Amazon Bedrock",
            Category::Developer,
            Strategy::Improve,
            Status::Built,
            Priority::Must,
            None,
            Some("https://aws.amazon.com/bedrock"),
            &API_KEY,
            AI_PLATFORM_ACTIONS,
        )
    },
];