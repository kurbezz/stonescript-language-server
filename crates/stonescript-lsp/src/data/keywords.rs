//! StoneScript Keywords and Commands Data

/// Information about a keyword or command
#[derive(Debug, Clone)]
pub struct KeywordInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub examples: &'static [&'static str],
    pub category: KeywordCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordCategory {
    ControlFlow,
    Declaration,
    Loop,
    Import,
    Equipment,
    Action,
    Toggle,
    Print,
}

/// All StoneScript keywords
pub const KEYWORDS: &[KeywordInfo] = &[
    // Control Flow
    KeywordInfo {
        name: "?",
        description: "Evaluates a logical condition. If true, executes indented lines.",
        usage: "?<condition>",
        examples: &["?loc = caves", "?hp < 10"],
        category: KeywordCategory::ControlFlow,
    },
    KeywordInfo {
        name: ":",
        description: "Alternative logical branch (else).",
        usage: ":",
        examples: &["?loc = caves\n  loadout 1\n:\n  loadout 2"],
        category: KeywordCategory::ControlFlow,
    },
    KeywordInfo {
        name: ":?",
        description: "Alternative logical branch with condition (else-if).",
        usage: ":?<condition>",
        examples: &[":?loc = deadwood\n  loadout 2"],
        category: KeywordCategory::ControlFlow,
    },
    
    // Declarations
    KeywordInfo {
        name: "var",
        description: "Declares a variable.",
        usage: "var <name> [= <value>]",
        examples: &["var x = 10", "var message = \"Hello\""],
        category: KeywordCategory::Declaration,
    },
    KeywordInfo {
        name: "func",
        description: "Declares a function.",
        usage: "func <name>([params])",
        examples: &["func Print(msg)\n  >@msg@"],
        category: KeywordCategory::Declaration,
    },
    
    // Loops
    KeywordInfo {
        name: "for",
        description: "Creates a loop.",
        usage: "for <var> = <start>..<end> | for <var> : <array>",
        examples: &["for i = 1..5\n  >@i@", "for item : arr\n  >@item@"],
        category: KeywordCategory::Loop,
    },
    KeywordInfo {
        name: "break",
        description: "Breaks out of the current loop.",
        usage: "break",
        examples: &["for i = 1..10\n  ?i = 5\n    break"],
        category: KeywordCategory::Loop,
    },
    KeywordInfo {
        name: "continue",
        description: "Skips rest of loop iteration.",
        usage: "continue",
        examples: &["for i = 1..10\n  ?i = 5\n    continue"],
        category: KeywordCategory::Loop,
    },
    KeywordInfo {
        name: "return",
        description: "Returns a value from a function.",
        usage: "return [<value>]",
        examples: &["func add(a, b)\n  return a + b"],
        category: KeywordCategory::Loop,
    },
    
    // Import
    KeywordInfo {
        name: "import",
        description: "Loads an external script (singleton).",
        usage: "import <scriptPath>",
        examples: &["import UI/Button"],
        category: KeywordCategory::Import,
    },
    KeywordInfo {
        name: "new",
        description: "Loads an external script (unique instance).",
        usage: "new <scriptPath>",
        examples: &["var v = new Components/Vector"],
        category: KeywordCategory::Import,
    },
    
    // Equipment
    KeywordInfo {
        name: "equip",
        description: "Equips an item (two-handed).",
        usage: "equip <criteria...>",
        examples: &["equip vigor crossbow *8 +5", "equip shovel"],
        category: KeywordCategory::Equipment,
    },
    KeywordInfo {
        name: "equipL",
        description: "Equips an item to left hand.",
        usage: "equipL <criteria...>",
        examples: &["equipL poison d_sword"],
        category: KeywordCategory::Equipment,
    },
    KeywordInfo {
        name: "equipR",
        description: "Equips an item to right hand.",
        usage: "equipR <criteria...>",
        examples: &["equipR vigor shield"],
        category: KeywordCategory::Equipment,
    },
    
    // Actions
    KeywordInfo {
        name: "activate",
        description: "Activates an item ability or potion.",
        usage: "activate <ability>",
        examples: &["activate R", "activate potion", "activate bardiche"],
        category: KeywordCategory::Action,
    },
    KeywordInfo {
        name: "loadout",
        description: "Equips a loadout number.",
        usage: "loadout <number>",
        examples: &["loadout 1", "loadout 2"],
        category: KeywordCategory::Action,
    },
    KeywordInfo {
        name: "brew",
        description: "Refills potion with ingredients.",
        usage: "brew <ingredients...>",
        examples: &["brew bronze + tar", "brew stone + wood"],
        category: KeywordCategory::Action,
    },
    KeywordInfo {
        name: "play",
        description: "Plays a sound effect.",
        usage: "play <sound> [pitch]",
        examples: &["play buy", "play buy 200"],
        category: KeywordCategory::Action,
    },
    
    // Toggle
    KeywordInfo {
        name: "disable",
        description: "Disables game features.",
        usage: "disable <feature>",
        examples: &["disable hud", "disable abilities"],
        category: KeywordCategory::Toggle,
    },
    KeywordInfo {
        name: "enable",
        description: "Enables game features.",
        usage: "enable <feature>",
        examples: &["enable hud", "enable abilities"],
        category: KeywordCategory::Toggle,
    },
    
    // Print
    KeywordInfo {
        name: ">",
        description: "Prints text to screen.",
        usage: "><text>",
        examples: &[">Hello World!", ">@varName@"],
        category: KeywordCategory::Print,
    },
    KeywordInfo {
        name: ">o",
        description: "Advanced print relative to player.",
        usage: ">oX,Y,[#color,]<text>",
        examples: &[">o-6,3,#red,Let's go!"],
        category: KeywordCategory::Print,
    },
];

/// Get keyword info by name
pub fn get_keyword(name: &str) -> Option<&'static KeywordInfo> {
    KEYWORDS.iter().find(|k| k.name == name)
}

/// Get all keywords in a category
pub fn get_keywords_by_category(category: KeywordCategory) -> Vec<&'static KeywordInfo> {
    KEYWORDS.iter().filter(|k| k.category == category).collect()
}
