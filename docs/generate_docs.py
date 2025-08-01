#!/usr/bin/env python3
"""
Generate tool documentation by scanning Rust source files.
This approach directly parses source code instead of relying on rustdoc JSON.
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Dict, Optional, Tuple

def extract_doc_comments(lines: List[str], start_idx: int) -> str:
    """Extract documentation comments above a function."""
    docs = []
    idx = start_idx - 1
    
    while idx >= 0:
        line = lines[idx].strip()
        if line.startswith('///'):
            # Extract doc comment content
            doc_line = line[3:].strip()
            docs.insert(0, doc_line)
        elif line.startswith('//'):
            # Regular comment, skip
            pass
        elif not line:
            # Empty line, continue
            pass
        else:
            # Not a comment, stop
            break
        idx -= 1
    
    return '\n'.join(docs)

def parse_function_signature(lines: List[str], start_idx: int) -> Tuple[str, str]:
    """Parse a function signature from source."""
    signature_lines = []
    idx = start_idx
    brace_count = 0
    paren_count = 0
    
    while idx < len(lines):
        line = lines[idx]
        signature_lines.append(line)
        
        # Count braces and parens
        for char in line:
            if char == '(':
                paren_count += 1
            elif char == ')':
                paren_count -= 1
            elif char == '{':
                brace_count += 1
                if paren_count == 0:
                    # Found opening brace after closing paren
                    break
        
        if brace_count > 0:
            break
            
        idx += 1
    
    # Clean up signature
    signature = ' '.join(signature_lines).strip()
    signature = re.sub(r'\s+', ' ', signature)  # Normalize whitespace
    signature = signature.split('{')[0].strip()  # Remove body
    
    # Extract function name
    match = re.search(r'fn\s+(\w+)', signature)
    name = match.group(1) if match else 'unknown'
    
    return name, signature

def find_tool_functions(file_path: Path) -> List[Dict]:
    """Find functions marked with #[tool] in a Rust file."""
    tools = []
    
    try:
        with open(file_path, 'r') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"  Warning: Could not read {file_path}: {e}")
        return tools
    
    for i, line in enumerate(lines):
        # Look for #[tool] attribute
        if '#[tool' in line or '#[riglr_macros::tool' in line:
            # Found a tool attribute, look for the function below
            for j in range(i + 1, min(i + 10, len(lines))):
                if 'pub fn' in lines[j] or 'pub async fn' in lines[j]:
                    name, signature = parse_function_signature(lines, j)
                    docs = extract_doc_comments(lines, i)
                    
                    tools.append({
                        'name': name,
                        'signature': signature,
                        'docs': docs,
                        'file': str(file_path.relative_to(file_path.parent.parent))
                    })
                    break
    
    return tools

def scan_crate_for_tools(crate_path: Path) -> List[Dict]:
    """Scan a crate directory for tool functions."""
    all_tools = []
    
    # Scan src directory
    src_dir = crate_path / 'src'
    if not src_dir.exists():
        return all_tools
    
    # Find all .rs files
    for rs_file in src_dir.rglob('*.rs'):
        tools = find_tool_functions(rs_file)
        all_tools.extend(tools)
    
    return all_tools

def generate_markdown(crate_name: str, tools: List[Dict]) -> str:
    """Generate markdown documentation for tools."""
    lines = []
    
    # Header
    lines.append(f"# {crate_name} Tool Reference")
    lines.append("")
    lines.append(f"This page contains documentation for tools provided by the `{crate_name}` crate.")
    lines.append("")
    
    if not tools:
        lines.append("> **Note**: No tools with `#[tool]` attribute were found in this crate.")
        lines.append("> ")
        lines.append("> If this crate should have tools, ensure they are marked with the `#[tool]` attribute.")
        lines.append("")
        return '\n'.join(lines)
    
    # Table of Contents
    lines.append("## Available Tools")
    lines.append("")
    for tool in tools:
        lines.append(f"- [`{tool['name']}`](#{tool['name'].lower()}) - {tool['file']}")
    lines.append("")
    
    # Tool Documentation
    lines.append("## Tool Functions")
    lines.append("")
    
    for tool in tools:
        lines.append(f"### {tool['name']}")
        lines.append("")
        
        lines.append(f"**Source**: `{tool['file']}`")
        lines.append("")
        
        if tool['signature']:
            lines.append("```rust")
            lines.append(tool['signature'])
            lines.append("```")
            lines.append("")
        
        if tool['docs']:
            lines.append("**Documentation:**")
            lines.append("")
            lines.append(tool['docs'])
            lines.append("")
        else:
            lines.append("*No documentation available for this tool.*")
            lines.append("")
        
        lines.append("---")
        lines.append("")
    
    # Footer
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("*This documentation was automatically generated from the source code.*")
    
    return '\n'.join(lines)

def main():
    if len(sys.argv) != 3:
        print("Usage: generate_docs.py <repo_root> <output_dir>")
        sys.exit(1)
    
    repo_root = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    
    if not repo_root.exists():
        print(f"Error: Repository root {repo_root} does not exist")
        sys.exit(1)
    
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # List of crates to document
    crates = [
        "riglr-agents",
        "riglr-auth",
        "riglr-config",
        "riglr-core",
        "riglr-cross-chain-tools",
        "riglr-events-core",
        "riglr-evm-common",
        "riglr-evm-tools",
        "riglr-graph-memory",
        "riglr-hyperliquid-tools",
        "riglr-indexer",
        "riglr-macros",
        "riglr-server",
        "riglr-showcase",
        "riglr-solana-common",
        "riglr-solana-events",
        "riglr-solana-tools",
        "riglr-streams",
        "riglr-web-adapters",
        "riglr-web-tools"
    ]
    
    for crate_name in crates:
        print(f"  Processing {crate_name}...")
        crate_path = repo_root / crate_name
        
        if not crate_path.exists():
            print(f"    ⚠️  Crate directory not found")
            continue
        
        # Scan for tools
        tools = scan_crate_for_tools(crate_path)
        print(f"    Found {len(tools)} tool(s)")
        
        # Generate markdown
        markdown = generate_markdown(crate_name, tools)
        
        # Write to file
        output_file = output_dir / f"{crate_name}.md"
        output_file.write_text(markdown)
        print(f"    ✓ Generated {output_file.name}")
    
    print("✅ Tool documentation generation complete!")

if __name__ == "__main__":
    main()
