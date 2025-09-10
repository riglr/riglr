#!/usr/bin/env python3
"""
Generate dependency graph documentation for riglr crates using cargo metadata.
This script automatically creates a Mermaid diagram and statistics for the documentation.
"""

import json
import subprocess
import sys
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Set, Tuple

def run_cargo_metadata(repo_root: Path) -> Dict:
    """Run cargo metadata to get dependency information."""
    try:
        result = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"],
            cwd=repo_root,
            capture_output=True,
            text=True,
            check=True
        )
        return json.loads(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Error running cargo metadata: {e}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error parsing cargo metadata JSON: {e}")
        sys.exit(1)

def extract_riglr_dependencies(metadata: Dict) -> Dict[str, List[str]]:
    """Extract dependencies between riglr crates."""
    dependencies = defaultdict(list)
    riglr_packages = {}
    
    # First, collect all riglr packages
    for package in metadata["packages"]:
        if package["name"].startswith("riglr-") or package["name"] == "create-riglr-app":
            riglr_packages[package["id"]] = package["name"]
    
    # Then extract dependencies
    for package in metadata["packages"]:
        if package["name"] in riglr_packages.values():
            pkg_name = package["name"]
            dependencies[pkg_name] = []
            
            for dep in package["dependencies"]:
                dep_name = dep["name"]
                if dep_name.startswith("riglr-") or dep_name == "create-riglr-app":
                    # Only include normal dependencies, not dev or build
                    dep_kind = dep.get("kind", None)
                    if dep_kind is None or (isinstance(dep_kind, str) and dep_kind not in ["dev", "build"]):
                        dependencies[pkg_name].append(dep_name)
            
            # Sort dependencies for consistency
            dependencies[pkg_name].sort()
    
    return dict(dependencies)

def categorize_crates(crate_name: str) -> str:
    """Categorize a crate into its architectural layer."""
    if crate_name == "riglr-core":
        return "Core"
    elif crate_name in ["riglr-config", "riglr-macros"]:
        return "Foundation"
    elif "common" in crate_name:
        return "Common"
    elif "events" in crate_name:
        return "Events"
    elif "tools" in crate_name or crate_name == "riglr-streams":
        return "Tools"
    elif crate_name in ["riglr-graph-memory", "riglr-web-adapters"]:
        return "Adapters"
    elif crate_name in ["riglr-auth", "riglr-server", "riglr-agents"]:
        return "Services"
    elif crate_name in ["riglr-showcase", "riglr-indexer", "create-riglr-app"]:
        return "Applications"
    else:
        return "Other"

def calculate_statistics(dependencies: Dict[str, List[str]]) -> Tuple[Dict, Dict]:
    """Calculate dependency statistics."""
    # Count how many times each crate is depended upon
    depended_upon = defaultdict(int)
    for crate, deps in dependencies.items():
        for dep in deps:
            depended_upon[dep] += 1
    
    # Sort crates by number of dependencies
    most_dependent = sorted(
        [(crate, len(deps)) for crate, deps in dependencies.items() if len(deps) > 0],
        key=lambda x: x[1],
        reverse=True
    )
    
    # Sort crates by how many depend on them
    most_depended = sorted(
        [(crate, count) for crate, count in depended_upon.items()],
        key=lambda x: x[1],
        reverse=True
    )
    
    return most_dependent, most_depended

def generate_mermaid_diagram(dependencies: Dict[str, List[str]]) -> str:
    """Generate a Mermaid diagram from dependencies."""
    lines = []
    lines.append("```mermaid")
    lines.append("graph TB")
    lines.append("    %% Core layer")
    
    # Group crates by layer
    layers = defaultdict(list)
    for crate in dependencies.keys():
        layer = categorize_crates(crate)
        layers[layer].append(crate)
    
    # Define subgraphs for each layer
    layer_order = ["Core", "Foundation", "Common", "Events", "Adapters", "Tools", "Services", "Applications"]
    
    for layer in layer_order:
        if layer in layers:
            lines.append(f'    subgraph "{layer} Layer"')
            for crate in sorted(layers[layer]):
                var_name = crate.replace("-", "_").replace("riglr_", "")
                lines.append(f'        {var_name}["{crate}"]')
            lines.append("    end")
            lines.append("    ")
    
    # Add dependencies
    lines.append("    %% Dependencies")
    for crate, deps in sorted(dependencies.items()):
        if deps:
            crate_var = crate.replace("-", "_").replace("riglr_", "")
            for dep in deps:
                dep_var = dep.replace("-", "_").replace("riglr_", "")
                lines.append(f"    {dep_var} --> {crate_var}")
    
    lines.append("    ")
    lines.append("    %% Styling")
    lines.append("    classDef coreStyle fill:#ff6b6b,stroke:#333,stroke-width:3px")
    lines.append("    classDef solanaStyle fill:#4ecdc4,stroke:#333,stroke-width:2px")
    lines.append("    classDef evmStyle fill:#45b7d1,stroke:#333,stroke-width:2px")
    lines.append("    classDef webStyle fill:#96ceb4,stroke:#333,stroke-width:2px")
    lines.append("    classDef appStyle fill:#ffeaa7,stroke:#333,stroke-width:2px")
    lines.append("    classDef otherStyle fill:#dfe6e9,stroke:#333,stroke-width:2px")
    lines.append("    ")
    lines.append("    class core coreStyle")
    
    # Apply styles based on crate names
    solana_crates = [c.replace("-", "_").replace("riglr_", "") for c in dependencies.keys() if "solana" in c]
    if solana_crates:
        lines.append(f"    class {','.join(solana_crates)} solanaStyle")
    
    evm_crates = [c.replace("-", "_").replace("riglr_", "") for c in dependencies.keys() if "evm" in c]
    if evm_crates:
        lines.append(f"    class {','.join(evm_crates)} evmStyle")
    
    web_crates = [c.replace("-", "_").replace("riglr_", "") for c in dependencies.keys() if "web" in c]
    if web_crates:
        lines.append(f"    class {','.join(web_crates)} webStyle")
    
    app_crates = [c.replace("-", "_").replace("riglr_", "") for c in dependencies.keys() if c in ["riglr-showcase", "riglr-indexer", "create-riglr-app"]]
    if app_crates:
        lines.append(f"    class {','.join(app_crates)} appStyle")
    
    lines.append("```")
    
    return "\n".join(lines)

def generate_dependency_table(dependencies: Dict[str, List[str]]) -> str:
    """Generate a dependency table."""
    lines = []
    lines.append("## Dependency Table")
    lines.append("")
    lines.append("| Crate | Direct Dependencies | Count | Layer |")
    lines.append("|-------|---------------------|-------|-------|")
    
    # Sort by number of dependencies
    sorted_crates = sorted(
        dependencies.items(),
        key=lambda x: (len(x[1]), x[0])
    )
    
    for crate, deps in sorted_crates:
        layer = categorize_crates(crate)
        if deps:
            deps_str = ", ".join(deps)
        else:
            deps_str = "None"
        count = len(deps)
        lines.append(f"| **{crate}** | {deps_str} | {count} | {layer} |")
    
    return "\n".join(lines)

def generate_markdown(dependencies: Dict[str, List[str]]) -> str:
    """Generate the complete markdown documentation."""
    lines = []
    
    # Header
    lines.append("# Dependency Graph")
    lines.append("")
    lines.append("*This file is auto-generated by `docs/generate_dependency_graph.py`*")
    lines.append("")
    
    # Dependency table
    lines.append(generate_dependency_table(dependencies))
    lines.append("")
    
    # Statistics
    most_dependent, most_depended = calculate_statistics(dependencies)
    
    total_crates = len(dependencies)
    total_deps = sum(len(deps) for deps in dependencies.values())
    avg_deps = total_deps / total_crates if total_crates > 0 else 0
    
    lines.append("## Dependency Statistics")
    lines.append("")
    lines.append(f"- **Total crates**: {total_crates}")
    lines.append(f"- **Total dependency relationships**: {total_deps}")
    lines.append(f"- **Average dependencies per crate**: {avg_deps:.1f}")
    lines.append("")
    
    if most_dependent:
        lines.append("### Most Dependent Crates (have most dependencies)")
        lines.append("")
        for i, (crate, count) in enumerate(most_dependent[:5], 1):
            lines.append(f"{i}. **{crate}**: {count} dependencies")
        lines.append("")
    
    if most_depended:
        lines.append("### Most Depended Upon (used by most crates)")
        lines.append("")
        for i, (crate, count) in enumerate(most_depended[:5], 1):
            lines.append(f"{i}. **{crate}**: used by {count} crates")
        lines.append("")
    
    # Mermaid diagram
    lines.append("## Dependency Graph (Mermaid)")
    lines.append("")
    lines.append(generate_mermaid_diagram(dependencies))
    lines.append("")
    
    # Architecture layers
    lines.append("## Architecture Layers")
    lines.append("")
    
    layers_desc = [
        ("Core", "The foundational crate that all others depend on"),
        ("Foundation", "Configuration management and procedural macros"),
        ("Common", "Shared utilities for specific blockchains"),
        ("Events", "Event processing and streaming"),
        ("Tools", "Blockchain-specific tools and integrations"),
        ("Adapters", "External service adapters"),
        ("Services", "Application services and middleware"),
        ("Applications", "End-user applications and examples")
    ]
    
    for i, (layer, desc) in enumerate(layers_desc, 1):
        lines.append(f"### {i}. **{layer} Layer**")
        lines.append("")
        lines.append(f"{desc}")
        lines.append("")
        
        # List crates in this layer
        layer_crates = [c for c in dependencies.keys() if categorize_crates(c) == layer]
        if layer_crates:
            for crate in sorted(layer_crates):
                deps_count = len(dependencies[crate])
                lines.append(f"- `{crate}`: {deps_count} direct dependencies")
            lines.append("")
    
    # Key insights
    lines.append("## Key Insights")
    lines.append("")
    
    if most_depended and most_depended[0][1] > 0:
        lines.append(f"1. **{most_depended[0][0]}** is the absolute foundation - {most_depended[0][1]} out of {total_crates} crates depend on it")
    
    if most_dependent and most_dependent[0][1] > 0:
        lines.append(f"2. **{most_dependent[0][0]}** serves as an integration point with {most_dependent[0][1]} direct dependencies")
    
    lines.append("3. The architecture follows a clear layered approach with increasing specialization")
    lines.append("4. Blockchain-specific tools (Solana, EVM) are well-separated but share common foundations")
    
    return "\n".join(lines)

def main():
    if len(sys.argv) != 3:
        print("Usage: generate_dependency_graph.py <repo_root> <output_file>")
        sys.exit(1)
    
    repo_root = Path(sys.argv[1])
    output_file = Path(sys.argv[2])
    
    if not repo_root.exists():
        print(f"Error: Repository root {repo_root} does not exist")
        sys.exit(1)
    
    print("🔍 Analyzing crate dependencies...")
    
    # Get cargo metadata
    metadata = run_cargo_metadata(repo_root)
    
    # Extract dependencies
    dependencies = extract_riglr_dependencies(metadata)
    
    print(f"📊 Found {len(dependencies)} riglr crates")
    
    # Generate full markdown report
    markdown = generate_markdown(dependencies)
    
    # Generate diagram-only file content
    diagram_only = generate_mermaid_diagram(dependencies)
    
    # Write to files
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    # File 1: Full report (existing output file)
    output_file.write_text(markdown)
    print(f"✅ Generated dependency graph: {output_file}")
    
    # File 2: Diagram only (new file)
    diagram_file = output_file.parent / "dependency-graph-diagram.md"
    diagram_file.write_text(diagram_only)
    print(f"✅ Generated diagram-only file: {diagram_file}")

if __name__ == "__main__":
    main()