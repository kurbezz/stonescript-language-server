//! Game state queries and types

/// Type of a value in StoneScript
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Object(&'static str),
    Array(&'static Type),  // Changed from Box to static reference
    Function,
    Unknown,
}

/// Property of a game object
#[derive(Debug, Clone)]
pub struct Property {
    pub name: &'static str,
    pub typ: Type,
    pub description: &'static str,
}

/// A game state query (like ?loc, ?foe, ?hp)
#[derive(Debug, Clone)]
pub struct GameStateQuery {
    pub name: &'static str,
    pub description: &'static str,
    pub return_type: Type,
    pub properties: Option<&'static [Property]>,
}

// Location properties
const LOC_PROPERTIES: &[Property] = &[
    Property {
        name: "id",
        typ: Type::String,
        description: "Location identifier",
    },
    Property {
        name: "name",
        typ: Type::String,
        description: "Location display name",
    },
    Property {
        name: "stars",
        typ: Type::Int,
        description: "Number of stars earned",
    },
    Property {
        name: "begin",
        typ: Type::Bool,
        description: "True at start of location",
    },
    Property {
        name: "loop",
        typ: Type::Bool,
        description: "True on each loop",
    },
    Property {
        name: "gp",
        typ: Type::Int,
        description: "Gold earned this run",
    },
];

// Foe properties
const FOE_PROPERTIES: &[Property] = &[
    Property {
        name: "id",
        typ: Type::String,
        description: "Foe identifier",
    },
    Property {
        name: "name",
        typ: Type::String,
        description: "Foe display name",
    },
    Property {
        name: "hp",
        typ: Type::Int,
        description: "Current health",
    },
    Property {
        name: "maxhp",
        typ: Type::Int,
        description: "Maximum health",
    },
    Property {
        name: "armor",
        typ: Type::Int,
        description: "Armor value",
    },
    Property {
        name: "distance",
        typ: Type::Int,
        description: "Distance from player",
    },
    Property {
        name: "damage",
        typ: Type::Int,
        description: "Damage dealt by foe",
    },
    Property {
        name: "count",
        typ: Type::Int,
        description: "Number of foes",
    },
];

/// All game state queries
pub const GAME_STATE_QUERIES: &[GameStateQuery] = &[
    // Location
    GameStateQuery {
        name: "loc",
        description: "Current location",
        return_type: Type::Object("Location"),
        properties: Some(LOC_PROPERTIES),
    },
    
    // Foe
    GameStateQuery {
        name: "foe",
        description: "Current foe",
        return_type: Type::Object("Foe"),
        properties: Some(FOE_PROPERTIES),
    },
    
    // Player stats
    GameStateQuery {
        name: "hp",
        description: "Current health",
        return_type: Type::Int,
        properties: None,
    },
    GameStateQuery {
        name: "maxhp",
        description: "Maximum health",
        return_type: Type::Int,
        properties: None,
    },
    GameStateQuery {
        name: "armor",
        description: "Current armor",
        return_type: Type::Int,
        properties: None,
    },
    GameStateQuery {
        name: "buffs",
        description: "Active buffs",
        return_type: Type::String,
        properties: None,
    },
    GameStateQuery {
        name: "debuffs",
        description: "Active debuffs",
        return_type: Type::String,
        properties: None,
    },
    
    // Time
    GameStateQuery {
        name: "time",
        description: "Current time in seconds",
        return_type: Type::Float,
        properties: None,
    },
    GameStateQuery {
        name: "totaltime",
        description: "Total time played",
        return_type: Type::Float,
        properties: None,
    },
    
    // Screen
    GameStateQuery {
        name: "screen",
        description: "Screen dimensions",
        return_type: Type::Object("Screen"),
        properties: Some(&[
            Property {
                name: "w",
                typ: Type::Int,
                description: "Screen width",
            },
            Property {
                name: "h",
                typ: Type::Int,
                description: "Screen height",
            },
        ]),
    },
    
    // Input
    GameStateQuery {
        name: "input",
        description: "Input state",
        return_type: Type::Object("Input"),
        properties: Some(&[
            Property {
                name: "x",
                typ: Type::Int,
                description: "Mouse X position",
            },
            Property {
                name: "y",
                typ: Type::Int,
                description: "Mouse Y position",
            },
        ]),
    },
];

/// Get game state query by name
pub fn get_game_state(name: &str) -> Option<&'static GameStateQuery> {
    GAME_STATE_QUERIES.iter().find(|q| q.name == name)
}

/// Get all properties of a game object
pub fn get_properties(object_name: &str) -> Option<&'static [Property]> {
    GAME_STATE_QUERIES
        .iter()
        .find(|q| q.name == object_name)
        .and_then(|q| q.properties)
}
