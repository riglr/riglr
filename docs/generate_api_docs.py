#!/usr/bin/env python3
"""
Generate comprehensive API documentation for riglr crates using rustdoc JSON output.
This provides much more accurate and complete documentation extraction than AST parsing.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional, Any, Set
from collections import defaultdict, OrderedDict

# Featured items for key crates to highlight at the top of API documentation
FEATURED_ITEMS = {
    "riglr-core": ["ApplicationContext", "SignerContext", "ToolWorker", "Tool", "Job", "JobResult", "ToolError"],
    "riglr-agents": ["Agent", "AgentDispatcher", "LocalAgentRegistry", "RedisAgentRegistry", "ToolCallingAgent"],
    "riglr-streams": ["StreamManager", "Stream", "ComposableStream"],
    "riglr-indexer": ["IndexerService", "ProcessingPipeline", "DataStore"],
    "riglr-config": ["Config", "ConfigBuilder"],
    "riglr-macros": ["tool"]
}

def run_cargo_doc_json(crate_path: Path) -> Optional[Dict]:
    """
    Run cargo doc with JSON output for a crate.
    Returns the parsed JSON documentation.
    """
    print(f"  📝 Generating rustdoc JSON for {crate_path.name}...")
    
    # Special handling for riglr-indexer due to SQLX compilation issues
    # Use explicit features instead of --all-features to avoid recursion overflow
    if crate_path.name == "riglr-indexer":
        cmd = [
            "cargo", "+nightly", "rustdoc", 
            "--lib",
            "--features", "postgres,metrics",
            "--",
            "-Zunstable-options",
            "--output-format", "json"
        ]
        print(f"    Using explicit features for riglr-indexer to avoid SQLX issues")
    else:
        # rustdoc with --output-format json generates files in workspace target/doc
        cmd = [
            "cargo", "+nightly", "rustdoc", 
            "--lib",
            "--all-features",
            "--",
            "-Zunstable-options",
            "--output-format", "json"
        ]
    
    # Set environment to prevent database connections
    env = os.environ.copy()
    env["SQLX_OFFLINE"] = "true"
    
    try:
        # Use a reasonable timeout for all crates
        timeout_seconds = 180
        
        result = subprocess.run(
            cmd,
            cwd=crate_path,
            capture_output=True,
            text=True,
            env=env,
            timeout=timeout_seconds
        )
        
        if result.returncode != 0:
            print(f"  ⚠️  Failed to generate rustdoc JSON: {result.stderr[:200]}")
            return None
    except subprocess.TimeoutExpired:
        print(f"  ⚠️  Timeout generating rustdoc JSON for {crate_path.name}")
        return None
        
        # The JSON files are generated in the workspace root's target/doc directory
        # The workspace root is the parent directory of the crate
        workspace_root = crate_path.parent
        
        # Look for the JSON file in workspace target/doc
        doc_dir = workspace_root / "target" / "doc"
        crate_name = crate_path.name.replace("-", "_")
        json_file = doc_dir / f"{crate_name}.json"
        
        if not json_file.exists():
            # Try other possible locations
            json_files = list(doc_dir.glob("*.json")) if doc_dir.exists() else []
            
            if json_files:
                # Look for the crate's JSON file
                for f in json_files:
                    if f.stem == crate_name:
                        json_file = f
                        break
                
                if not json_file.exists() and json_files:
                    # Fallback to any JSON file if exact match not found
                    json_file = json_files[0]
        
        if not json_file.exists():
            print(f"  ⚠️  No JSON documentation file found at {json_file}")
            return None
        
        with open(json_file, 'r') as f:
            return json.load(f)
            
    except Exception as e:
        print(f"  ⚠️  Error generating rustdoc JSON: {e}")
        return None

def format_rust_type(type_info: Any) -> str:
    """Format a Rust type from rustdoc JSON representation."""
    if isinstance(type_info, str):
        return type_info
    elif isinstance(type_info, dict):
        if "resolved_path" in type_info:
            # Full path type
            path = type_info["resolved_path"]
            name = path.get("name", "")
            args = path.get("args", {})
            if args and "angle_bracketed" in args:
                angle_args = args["angle_bracketed"]
                if "args" in angle_args:
                    type_args = [format_rust_type(arg.get("type", {})) for arg in angle_args["args"] if "type" in arg]
                    if type_args:
                        return f"{name}<{', '.join(type_args)}>"
            return name
        elif "primitive" in type_info:
            return type_info["primitive"]
        elif "generic" in type_info:
            return type_info["generic"]
        elif "impl_trait" in type_info:
            bounds = type_info["impl_trait"]
            if bounds:
                formatted_bounds = []
                for bound in bounds:
                    if isinstance(bound, dict) and "trait_bound" in bound:
                        trait = bound["trait_bound"].get("trait", {})
                        formatted_bounds.append(format_rust_type(trait))
                if formatted_bounds:
                    return f"impl {' + '.join(formatted_bounds)}"
            return "impl ..."
        elif "borrowed_ref" in type_info:
            ref_info = type_info["borrowed_ref"]
            lifetime = ref_info.get("lifetime", "")
            mutable = ref_info.get("mutable", False)
            inner = format_rust_type(ref_info.get("type", {}))
            
            if lifetime:
                return f"&'{lifetime} {'mut ' if mutable else ''}{inner}"
            else:
                return f"&{'mut ' if mutable else ''}{inner}"
        elif "slice" in type_info:
            inner = format_rust_type(type_info["slice"])
            return f"[{inner}]"
        elif "array" in type_info:
            arr = type_info["array"]
            inner = format_rust_type(arr.get("type", {}))
            len_val = arr.get("len", "")
            return f"[{inner}; {len_val}]"
        elif "tuple" in type_info:
            elements = type_info["tuple"]
            if elements:
                formatted = [format_rust_type(elem) for elem in elements]
                return f"({', '.join(formatted)})"
            return "()"
        else:
            # Try to extract any useful information
            if "trait_bound" in type_info:
                trait = type_info["trait_bound"].get("trait", {})
                return format_rust_type(trait)
            return "_"
    return "_"

def extract_docs_from_json(doc_json: Dict, crate_name: str) -> Dict[str, List[Dict]]:
    """
    Extract documentation items from rustdoc JSON.
    Returns items organized by category.
    """
    # Use OrderedDict to maintain consistent category order
    items = OrderedDict([
        ("Tools", []),
        ("Structs", []),
        ("Enums", []),
        ("Traits", []),
        ("Functions", []),
        ("Type Aliases", []),
        ("Constants", []),
        ("Attribute Macros", []),
        ("Derive Macros", []),
        ("Procedural Macros", [])
    ])
    
    # Track added items to prevent duplicates
    added_items = defaultdict(set)
    
    index = doc_json.get("index", {})
    paths = doc_json.get("paths", {})
    
    # First, find what items are actually exported/public via paths
    # Filter to only items from this crate (crate_id == 0)
    public_item_ids = set()
    for item_id, path_info in paths.items():
        # Only include items from this crate
        if path_info.get("crate_id") != 0:
            continue
            
        # Include all kinds of items that should be documented
        kind = path_info.get("kind")
        if kind in ["struct", "enum", "trait", "function", "module", "type_alias", "constant", 
                    "proc_attribute", "proc_derive", "proc_macro", "variant"]:
            public_item_ids.add(item_id)
    
    # Process public items from paths
    for item_id in public_item_ids:
        item_data = index.get(item_id, {})
        if not item_data:
            continue
        
        name = item_data.get("name", "")
        if not name or name.startswith("_"):
            continue
        
        # Skip if we've already processed this item in any category
        already_added = False
        for category_set in added_items.values():
            if name in category_set:
                already_added = True
                break
        if already_added:
            continue
        
        # Skip module items (we don't need to list the module itself)
        path_info = paths.get(item_id, {})
        if path_info.get("kind") == "module" and len(path_info.get("path", [])) <= 1:
            continue
        
        # Extract documentation
        docs = item_data.get("docs", "")
        
        # Get the inner details
        inner = item_data.get("inner", {})
        if not inner:
            continue
        
        # Process based on type in inner
        if "struct" in inner:
            struct_inner = inner["struct"]
            struct_info = {
                "name": name,
                "docs": docs,
                "fields": [],
                "generics": struct_inner.get("generics", {}),
                "impls": struct_inner.get("impls", [])
            }
            
            # Extract fields
            kind_with_fields = struct_inner.get("kind_with_fields")
            if kind_with_fields:
                for field_id in kind_with_fields.get("fields", []):
                    # Convert ID to string since index keys are strings
                    field_data = index.get(str(field_id), {})
                    if field_data:
                        field_info = {
                            "name": field_data.get("name", ""),
                            "type": format_rust_type(field_data.get("inner", {}).get("struct_field", {}).get("type", {})),
                            "docs": field_data.get("docs", "")
                        }
                        struct_info["fields"].append(field_info)
            
            if name not in added_items["Structs"]:
                items["Structs"].append(struct_info)
                added_items["Structs"].add(name)
            
        elif "enum" in inner:
            enum_inner = inner["enum"]
            enum_info = {
                "name": name,
                "docs": docs,
                "variants": [],
                "generics": enum_inner.get("generics", {}),
                "impls": enum_inner.get("impls", [])
            }
            
            # Extract variants
            for variant_id in enum_inner.get("variants", []):
                # Convert ID to string since index keys are strings
                variant_data = index.get(str(variant_id), {})
                if variant_data:
                    variant_info = {
                        "name": variant_data.get("name", ""),
                        "docs": variant_data.get("docs", "")
                    }
                    enum_info["variants"].append(variant_info)
            
            if name not in added_items["Enums"]:
                items["Enums"].append(enum_info)
                added_items["Enums"].add(name)
            
        elif "trait" in inner:
            trait_inner = inner["trait"]
            trait_info = {
                "name": name,
                "docs": docs,
                "methods": [],
                "generics": trait_inner.get("generics", {}),
                "bounds": trait_inner.get("bounds", [])
            }
            
            # Extract trait items (methods)
            for item_id in trait_inner.get("items", []):
                # Convert ID to string since index keys are strings
                method_data = index.get(str(item_id), {})
                if method_data:
                    method_inner = method_data.get("inner", {})
                    if "function" in method_inner:
                        method_info = {
                            "name": method_data.get("name", ""),
                            "docs": method_data.get("docs", ""),
                            "decl": method_inner["function"].get("decl", {}),
                            "is_async": method_inner["function"].get("is_async", False)
                        }
                        trait_info["methods"].append(method_info)
            
            if name not in added_items["Traits"]:
                items["Traits"].append(trait_info)
                added_items["Traits"].add(name)
            
        elif "function" in inner:
            func_inner = inner["function"]
            func_info = {
                "name": name,
                "docs": docs,
                "decl": func_inner.get("decl", {}),
                "generics": func_inner.get("generics", {}),
                "is_async": func_inner.get("is_async", False)
            }
            
            # Check if it's a tool function (has #[tool] attribute)
            attrs = item_data.get("attrs", [])
            is_tool = any("tool" in str(attr) for attr in attrs)
            
            if is_tool:
                if name not in added_items["Tools"]:
                    items["Tools"].append(func_info)
                    added_items["Tools"].add(name)
            else:
                if name not in added_items["Functions"]:
                    items["Functions"].append(func_info)
                    added_items["Functions"].add(name)
                    
        elif "type_alias" in inner:
            type_info = {
                "name": name,
                "docs": docs,
                "type": format_rust_type(inner.get("type_alias", {}).get("type", {}))
            }
            if name not in added_items["Type Aliases"]:
                items["Type Aliases"].append(type_info)
                added_items["Type Aliases"].add(name)
            
        elif "constant" in inner:
            const_info = {
                "name": name,
                "docs": docs,
                "type": format_rust_type(inner.get("constant", {}).get("type", {})),
                "value": inner.get("constant", {}).get("value", "")
            }
            if name not in added_items["Constants"]:
                items["Constants"].append(const_info)
                added_items["Constants"].add(name)
            
        elif "proc_macro" in inner:
            # Handle procedural macros (attribute macros, derive macros, etc.)
            proc_macro_inner = inner["proc_macro"]
            
            # Extract usage information from the docs
            usage_example = ""
            if docs:
                # Look for code examples in the docs
                import re
                code_blocks = re.findall(r'```rust?\n(.*?)\n```', docs, re.DOTALL)
                if code_blocks:
                    usage_example = code_blocks[0]
            
            # Extract macro signature if available
            signature = ""
            if crate_name == "riglr-macros" and name == "tool":
                # Special handling for the #[tool] macro
                signature = "#[tool]"
                if not usage_example:
                    usage_example = "#[tool]\nasync fn my_tool(context: &ApplicationContext) -> Result<String, ToolError> { ... }"
            
            macro_info = {
                "name": name,
                "docs": docs,
                "kind": path_info.get("kind", "proc_macro"),
                "signature": signature,
                "usage_example": usage_example,
                "helpers": proc_macro_inner.get("helpers", [])
            }
            
            # Determine the category based on the kind
            category = ""
            if path_info.get("kind") == "proc_attribute":
                category = "Attribute Macros"
            elif path_info.get("kind") == "proc_derive":
                category = "Derive Macros"
            else:
                category = "Procedural Macros"
            
            if name not in added_items[category]:
                items[category].append(macro_info)
                added_items[category].add(name)
    
    # Remove empty categories
    return {k: v for k, v in items.items() if v}

def generate_markdown_from_json(crate_name: str, items: Dict[str, List[Dict]], readme_content: str) -> str:
    """Generate comprehensive markdown documentation from extracted items."""
    lines = []
    
    # Header
    lines.append(f"# {crate_name}")
    lines.append("")
    
    # Use mdBook include directive instead of copying README content
    # This includes the crate's README directly, avoiding duplication
    lines.append(f"{{{{#include ../../../{crate_name}/README.md}}}}")
    lines.append("")
    
    if not items:
        lines.append("> **Note**: No public API items were found in this crate.")
        lines.append("")
        return '\n'.join(lines)
    
    # Generate table of contents for the API reference
    lines.append("## API Reference")
    lines.append("")
    
    # Check if this crate has featured items
    featured_item_names = FEATURED_ITEMS.get(crate_name, [])
    rendered_items = set()  # Track items we've already rendered
    
    # Add Key Components section if there are featured items
    if featured_item_names:
        lines.append("## Key Components")
        lines.append("")
        lines.append("> The most important types and functions in this crate.")
        lines.append("")
        
        # Find and render featured items
        for featured_name in featured_item_names:
            # Search for this item in all categories
            for category, category_items in items.items():
                for item in category_items:
                    if item.get("name") == featured_name:
                        # Render this featured item
                        name = item.get("name", "Unknown")
                        docs = item.get("docs", "")
                        
                        lines.append(f"### `{name}`")
                        lines.append("")
                        
                        # Add documentation
                        if docs:
                            # Take first paragraph of docs for featured items
                            doc_lines = docs.strip().split('\n\n')[0].split('\n')
                            for doc_line in doc_lines:
                                lines.append(doc_line)
                            lines.append("")
                        
                        # Add link to full documentation
                        anchor = category.lower().replace(" ", "-")
                        lines.append(f"[→ Full documentation](#{anchor})")
                        lines.append("")
                        
                        # Mark as rendered
                        rendered_items.add(featured_name)
                        break
        
        lines.append("---")
        lines.append("")
    
    # Add mini table of contents
    lines.append("### Contents")
    lines.append("")
    for category in items.keys():
        anchor = category.lower().replace(" ", "-")
        lines.append(f"- [{category}](#{anchor})")
    lines.append("")
    
    # Generate documentation for each category
    for category, category_items in items.items():
        if not category_items:
            continue
        
        lines.append(f"### {category}")
        lines.append("")
        
        # Add a brief description for each category
        if category == "Tools":
            lines.append("> Functions marked with `#[tool]` that can be used by riglr agents for blockchain operations.")
            lines.append("")
        elif category == "Structs":
            lines.append("> Core data structures and types.")
            lines.append("")
        elif category == "Enums":
            lines.append("> Enumeration types for representing variants.")
            lines.append("")
        elif category == "Traits":
            lines.append("> Trait definitions for implementing common behaviors.")
            lines.append("")
        elif category == "Functions":
            lines.append("> Standalone functions and utilities.")
            lines.append("")
        
        for item in sorted(category_items, key=lambda x: x.get("name", "")):
            name = item.get("name", "Unknown")
            
            # Skip if this item was already rendered in Key Components
            if name in rendered_items:
                continue
                
            docs = item.get("docs", "")
            
            lines.append(f"#### `{name}`")
            lines.append("")
            
            # Add documentation
            if docs:
                # Extract the first paragraph as a summary
                paragraphs = docs.strip().split('\n\n')
                if paragraphs:
                    # Add the first paragraph as a summary
                    first_paragraph = paragraphs[0]
                    lines.append(first_paragraph)
                    lines.append("")
                    
                    # If there are more paragraphs, add them too
                    if len(paragraphs) > 1:
                        for paragraph in paragraphs[1:]:
                            lines.append(paragraph)
                            lines.append("")
            
            # Add type-specific information
            if category == "Structs":
                # Add fields
                fields = item.get("fields", [])
                if fields:
                    lines.append("**Fields:**")
                    lines.append("")
                    for field in fields:
                        field_name = field.get("name", "")
                        field_type = field.get("type", "")
                        field_docs = field.get("docs", "")
                        
                        if field_name:  # Only show named fields
                            lines.append(f"- `{field_name}`: `{field_type}`")
                            if field_docs:
                                # Clean up docs - just take first line if multi-line
                                doc_lines = field_docs.strip().split('\n')
                                lines.append(f"  - {doc_lines[0]}")
                    lines.append("")
                    
            elif category == "Enums":
                # Add variants
                variants = item.get("variants", [])
                if variants:
                    lines.append("**Variants:**")
                    lines.append("")
                    for variant in variants:
                        variant_name = variant.get("name", "")
                        variant_docs = variant.get("docs", "")
                        
                        lines.append(f"- `{variant_name}`")
                        if variant_docs:
                            doc_first_line = variant_docs.strip().split('\n')[0]
                            lines.append(f"  - {doc_first_line}")
                    lines.append("")
                    
            elif category == "Traits":
                # Add methods
                methods = item.get("methods", [])
                if methods:
                    lines.append("**Methods:**")
                    lines.append("")
                    for method in methods:
                        method_name = method.get("name", "")
                        method_docs = method.get("docs", "")
                        is_async = method.get("is_async", False)
                        
                        async_prefix = "async " if is_async else ""
                        lines.append(f"- `{async_prefix}{method_name}()`")
                        if method_docs:
                            doc_first_line = method_docs.strip().split('\n')[0]
                            lines.append(f"  - {doc_first_line}")
                    lines.append("")
                    
            elif category in ["Functions", "Tools"]:
                # Add function signature details
                decl = item.get("decl", {})
                is_async = item.get("is_async", False)
                
                if is_async:
                    lines.append("**Async:** Yes")
                    lines.append("")
                
                # Add parameters
                inputs = decl.get("inputs", [])
                if inputs:
                    lines.append("**Parameters:**")
                    lines.append("")
                    for inp in inputs:
                        param_name = inp.get("name", "_")
                        param_type = format_rust_type(inp.get("type", {}))
                        lines.append(f"- `{param_name}`: `{param_type}`")
                    lines.append("")
                
                # Add return type
                output = decl.get("output")
                if output and output != {"unit": {}}:
                    lines.append(f"**Returns:** `{format_rust_type(output)}`")
                    lines.append("")
            
            elif category == "Type Aliases":
                # Add the aliased type
                alias_type = item.get("type", "_")
                lines.append(f"**Type:** `{alias_type}`")
                lines.append("")
            
            elif category == "Constants":
                # Add the constant type and value
                const_type = item.get("type", "_")
                const_value = item.get("value", "")
                lines.append(f"**Type:** `{const_type}`")
                if const_value:
                    lines.append(f"**Value:** `{const_value}`")
                lines.append("")
            
            elif category in ["Attribute Macros", "Derive Macros", "Procedural Macros"]:
                # Add macro-specific information
                kind = item.get("kind", "")
                signature = item.get("signature", "")
                usage_example = item.get("usage_example", "")
                
                if kind == "proc_attribute":
                    lines.append("**Type:** Attribute Macro")
                    lines.append("")
                    if signature:
                        lines.append(f"**Signature:** `{signature}`")
                        lines.append("")
                    lines.append("Use this macro as an attribute on functions or types.")
                elif kind == "proc_derive":
                    lines.append("**Type:** Derive Macro")
                    lines.append("")
                    lines.append("Use this macro in a `#[derive(...)]` attribute.")
                else:
                    lines.append("**Type:** Procedural Macro")
                    if signature:
                        lines.append(f"**Signature:** `{signature}`")
                
                if usage_example:
                    lines.append("")
                    lines.append("**Usage Example:**")
                    lines.append("")
                    lines.append("```rust")
                    lines.append(usage_example)
                    lines.append("```")
                
                lines.append("")
            
            lines.append("---")
            lines.append("")
    
    return '\n'.join(lines)

def generate_comprehensive_docs(crate_path: Path, force_regenerate: bool = False) -> Optional[str]:
    """
    Generate comprehensive documentation for a crate.
    Tries to generate fresh rustdoc JSON, falls back to existing if available.
    """
    crate_name = crate_path.name
    
    # Always try to generate fresh JSON
    workspace_root = crate_path.parent
    doc_dir = workspace_root / "target" / "doc"
    crate_name_underscore = crate_name.replace("-", "_")
    json_file = doc_dir / f"{crate_name_underscore}.json"
    
    doc_json = None
    
    # Check if JSON file exists and is recent (modified within last 5 minutes)
    should_regenerate = force_regenerate or not json_file.exists()
    if json_file.exists() and not force_regenerate:
        # Check if file is older than 5 minutes
        import time
        file_age = time.time() - json_file.stat().st_mtime
        if file_age > 300:  # If older than 5 minutes
            should_regenerate = True
    
    if should_regenerate:
        # Generate fresh JSON
        doc_json = run_cargo_doc_json(crate_path)
        
        # If generation failed but we have an existing file, use it
        if doc_json is None and json_file.exists():
            print(f"  📖 Falling back to existing rustdoc JSON for {crate_name}...")
            try:
                with open(json_file, 'r') as f:
                    doc_json = json.load(f)
            except Exception as e:
                print(f"  ⚠️  Failed to read existing JSON: {e}")
    else:
        # Use existing JSON file
        print(f"  📖 Using recent rustdoc JSON for {crate_name}...")
        try:
            with open(json_file, 'r') as f:
                doc_json = json.load(f)
        except Exception as e:
            print(f"  ⚠️  Failed to read existing JSON: {e}")
            # Try to generate fresh JSON
            doc_json = run_cargo_doc_json(crate_path)
    
    if doc_json:
        items = extract_docs_from_json(doc_json, crate_name)
        return generate_markdown_from_json(crate_name, items, None)
    
    # If JSON generation failed completely
    print(f"  ⚠️  Falling back to AST parsing for {crate_name}")
    return None

def main():
    if len(sys.argv) != 3:
        print("Usage: generate_api_docs.py <repo_root> <output_dir>")
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
        "riglr-solana-events",
        "riglr-solana-tools",
        "riglr-streams",
        "riglr-web-adapters",
        "riglr-web-tools"
    ]
    
    print("🚀 Starting API documentation generation using rustdoc JSON...")
    print("")
    
    success_count = 0
    failed_crates = []
    
    for crate_name in crates:
        print(f"📦 Processing {crate_name}...")
        crate_path = repo_root / crate_name
        
        if not crate_path.exists():
            print(f"  ⚠️  Crate directory not found")
            failed_crates.append(crate_name)
            continue
        
        try:
            # Generate documentation (force regenerate if JSON is stale)
            markdown = generate_comprehensive_docs(crate_path, force_regenerate=False)
            
            if markdown:
                # Write to file
                output_file = output_dir / f"{crate_name}.md"
                output_file.write_text(markdown)
                print(f"  ✅ Generated {output_file.name}")
                success_count += 1
            else:
                print(f"  ⚠️  Failed to generate documentation")
                failed_crates.append(crate_name)
        except Exception as e:
            # Print error to stderr with crate name and exception details
            print(f"  ❌ Error generating docs for {crate_name}: {e}", file=sys.stderr)
            failed_crates.append(crate_name)
            # Continue to next crate
        
        print("")
    
    print(f"🎉 Documentation generation complete!")
    print(f"   ✅ Successfully generated: {success_count}/{len(crates)} crates")
    if failed_crates:
        print(f"   ⚠️  Failed crates: {', '.join(failed_crates)}")

if __name__ == "__main__":
    main()