//! Completion provider

use crate::data::abilities::ABILITY_IDS;
use crate::data::filters::{FOE_FILTERS, ITEM_FILTERS};
use crate::data::foes::FOES;
use crate::data::locations::LOCATIONS;
use crate::data::music::MUSIC_TRACKS;
use crate::data::native_functions::{
    MATH_FUNCTIONS, MUSIC_FUNCTIONS, STORAGE_FUNCTIONS, STRING_FUNCTIONS, UI_FUNCTIONS,
};
use crate::data::sounds::SOUND_EFFECTS;
use crate::data::ui::{UI_METHODS, UI_PROPERTIES};
use crate::data::*;
use crate::utils::ScopeAnalyzer;
use regex;
use tower_lsp::lsp_types::*;
use tree_sitter::{Point, Tree};

#[derive(Debug)]
pub enum CompletionContext {
    /// Top-level statement context
    TopLevel,
    /// After a dot (member access)
    MemberAccess(String), // object name
    /// Inside function call
    FunctionCall,
    /// Inside specific function call (namespace, function_name)
    InsideFunctionCall(String, String),
    /// After keywords
    AfterKeyword(String),
    /// Variable/function reference
    Identifier,
    /// Binary expression with identifier (e.g., loc=, foe=)
    BinaryWithIdentifier(String),
    /// After equip/equipL/equipR keywords (for item filters)
    AfterEquip,
}

pub struct CompletionProvider {
    keywords: &'static [KeywordInfo],
    game_state: &'static [GameStateQuery],
}

impl CompletionProvider {
    pub fn new() -> Self {
        Self {
            keywords: KEYWORDS,
            game_state: GAME_STATE_QUERIES,
        }
    }

