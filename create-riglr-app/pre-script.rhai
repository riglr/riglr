// Pre-generation script for create-riglr-app template
// This script runs before the template is processed

// Validate project name
if variable::get("project-name").len() < 3 {
    abort("Project name must be at least 3 characters long");
}

// Validate email format if provided
let email = variable::get("author-email");
if email != "user@example.com" && !email.contains("@") {
    abort("Please provide a valid email address");
}

// Set derived variables
variable::set("project_name_snake", variable::get("project-name").replace("-", "_"));
variable::set("project_name_title", variable::get("project-name").to_title());

// Validate chain selection
let primary_chain = variable::get("primary-chain");
if primary_chain != "solana" && primary_chain != "ethereum" && primary_chain != "both" {
    abort("Primary chain must be 'solana', 'ethereum', or 'both'");
}

// Validate agent type
let agent_type = variable::get("agent-type");
let valid_types = ["trading-bot", "market-analyst", "dex-arbitrage", "news-monitor", "portfolio-tracker", "custom"];
if !valid_types.contains(agent_type) {
    abort(`Agent type must be one of: ${valid_types}`);
}

print("✅ Template validation complete");
print(`📦 Generating project: ${variable::get("project-name")}`);
print(`🤖 Agent type: ${agent_type}`);
print(`⛓️  Primary chain: ${primary_chain}`);