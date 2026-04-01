// Plantastic LLM clients
// Anthropic Claude for proposal narrative generation

client<llm> ProposalWriter {
  provider anthropic
  retry_policy Exponential
  options {
    model "claude-sonnet-4-20250514"
    api_key env.ANTHROPIC_API_KEY
  }
}

client<llm> ProposalWriterFast {
  provider anthropic
  retry_policy Exponential
  options {
    model "claude-haiku-4-5-20251001"
    api_key env.ANTHROPIC_API_KEY
  }
}

// Fallback: try fast model first, then full model
client<llm> ProposalFallback {
  provider fallback
  options {
    strategy [ProposalWriterFast, ProposalWriter]
  }
}

retry_policy Exponential {
  max_retries 2
  strategy {
    type exponential_backoff
    delay_ms 300
    multiplier 1.5
    max_delay_ms 10000
  }
}
