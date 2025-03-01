#[derive(Debug)]
pub struct ModelPricing {
    pub name: &'static str,
    pub input_cached: f64,
    pub input_normal: f64,
    pub output: f64,
}

#[derive(Debug)]
pub struct Provider {
    pub name: &'static str,
    pub models: &'static [ModelPricing],
}

// Define OpenAI pricing
const OPENAI_MODELS: &[ModelPricing] = &[
    ModelPricing {
        name: "o1",
        input_cached: 7.50,
        input_normal: 15.00,
        output: 60.00,
    },
    ModelPricing {
        name: "o3-mini",
        input_cached: 0.55,
        input_normal: 1.10,
        output: 4.40,
    },
    ModelPricing {
        name: "gpt-4-5",
        input_cached: 37.50,
        input_normal: 75.00,
        output: 150.00,
    },
    ModelPricing {
        name: "gpt-4o",
        input_cached: 1.25,
        input_normal: 2.50,
        output: 10.00,
    },
    ModelPricing {
        name: "gpt-4o-mini",
        input_cached: 0.075,
        input_normal: 0.150,
        output: 0.600,
    },
];

const OPENAI: Provider = Provider {
    name: "openai",
    models: OPENAI_MODELS,
};

// Define Groq pricing
const GROQ_MODELS: &[ModelPricing] = &[
    ModelPricing {
        name: "deepseek-r1-distill-llama-70b",
        input_cached: 0.75,
        input_normal: 0.75,
        output: 0.99,
    },
    ModelPricing {
        name: "deepseek-r1-distill-qwen-32b-128k",
        input_cached: 0.69,
        input_normal: 0.69,
        output: 0.69,
    },
    ModelPricing {
        name: "qwen-2.5-32b-instruct-128k",
        input_cached: 0.79,
        input_normal: 0.79,
        output: 0.79,
    },
    ModelPricing {
        name: "qwen-2.5-coder-32b-instruct-128k",
        input_cached: 0.79,
        input_normal: 0.79,
        output: 0.79,
    },
    ModelPricing {
        name: "mistral-saba-24b",
        input_cached: 0.79,
        input_normal: 0.79,
        output: 0.79,
    },
    ModelPricing {
        name: "llama-3.2-1b-preview-8k",
        input_cached: 0.04,
        input_normal: 0.04,
        output: 0.04,
    },
    ModelPricing {
        name: "llama-3.2-3b-preview-8k",
        input_cached: 0.06,
        input_normal: 0.06,
        output: 0.06,
    },
    ModelPricing {
        name: "llama-3.3-70b-versatile-128k",
        input_cached: 0.59,
        input_normal: 0.59,
        output: 0.79,
    },
    ModelPricing {
        name: "llama-3.1-8b-instant-128k",
        input_cached: 0.05,
        input_normal: 0.05,
        output: 0.08,
    },
    ModelPricing {
        name: "llama-3-70b-8k",
        input_cached: 0.59,
        input_normal: 0.59,
        output: 0.79,
    },
    ModelPricing {
        name: "llama-3-8b-8k",
        input_cached: 0.05,
        input_normal: 0.05,
        output: 0.08,
    },
    ModelPricing {
        name: "mixtral-8x7b-instruct-32k",
        input_cached: 0.24,
        input_normal: 0.24,
        output: 0.24,
    },
    ModelPricing {
        name: "gemma-2.9b-8k",
        input_cached: 0.20,
        input_normal: 0.20,
        output: 0.20,
    },
    ModelPricing {
        name: "llama-guard-3-8b-8k",
        input_cached: 0.20,
        input_normal: 0.20,
        output: 0.20,
    },
    ModelPricing {
        name: "llama-3.3-70b-specdec-8k",
        input_cached: 0.59,
        input_normal: 0.59,
        output: 0.99,
    },
];

const GROQ: Provider = Provider {
    name: "groq",
    models: GROQ_MODELS,
};

// Define Gemini pricing
const GEMINI_MODELS: &[ModelPricing] = &[
    ModelPricing {
        name: "gemini-2.0-flash",
        input_cached: 0.025,
        input_normal: 0.10,
        output: 0.40,
    },
    ModelPricing {
        name: "gemini-2.0-flash-lite",
        input_cached: 0.0,
        input_normal: 0.075,
        output: 0.30,
    },
    ModelPricing {
        name: "imagen-3",
        input_cached: 0.0,
        input_normal: 0.03,
        output: 0.0,
    },
    ModelPricing {
        name: "gemini-1.5-flash",
        input_cached: 0.01875,
        input_normal: 0.075,
        output: 0.30,
    },
    ModelPricing {
        name: "gemini-1.5-flash",
        input_cached: 0.0375,
        input_normal: 0.15,
        output: 0.60,
    },
    ModelPricing {
        name: "gemini-1.5-flash-8b",
        input_cached: 0.01,
        input_normal: 0.0375,
        output: 0.15,
    },
    ModelPricing {
        name: "gemini-1.5-flash-8b",
        input_cached: 0.02,
        input_normal: 0.075,
        output: 0.30,
    },
    ModelPricing {
        name: "gemini-1.5-pro",
        input_cached: 0.3125,
        input_normal: 1.25,
        output: 5.00,
    },
    ModelPricing {
        name: "gemini-1.5-pro",
        input_cached: 0.625,
        input_normal: 2.50,
        output: 10.00,
    },
    ModelPricing {
        name: "text-embedding-004",
        input_cached: 0.0,
        input_normal: 0.0,
        output: 0.0,
    },
];

const GEMINI: Provider = Provider {
    name: "gemini",
    models: GEMINI_MODELS,
};

// Define Deepseek pricing
const DEEPSEEK_MODELS: &[ModelPricing] = &[
    ModelPricing {
        name: "deepseek-chat",
        input_cached: 0.07,
        input_normal: 0.27,
        output: 1.10,
    },
    ModelPricing {
        name: "deepseek-reasoner",
        input_cached: 0.14,
        input_normal: 0.55,
        output: 2.19,
    },
];

const DEEPSEEK: Provider = Provider {
    name: "deepseek",
    models: DEEPSEEK_MODELS,
};

// Define Anthropic pricing
const ANTHROPIC_MODELS: &[ModelPricing] = &[
  ModelPricing {
    name: "claude-3-7-sonnet",
    input_cached: 0.30,
    input_normal: 3.00,
    output: 15.00,
  },
  ModelPricing {
    name: "claude-3-5-haiku",
    input_cached: 0.08,
    input_normal: 0.80,
    output: 4.00,
  },
  ModelPricing {
    name: "claude-3-opus",
    input_cached: 1.50,
    input_normal: 15.00,
    output: 75.00,
  },
    ModelPricing {
    name: "claude-3-5-sonnet",
    input_cached: 0.30,
    input_normal: 3.00,
    output: 15.00,
  },
    ModelPricing {
    name: "claude-3-haiku",
    input_cached: 0.03,
    input_normal: 0.25,
    output: 1.25,
  },
];

const ANTHROPIC: Provider = Provider {
  name: "anthropic",
  models: ANTHROPIC_MODELS,
};

pub const PROVIDERS: &[Provider] = &[OPENAI, GROQ, GEMINI, DEEPSEEK, ANTHROPIC];