    pub fn provide_completion(
        &self,
        tree: &Tree,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Vec<CompletionItem> {
        let point = Point::new(position.line as usize, position.character as usize);

        // Determine context from AST
        let context = self.determine_context(tree, point, source);

        let mut items = match context {
            CompletionContext::TopLevel => self.complete_top_level(scope),
            CompletionContext::MemberAccess(object) => self.complete_member_access(&object, source),
            CompletionContext::AfterKeyword(keyword) => self.complete_after_keyword(&keyword),
            CompletionContext::AfterEquip => self.complete_after_equip(),
            CompletionContext::FunctionCall => self.complete_function_call(scope),
            CompletionContext::InsideFunctionCall(namespace, func) => {
                self.complete_inside_function_call(&namespace, &func)
            }
            CompletionContext::Identifier => self.complete_identifier(scope),
            CompletionContext::BinaryWithIdentifier(identifier) => {
                self.complete_binary_identifier(&identifier, scope)
            }
        };

        // If no items found, try fallback strategies
        if items.is_empty() {
            // Try text-based detection one more time with simpler patterns
            items = self.fallback_completion(position, source, scope);
        }

        items
    }

    fn determine_context(&self, tree: &Tree, point: Point, source: &str) -> CompletionContext {
        // First, try text-based detection for incomplete expressions
        // This handles cases like "?loc=" where the tree has ERROR nodes
        let line_start = source
            .lines()
            .take(point.row)
            .map(|line| line.len() + 1)
            .sum::<usize>();
        let line_end = line_start + source.lines().nth(point.row).map(|l| l.len()).unwrap_or(0);

        if line_end > line_start {
            let line_text = &source[line_start..line_end];
            let cursor_col = point.column;

            // Check for patterns like "?loc=" or "?foe=" with cursor after =
            if let Some(text_before_cursor) = line_text.get(..cursor_col.min(line_text.len())) {
                // Pattern 1: ?identifier= or ?identifier=partial
                // Matches: "?loc=", "?loc= ", "?loc=r", etc.
                if let Some(caps) = regex::Regex::new(r"\?(\w+)\s*=\s*\w*$")
                    .ok()
                    .and_then(|re| re.captures(text_before_cursor))
                {
                    if let Some(ident) = caps.get(1) {
                        let ident_str = ident.as_str();
                        // Check if this is a known game state identifier
                        if matches!(ident_str, "loc" | "foe" | "item" | "ai") {
                            return CompletionContext::BinaryWithIdentifier(ident_str.to_string());
                        }
                    }
                }

                // Pattern 2: identifier= (without ?) for nested contexts
                // Matches: " loc=", " foe=", etc.
                if let Some(caps) = regex::Regex::new(r"[\s\(](\w+)\s*=\s*\w*$")
                    .ok()
                    .and_then(|re| re.captures(text_before_cursor))
                {
                    if let Some(ident) = caps.get(1) {
                        let ident_str = ident.as_str();
                        // Only trigger for known game state identifiers
                        if matches!(ident_str, "loc" | "foe" | "item" | "ai") {
                            return CompletionContext::BinaryWithIdentifier(ident_str.to_string());
                        }
                    }
                }

                // Pattern 3: Check for member access with dot
                // Matches: "loc.", "foe.", etc.
                if let Some(caps) = regex::Regex::new(r"(\w+)\.\s*$")
                    .ok()
                    .and_then(|re| re.captures(text_before_cursor))
                {
                    if let Some(ident) = caps.get(1) {
                        return CompletionContext::MemberAccess(ident.as_str().to_string());
                    }
                }

                // Pattern 4: Check for keywords followed by space
                // Matches: "play ", "activate ", "equip ", "equipL ", "equipR "
                if let Some(caps) = regex::Regex::new(r"^\s*(play|activate)\s+\w*$")
                    .ok()
                    .and_then(|re| re.captures(text_before_cursor))
                {
                    if let Some(keyword) = caps.get(1) {
                        return CompletionContext::AfterKeyword(keyword.as_str().to_string());
                    }
                }

                // Pattern 5: Check for equip commands
                if let Some(_caps) = regex::Regex::new(r"^\s*(equip|equipL|equipR)\s+")
                    .ok()
                    .and_then(|re| re.captures(text_before_cursor))
                {
                    return CompletionContext::AfterEquip;
                }
            }
        }

        let node = tree
            .root_node()
            .named_descendant_for_point_range(point, point);

        if let Some(current_node) = node {
            // Check if we're after a dot
            if current_node.kind() == "." {
                // Find the object before the dot
                if let Some(prev) = current_node.prev_sibling() {
                    if prev.kind() == "identifier" {
                        let text = prev.utf8_text(source.as_bytes()).unwrap_or("");
                        return CompletionContext::MemberAccess(text.to_string());
                    }
                }
            }

            // Walk up the tree to find context
            let mut check_node = Some(current_node);
            while let Some(node) = check_node {
                // Check if we're in a binary expression
                if node.kind() == "binary_expression" {
                    // Find the left side of the comparison
                    if let Some(left_node) = node.child(0) {
                        if left_node.kind() == "identifier" {
                            let text = left_node.utf8_text(source.as_bytes()).unwrap_or("");
                            // Check if cursor is after the operator (for completion)
                            let operator_end = if let Some(op_node) = node.child(1) {
                                op_node.end_position()
                            } else {
                                left_node.end_position()
                            };

                            // If cursor is after the operator, provide value completion
                            if point.column >= operator_end.column {
                                return CompletionContext::BinaryWithIdentifier(text.to_string());
                            }
                        }
                    }
                }

                // Check if we're in a member expression
                if node.kind() == "member_expression" {
                    if let Some(object_node) = node.child_by_field_name("object") {
                        let text = object_node.utf8_text(source.as_bytes()).unwrap_or("");
                        return CompletionContext::MemberAccess(text.to_string());
                    }
                }

                // Check if we're in a call expression
                if node.kind() == "call_expression" {
                    // Try to determine the function being called
                    if let Some(func_node) = node.child_by_field_name("function") {
                        if func_node.kind() == "member_expression" {
                            // namespace.function() call
                            if let Some(obj) = func_node.child_by_field_name("object") {
                                if let Some(prop) = func_node.child_by_field_name("property") {
                                    let namespace = obj.utf8_text(source.as_bytes()).unwrap_or("");
                                    let function = prop.utf8_text(source.as_bytes()).unwrap_or("");
                                    return CompletionContext::InsideFunctionCall(
                                        namespace.to_string(),
                                        function.to_string(),
                                    );
                                }
                            }
                        }
                    }
                    return CompletionContext::FunctionCall;
                }

                // Check if we're in a conditional
                if node.kind() == "conditional" {
                    // Check the expression part of the conditional
                    if let Some(expr_node) = node.child(1) {
                        if expr_node.kind() == "binary_expression" {
                            if let Some(left_node) = expr_node.child(0) {
                                if left_node.kind() == "identifier" {
                                    let text = left_node.utf8_text(source.as_bytes()).unwrap_or("");
                                    let operator_end = if let Some(op_node) = expr_node.child(1) {
                                        op_node.end_position()
                                    } else {
                                        left_node.end_position()
                                    };

                                    if point.column >= operator_end.column {
                                        return CompletionContext::BinaryWithIdentifier(
                                            text.to_string(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }

                check_node = node.parent();
            }
        }

        // Default to top-level
        CompletionContext::TopLevel
    }

    fn complete_top_level(&self, scope: &ScopeAnalyzer) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Add keywords
        for keyword in self.keywords {
            items.push(CompletionItem {
                label: keyword.name.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(keyword.description.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "```stonescript\n{}\n```\n\n{}",
                        keyword.usage, keyword.description
                    ),
                })),
                ..Default::default()
            });
        }

        // Add game state queries
        for query in self.game_state {
            items.push(CompletionItem {
                label: query.name.to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some(query.description.to_string()),
                ..Default::default()
            });
        }

        // Add user-defined variables
        let variables = scope.find_variables_at(0);
        for var in variables {
            items.push(CompletionItem {
                label: var.name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("user variable".to_string()),
                ..Default::default()
            });
        }

        // Add user-defined functions
        for func in scope.get_functions() {
            items.push(CompletionItem {
                label: func.name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("func({})", func.parameters.join(", "))),
                ..Default::default()
            });
        }

        items
    }

    fn complete_member_access(&self, object: &str, source: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Helper to check if a variable is a UI component
        let is_ui_component = |var_name: &str| -> bool {
            let pattern = format!(
                r"var\s+{}\s*=\s*ui\.(AddPanel|AddButton|AddText|AddAnim|root)",
                regex::escape(var_name)
            );
            regex::Regex::new(&pattern)
                .ok()
                .and_then(|re| re.find(source))
                .is_some()
        };

        // Find game state object
        if let Some(query) = self.game_state.iter().find(|q| q.name == object) {
            if let Some(properties) = query.properties {
                for prop in properties {
                    items.push(CompletionItem {
                        label: prop.name.to_string(),
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(prop.description.to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        // Check for namespace functions
        match object {
            "math" => {
                for func in MATH_FUNCTIONS {
                    items.push(CompletionItem {
                        label: func.name.to_string(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(func.description.to_string()),
                        ..Default::default()
                    });
                }
            }
            "string" => {
                for func in STRING_FUNCTIONS {
                    items.push(CompletionItem {
                        label: func.name.to_string(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(func.description.to_string()),
                        ..Default::default()
                    });
                }
            }
            "storage" => {
                for func in STORAGE_FUNCTIONS {
                    items.push(CompletionItem {
                        label: func.name.to_string(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(func.description.to_string()),
                        ..Default::default()
                    });
                }
            }
            "music" => {
                for func in MUSIC_FUNCTIONS {
                    items.push(CompletionItem {
                        label: func.name.to_string(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(func.description.to_string()),
                        ..Default::default()
                    });
                }
            }
            "ui" => {
                for func in UI_FUNCTIONS {
                    items.push(CompletionItem {
                        label: func.name.to_string(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(func.description.to_string()),
                        ..Default::default()
                    });
                }
            }
            "cooldown" => {
                // Add ability IDs for cooldown queries
                for ability in ABILITY_IDS {
                    items.push(CompletionItem {
                        label: ability.to_string(),
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(format!("Ability cooldown for {}", ability)),
                        ..Default::default()
                    });
                }
            }
            _ => {
                // Check if this is a UI component variable
                if is_ui_component(object) {
                    // Add UI properties
                    for prop in UI_PROPERTIES {
                        items.push(CompletionItem {
                            label: prop.to_string(),
                            kind: Some(CompletionItemKind::PROPERTY),
                            detail: Some("UI property".to_string()),
                            ..Default::default()
                        });
                    }
                    // Add UI methods
                    for method in UI_METHODS {
                        items.push(CompletionItem {
                            label: method.to_string(),
                            kind: Some(CompletionItemKind::METHOD),
                            detail: Some("UI method".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        items
    }

    fn complete_after_keyword(&self, keyword: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        match keyword {
            "play" => {
                // Complete with sound effect IDs
                for sound in SOUND_EFFECTS {
                    items.push(CompletionItem {
                        label: sound.to_string(),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some(format!("Sound effect: {}", sound)),
                        insert_text: Some(sound.to_string()),
                        ..Default::default()
                    });
                }
            }
            "activate" => {
                // Complete with ability IDs
                for ability in ABILITY_IDS {
                    items.push(CompletionItem {
                        label: ability.to_string(),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some(format!("Ability: {}", ability)),
                        insert_text: Some(ability.to_string()),
                        ..Default::default()
                    });
                }
                // Also add special activations
                items.push(CompletionItem {
                    label: "R".to_string(),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("Activate right hand item".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "L".to_string(),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("Activate left hand item".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "potion".to_string(),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("Drink a potion".to_string()),
                    ..Default::default()
                });
            }
            _ => {}
        }

        items
    }

    fn complete_after_equip(&self) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Add item filters
        for filter in ITEM_FILTERS {
            items.push(CompletionItem {
                label: filter.to_string(),
                kind: Some(CompletionItemKind::CONSTANT),
                detail: Some(format!("Item filter: {}", filter)),
                insert_text: Some(filter.to_string()),
                ..Default::default()
            });
        }

        items
    }

    fn complete_function_call(&self, scope: &ScopeAnalyzer) -> Vec<CompletionItem> {
        // Complete function names and variables
        self.complete_identifier(scope)
    }

    fn complete_inside_function_call(
        &self,
        namespace: &str,
        function: &str,
    ) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Special handling for music.Play() - suggest music tracks
        if namespace == "music" && function == "Play" {
            for track in MUSIC_TRACKS {
                items.push(CompletionItem {
                    label: track.to_string(),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some(format!("Music track: {}", track)),
                    insert_text: Some(format!("\"{}\"", track)),
                    ..Default::default()
                });
            }
        }

        items
    }

    fn complete_identifier(&self, scope: &ScopeAnalyzer) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Variables
        for var in scope.find_variables_at(0) {
            items.push(CompletionItem {
                label: var.name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            });
        }

        // Functions
        for func in scope.get_functions() {
            items.push(CompletionItem {
                label: func.name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()
            });
        }

        items
    }

    fn complete_binary_identifier(
        &self,
        identifier: &str,
        scope: &ScopeAnalyzer,
    ) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        match identifier {
            "loc" => {
                // Complete with location identifiers
                for location in LOCATIONS {
                    items.push(CompletionItem {
                        label: location.to_string(),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some(format!(
                            "Location: {}",
                            crate::data::locations::get_location_name(location).unwrap_or(location)
                        )),
                        insert_text: Some(location.to_string()),
                        filter_text: Some(location.to_string()),
                        sort_text: Some(format!("00_{}", location)),
                        ..Default::default()
                    });
                }
            }
            "foe" => {
                // Complete with foe identifiers and filters
                for foe in FOES {
                    items.push(CompletionItem {
                        label: foe.to_string(),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some(format!(
                            "Foe: {}",
                            crate::data::foes::get_foe_name(foe).unwrap_or(foe)
                        )),
                        insert_text: Some(foe.to_string()),
                        filter_text: Some(foe.to_string()),
                        sort_text: Some(format!("00_{}", foe)),
                        ..Default::default()
                    });
                }
                // Add foe filters
                for filter in FOE_FILTERS {
                    items.push(CompletionItem {
                        label: filter.to_string(),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some(format!("Foe filter: {}", filter)),
                        insert_text: Some(filter.to_string()),
                        filter_text: Some(filter.to_string()),
                        sort_text: Some(format!("01_{}", filter)),
                        ..Default::default()
                    });
                }
            }
            "item" => {
                // Complete with item filters
                for filter in ITEM_FILTERS {
                    items.push(CompletionItem {
                        label: filter.to_string(),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some(format!("Item filter: {}", filter)),
                        insert_text: Some(filter.to_string()),
                        filter_text: Some(filter.to_string()),
                        sort_text: Some(format!("00_{}", filter)),
                        ..Default::default()
                    });
                }
            }
            _ => {
                // For other identifiers, fall back to general completion
                items.extend(self.complete_identifier(scope));
            }
        }

        items
    }

    fn fallback_completion(
        &self,
        position: Position,
        source: &str,
        scope: &ScopeAnalyzer,
    ) -> Vec<CompletionItem> {
        let line_start = source
            .lines()
            .take(position.line as usize)
            .map(|line| line.len() + 1)
            .sum::<usize>();
        let line_end = line_start
            + source
                .lines()
                .nth(position.line as usize)
                .map(|l| l.len())
                .unwrap_or(0);

        if line_end > line_start {
            let line_text = &source[line_start..line_end];

            // Very aggressive pattern matching - check for loc or foe anywhere in line
            if line_text.contains("loc") && line_text.contains("=") {
                return self.complete_binary_identifier("loc", scope);
            }
            if line_text.contains("foe") && line_text.contains("=") {
                return self.complete_binary_identifier("foe", scope);
            }
            if line_text.contains("item") && line_text.contains("=") {
                return self.complete_binary_identifier("item", scope);
            }
        }

        // Last resort - return top level completion
        self.complete_top_level(scope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_provider() {
        let provider = CompletionProvider::new();
        let scope = ScopeAnalyzer::new();

        // Test would need a real tree
        // This is a placeholder
        assert!(provider.keywords.len() > 0);
    }
}
