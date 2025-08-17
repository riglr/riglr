#!/usr/bin/env python3
"""
Generate comprehensive documentation for all riglr crates.
Extracts and documents all public APIs including structs, traits, enums, functions, and more.
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Dict, Optional, Tuple, Set
from dataclasses import dataclass, field
from collections import defaultdict

@dataclass
class DocItem:
    """Represents a documented item from the source code."""
    name: str
    kind: str  # 'struct', 'trait', 'enum', 'function', 'impl', 'type', 'const'
    signature: str
    docs: str
    file: str
    visibility: str = 'public'
    attributes: List[str] = field(default_factory=list)
    methods: List['DocItem'] = field(default_factory=list)
    fields: List[Dict[str, str]] = field(default_factory=list)
    variants: List[Dict[str, str]] = field(default_factory=list)

class RustDocParser:
    """Parser for extracting documentation from Rust source files."""
    
    def __init__(self):
        self.items = []
        
    def extract_doc_comments(self, lines: List[str], start_idx: int) -> str:
        """Extract documentation comments above an item."""
        docs = []
        idx = start_idx - 1
        
        while idx >= 0:
            line = lines[idx].strip()
            if line.startswith('///'):
                doc_line = line[3:].strip() if len(line) > 3 else ''
                docs.insert(0, doc_line)
            elif line.startswith('//!'):
                # Module-level docs
                doc_line = line[3:].strip() if len(line) > 3 else ''
                docs.insert(0, doc_line)
            elif line.startswith('#['):
                # Attribute, might be relevant
                pass
            elif not line or line.startswith('//'):
                # Empty line or regular comment
                pass
            else:
                # Not a doc comment, stop
                break
            idx -= 1
        
        return '\n'.join(docs)
    
    def extract_attributes(self, lines: List[str], start_idx: int) -> List[str]:
        """Extract attributes like #[derive(...)] above an item."""
        attrs = []
        idx = start_idx - 1
        
        while idx >= 0:
            line = lines[idx].strip()
            if line.startswith('#['):
                attrs.insert(0, line)
            elif line.startswith('///') or line.startswith('//'):
                # Skip comments
                pass
            elif not line:
                # Empty line, continue
                pass
            else:
                # Not an attribute, stop
                break
            idx -= 1
        
        return attrs
    
    def parse_struct(self, lines: List[str], start_idx: int) -> Optional[DocItem]:
        """Parse a struct definition."""
        line = lines[start_idx]
        
        # Extract struct name
        match = re.search(r'struct\s+(\w+)', line)
        if not match:
            return None
            
        name = match.group(1)
        docs = self.extract_doc_comments(lines, start_idx)
        attrs = self.extract_attributes(lines, start_idx)
        
        # Determine if it's a tuple struct, unit struct, or regular struct
        fields = []
        signature_lines = []
        idx = start_idx
        brace_count = 0
        paren_count = 0
        
        while idx < len(lines):
            curr_line = lines[idx]
            signature_lines.append(curr_line.strip())
            
            for char in curr_line:
                if char == '{':
                    brace_count += 1
                elif char == '}':
                    brace_count -= 1
                elif char == '(':
                    paren_count += 1
                elif char == ')':
                    paren_count -= 1
            
            # Extract field information if it's a regular struct
            if '{' in curr_line and idx > start_idx:
                # Parse fields
                field_match = re.match(r'\s*(?:pub\s+)?(\w+):\s+(.+?)(?:,|$)', curr_line.strip())
                if field_match:
                    fields.append({
                        'name': field_match.group(1),
                        'type': field_match.group(2).strip(',')
                    })
            
            if (brace_count == 0 and '{' in ''.join(signature_lines)) or \
               (paren_count == 0 and '(' in ''.join(signature_lines) and ')' in ''.join(signature_lines)) or \
               (';' in curr_line and brace_count == 0):
                break
                
            idx += 1
        
        signature = ' '.join(signature_lines)
        signature = re.sub(r'\s+', ' ', signature)
        
        return DocItem(
            name=name,
            kind='struct',
            signature=signature,
            docs=docs,
            file='',
            attributes=attrs,
            fields=fields
        )
    
    def parse_enum(self, lines: List[str], start_idx: int) -> Optional[DocItem]:
        """Parse an enum definition."""
        line = lines[start_idx]
        
        match = re.search(r'enum\s+(\w+)', line)
        if not match:
            return None
            
        name = match.group(1)
        docs = self.extract_doc_comments(lines, start_idx)
        attrs = self.extract_attributes(lines, start_idx)
        
        # Parse enum variants
        variants = []
        signature_lines = []
        idx = start_idx
        brace_count = 0
        
        while idx < len(lines):
            curr_line = lines[idx]
            signature_lines.append(curr_line.strip())
            
            for char in curr_line:
                if char == '{':
                    brace_count += 1
                elif char == '}':
                    brace_count -= 1
            
            # Extract variant information
            if brace_count > 0 and idx > start_idx:
                variant_match = re.match(r'\s*(\w+)(?:\(([^)]*)\))?(?:\s*\{[^}]*\})?', curr_line.strip())
                if variant_match and not curr_line.strip().startswith('//'):
                    variant_name = variant_match.group(1)
                    variant_data = variant_match.group(2) or ''
                    if variant_name and variant_name not in ['pub', 'impl', 'fn']:
                        variants.append({
                            'name': variant_name,
                            'data': variant_data
                        })
            
            if brace_count == 0 and '{' in ''.join(signature_lines):
                break
                
            idx += 1
        
        signature = ' '.join(signature_lines)
        signature = re.sub(r'\s+', ' ', signature)
        
        return DocItem(
            name=name,
            kind='enum',
            signature=signature,
            docs=docs,
            file='',
            attributes=attrs,
            variants=variants
        )
    
    def parse_trait(self, lines: List[str], start_idx: int) -> Optional[DocItem]:
        """Parse a trait definition."""
        line = lines[start_idx]
        
        match = re.search(r'trait\s+(\w+)', line)
        if not match:
            return None
            
        name = match.group(1)
        docs = self.extract_doc_comments(lines, start_idx)
        attrs = self.extract_attributes(lines, start_idx)
        
        # Parse trait methods
        methods = []
        signature_lines = []
        idx = start_idx
        brace_count = 0
        
        while idx < len(lines):
            curr_line = lines[idx]
            signature_lines.append(curr_line.strip())
            
            for char in curr_line:
                if char == '{':
                    brace_count += 1
                elif char == '}':
                    brace_count -= 1
            
            # Look for method signatures
            if brace_count > 0 and 'fn ' in curr_line:
                method_match = re.search(r'fn\s+(\w+)', curr_line)
                if method_match:
                    method_name = method_match.group(1)
                    # Find the end of this method signature
                    method_sig = curr_line.strip()
                    if ';' not in method_sig and '{' not in method_sig:
                        # Multi-line signature
                        j = idx + 1
                        while j < len(lines) and ';' not in lines[j] and '{' not in lines[j]:
                            method_sig += ' ' + lines[j].strip()
                            j += 1
                        if j < len(lines):
                            method_sig += ' ' + lines[j].strip().split(';')[0] + ';'
                    
                    methods.append(DocItem(
                        name=method_name,
                        kind='method',
                        signature=method_sig,
                        docs='',
                        file=''
                    ))
            
            if brace_count == 0 and '{' in ''.join(signature_lines):
                break
                
            idx += 1
        
        signature = ' '.join(signature_lines[:3])  # Just the trait declaration
        signature = re.sub(r'\s+', ' ', signature)
        if '{' in signature:
            signature = signature.split('{')[0].strip() + ' { ... }'
        
        return DocItem(
            name=name,
            kind='trait',
            signature=signature,
            docs=docs,
            file='',
            attributes=attrs,
            methods=methods
        )
    
    def parse_function(self, lines: List[str], start_idx: int) -> Optional[DocItem]:
        """Parse a function definition."""
        line = lines[start_idx]
        
        # Check if it's a function
        if 'fn ' not in line:
            return None
        
        match = re.search(r'fn\s+(\w+)', line)
        if not match:
            return None
            
        name = match.group(1)
        docs = self.extract_doc_comments(lines, start_idx)
        attrs = self.extract_attributes(lines, start_idx)
        
        # Build complete signature
        signature_lines = []
        idx = start_idx
        brace_count = 0
        paren_count = 0
        
        while idx < len(lines):
            curr_line = lines[idx]
            signature_lines.append(curr_line.strip())
            
            for char in curr_line:
                if char == '(':
                    paren_count += 1
                elif char == ')':
                    paren_count -= 1
                elif char == '{':
                    brace_count += 1
                    if paren_count == 0:
                        break
            
            if brace_count > 0 or (';' in curr_line and paren_count == 0):
                break
                
            idx += 1
        
        signature = ' '.join(signature_lines)
        signature = re.sub(r'\s+', ' ', signature)
        signature = signature.split('{')[0].strip()
        
        # Check if it's a tool function
        is_tool = any('#[tool' in attr or '#[riglr_macros::tool' in attr for attr in attrs)
        
        return DocItem(
            name=name,
            kind='tool' if is_tool else 'function',
            signature=signature,
            docs=docs,
            file='',
            attributes=attrs
        )
    
    def parse_impl_block(self, lines: List[str], start_idx: int) -> Optional[DocItem]:
        """Parse an impl block."""
        line = lines[start_idx]
        
        # Extract what's being implemented
        impl_match = re.match(r'impl(?:<[^>]+>)?\s+(?:(\w+)\s+for\s+)?(\w+)', line)
        if not impl_match:
            return None
        
        trait_name = impl_match.group(1)
        struct_name = impl_match.group(2)
        
        if trait_name:
            name = f"{trait_name} for {struct_name}"
        else:
            name = struct_name
        
        docs = self.extract_doc_comments(lines, start_idx)
        
        # Parse methods in the impl block
        methods = []
        idx = start_idx + 1
        brace_count = 1
        
        while idx < len(lines) and brace_count > 0:
            curr_line = lines[idx]
            
            for char in curr_line:
                if char == '{':
                    brace_count += 1
                elif char == '}':
                    brace_count -= 1
            
            if 'pub fn' in curr_line or 'pub async fn' in curr_line:
                method = self.parse_function(lines, idx)
                if method:
                    methods.append(method)
            
            idx += 1
        
        if not methods:
            return None
        
        return DocItem(
            name=name,
            kind='impl',
            signature=f"impl {name}",
            docs=docs,
            file='',
            methods=methods
        )
    
    def parse_type_alias(self, lines: List[str], start_idx: int) -> Optional[DocItem]:
        """Parse a type alias."""
        line = lines[start_idx]
        
        match = re.search(r'type\s+(\w+)\s*=\s*(.+?);', line)
        if not match:
            return None
        
        name = match.group(1)
        alias = match.group(2)
        docs = self.extract_doc_comments(lines, start_idx)
        
        return DocItem(
            name=name,
            kind='type',
            signature=f"type {name} = {alias}",
            docs=docs,
            file=''
        )
    
    def parse_const(self, lines: List[str], start_idx: int) -> Optional[DocItem]:
        """Parse a constant."""
        line = lines[start_idx]
        
        match = re.search(r'const\s+(\w+):\s*([^=]+)\s*=', line)
        if not match:
            return None
        
        name = match.group(1)
        const_type = match.group(2).strip()
        docs = self.extract_doc_comments(lines, start_idx)
        
        return DocItem(
            name=name,
            kind='const',
            signature=f"const {name}: {const_type}",
            docs=docs,
            file=''
        )
    
    def parse_file(self, file_path: Path) -> List[DocItem]:
        """Parse a Rust source file and extract all documented items."""
        items = []
        
        try:
            with open(file_path, 'r') as f:
                lines = f.readlines()
        except Exception as e:
            print(f"  Warning: Could not read {file_path}: {e}")
            return items
        
        for i, line in enumerate(lines):
            stripped = line.strip()
            
            # Skip private items unless they have special attributes
            if not stripped.startswith('pub') and not any(attr in line for attr in ['#[tool', '#[derive']):
                # Check if the previous line has a tool attribute
                if i > 0 and '#[tool' not in lines[i-1] and '#[riglr_macros::tool' not in lines[i-1]:
                    continue
            
            # Parse different item types
            item = None
            
            if 'pub struct' in line or (i > 0 and '#[derive' in lines[i-1] and 'struct ' in line):
                item = self.parse_struct(lines, i)
            elif 'pub enum' in line or (i > 0 and '#[derive' in lines[i-1] and 'enum ' in line):
                item = self.parse_enum(lines, i)
            elif 'pub trait' in line:
                item = self.parse_trait(lines, i)
            elif ('pub fn' in line or 'pub async fn' in line or 
                  (i > 0 and ('#[tool' in lines[i-1] or '#[riglr_macros::tool' in lines[i-1]) and 'fn ' in line)):
                item = self.parse_function(lines, i)
            elif 'impl ' in stripped and not stripped.startswith('//'):
                item = self.parse_impl_block(lines, i)
            elif 'pub type ' in line:
                item = self.parse_type_alias(lines, i)
            elif 'pub const ' in line:
                item = self.parse_const(lines, i)
            
            if item:
                item.file = str(file_path.relative_to(file_path.parent.parent))
                items.append(item)
        
        return items

