// Post-generation script for create-riglr-app template
// This script runs after the template has been generated

let project_name = variable::get("project-name");
let agent_type = variable::get("agent-type");
let primary_chain = variable::get("primary-chain");
let include_web_tools = variable::get("include-web-tools");
let include_graph_memory = variable::get("include-graph-memory");

print("🎉 Successfully generated riglr project!");
print("");
print("📋 Project Summary:");
print(`   Name: ${project_name}`);
print(`   Type: ${agent_type}`);  
print(`   Chain: ${primary_chain}`);
print(`   Web Tools: ${if include_web_tools { "✅" } else { "❌" }}`);
print(`   Graph Memory: ${if include_graph_memory { "✅" } else { "❌" }}`);
print("");

print("🚀 Next Steps:");
print("1. Navigate to your project:");
print(`   cd ${project_name}`);
print("");
print("2. Configure your environment:");
print("   cp .env.example .env");
print("   # Edit .env with your API keys");
print("");

if include_graph_memory {
    print("3. Start required services:");
    print("   # Neo4j for graph memory:");
    print("   docker run -d --name neo4j -p 7474:7474 -p 7687:7687 -e NEO4J_AUTH=neo4j/password neo4j:latest");
    print("");
    print("   # Redis for job queue:");
    print("   docker run -d --name redis -p 6379:6379 redis:alpine");
    print("");
    print("4. Build and run your agent:");
} else {
    print("3. Start Redis for job queue:");
    print("   docker run -d --name redis -p 6379:6379 redis:alpine");
    print("");
    print("4. Build and run your agent:");
}

print("   cargo build");
print("   cargo run -- --interactive");
print("");

print("📚 Documentation & Resources:");
print("   • Project README: ./README.md");
print("   • riglr Documentation: https://docs.riglr.dev");
print("   • Examples: https://github.com/riglr-project/examples");
print("   • Discord Community: https://discord.gg/riglr");
print("");

print("⚠️  Important:");
print("   • Never commit your .env file with real API keys");
print("   • Start with testnet/devnet for development");
print("   • Review the security guidelines in the documentation");
print("");

print("Happy building! 🔨✨");