def organize_items_by_category(items: List[DocItem]) -> Dict[str, List[DocItem]]:
    """Organize items by their category."""
    categories = defaultdict(list)
    
    for item in items:
        if item.kind == 'tool':
            categories['Tools'].append(item)
        elif item.kind == 'struct':
            categories['Structs'].append(item)
        elif item.kind == 'enum':
            categories['Enums'].append(item)
        elif item.kind == 'trait':
            categories['Traits'].append(item)
        elif item.kind == 'function':
            # Group functions by their file/module
            module_name = item.file.split('/')[-1].replace('.rs', '') if item.file else 'unknown'
            categories[f'Functions ({module_name})'].append(item)
        elif item.kind == 'impl':
            categories['Implementations'].append(item)
        elif item.kind == 'type':
            categories['Type Aliases'].append(item)
        elif item.kind == 'const':
            categories['Constants'].append(item)
    
    return dict(categories)

def generate_markdown(crate_name: str, items: List[DocItem]) -> str:
    """Generate comprehensive markdown documentation for a crate."""
    lines = []
    
    # Header
    lines.append(f"# {crate_name} API Reference")
    lines.append("")
    lines.append(f"Comprehensive API documentation for the `{crate_name}` crate.")
    lines.append("")
    
    if not items:
        lines.append("> **Note**: No public API items were found in this crate.")
        lines.append("")
        return '\n'.join(lines)
    
    # Organize items by category
    categories = organize_items_by_category(items)
    
    # Generate table of contents
    lines.append("## Table of Contents")
    lines.append("")
    
    for category, category_items in categories.items():
        if category_items:
            lines.append(f"### {category}")
            lines.append("")
            for item in sorted(category_items, key=lambda x: x.name):
                anchor = f"{item.name.lower().replace(' ', '-').replace(':', '')}"
                lines.append(f"- [`{item.name}`](#{anchor})")
            lines.append("")
    
    # Generate documentation for each category
    for category, category_items in categories.items():
        if not category_items:
            continue
        
        lines.append(f"## {category}")
        lines.append("")
        
        for item in sorted(category_items, key=lambda x: x.name):
            anchor = f"{item.name.lower().replace(' ', '-').replace(':', '')}"
            lines.append(f"### {item.name}")
            lines.append("")
            
            # Source file
            if item.file:
                lines.append(f"**Source**: `{item.file}`")
                lines.append("")
            
            # Attributes
            if item.attributes:
                lines.append("**Attributes**:")
                lines.append("```rust")
                for attr in item.attributes:
                    lines.append(attr)
                lines.append("```")
                lines.append("")
            
            # Signature
            if item.signature:
                lines.append("```rust")
                lines.append(item.signature)
                lines.append("```")
                lines.append("")
            
            # Documentation
            if item.docs:
                lines.append(item.docs)
                lines.append("")
            
            # Fields (for structs)
            if item.fields:
                lines.append("**Fields**:")
                lines.append("")
                for field in item.fields:
                    lines.append(f"- `{field['name']}`: `{field['type']}`")
                lines.append("")
            
            # Variants (for enums)
            if item.variants:
                lines.append("**Variants**:")
                lines.append("")
                for variant in item.variants:
                    if variant.get('data'):
                        lines.append(f"- `{variant['name']}({variant['data']})`")
                    else:
                        lines.append(f"- `{variant['name']}`")
                lines.append("")
            
            # Methods (for traits and impls)
            if item.methods:
                lines.append("**Methods**:")
                lines.append("")
                for method in item.methods:
                    lines.append(f"#### `{method.name}`")
                    lines.append("")
                    if method.signature:
                        lines.append("```rust")
                        lines.append(method.signature)
                        lines.append("```")
                        lines.append("")
                    if method.docs:
                        lines.append(method.docs)
                        lines.append("")
            
            lines.append("---")
            lines.append("")
    
    # Footer
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("*This documentation was automatically generated from the source code.*")
    
    return '\n'.join(lines)

def scan_crate(crate_path: Path) -> List[DocItem]:
    """Scan a crate directory for all documented items."""
    parser = RustDocParser()
    all_items = []
    
    # Scan src directory
    src_dir = crate_path / 'src'
    if not src_dir.exists():
        return all_items
    
    # Find all .rs files
    for rs_file in src_dir.rglob('*.rs'):
        items = parser.parse_file(rs_file)
        all_items.extend(items)
    
    return all_items

def main():
    if len(sys.argv) != 3:
        print("Usage: generate_comprehensive_docs.py <repo_root> <output_dir>")
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
    
    total_items = 0
    
    for crate_name in crates:
        print(f"📦 Processing {crate_name}...")
        crate_path = repo_root / crate_name
        
        if not crate_path.exists():
            print(f"  ⚠️  Crate directory not found")
            continue
        
        # Scan for all items
        items = scan_crate(crate_path)
        total_items += len(items)
        
        # Count by type
        categories = organize_items_by_category(items)
        type_counts = {k: len(v) for k, v in categories.items()}
        
        print(f"  📊 Found {len(items)} items total:")
        for category, count in sorted(type_counts.items()):
            if count > 0:
                print(f"     - {count} {category.lower()}")
        
        # Generate markdown
        markdown = generate_markdown(crate_name, items)
        
        # Write to file
        output_file = output_dir / f"{crate_name}.md"
        output_file.write_text(markdown)
        print(f"  ✅ Generated {output_file.name}")
        print("")
    
    print(f"🎉 Documentation generation complete!")
    print(f"📈 Generated documentation for {total_items} total items across {len(crates)} crates")

if __name__ == "__main__":
    main()