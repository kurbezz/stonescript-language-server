//! Native functions available in StoneScript

use super::game_state::Type;

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: &'static str,
    pub typ: Type,
    pub optional: bool,
}

/// Native function signature
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub namespace: &'static str,
    pub name: &'static str,
    pub parameters: &'static [Parameter],
    pub return_type: Type,
    pub description: &'static str,
}

/// Math namespace functions
pub const MATH_FUNCTIONS: &[FunctionSignature] = &[
    FunctionSignature {
        namespace: "math",
        name: "Abs",
        parameters: &[Parameter {
            name: "x",
            typ: Type::Float,
            optional: false,
        }],
        return_type: Type::Float,
        description: "Returns absolute value",
    },
    FunctionSignature {
        namespace: "math",
        name: "Sqrt",
        parameters: &[Parameter {
            name: "x",
            typ: Type::Float,
            optional: false,
        }],
        return_type: Type::Float,
        description: "Returns square root",
    },
    FunctionSignature {
        namespace: "math",
        name: "Pow",
        parameters: &[
            Parameter {
                name: "base",
                typ: Type::Float,
                optional: false,
            },
            Parameter {
                name: "exponent",
                typ: Type::Float,
                optional: false,
            },
        ],
        return_type: Type::Float,
        description: "Returns base raised to exponent",
    },
    FunctionSignature {
        namespace: "math",
        name: "Max",
        parameters: &[
            Parameter {
                name: "a",
                typ: Type::Float,
                optional: false,
            },
            Parameter {
                name: "b",
                typ: Type::Float,
                optional: false,
            },
        ],
        return_type: Type::Float,
        description: "Returns maximum value",
    },
    FunctionSignature {
        namespace: "math",
        name: "Min",
        parameters: &[
            Parameter {
                name: "a",
                typ: Type::Float,
                optional: false,
            },
            Parameter {
                name: "b",
                typ: Type::Float,
                optional: false,
            },
        ],
        return_type: Type::Float,
        description: "Returns minimum value",
    },
    FunctionSignature {
        namespace: "math",
        name: "Floor",
        parameters: &[Parameter {
            name: "x",
            typ: Type::Float,
            optional: false,
        }],
        return_type: Type::Float,
        description: "Rounds down to nearest integer",
    },
    FunctionSignature {
        namespace: "math",
        name: "Ceil",
        parameters: &[Parameter {
            name: "x",
            typ: Type::Float,
            optional: false,
        }],
        return_type: Type::Float,
        description: "Rounds up to nearest integer",
    },
    FunctionSignature {
        namespace: "math",
        name: "Round",
        parameters: &[Parameter {
            name: "x",
            typ: Type::Float,
            optional: false,
        }],
        return_type: Type::Float,
        description: "Rounds to nearest integer",
    },
];

/// String namespace functions
pub const STRING_FUNCTIONS: &[FunctionSignature] = &[
    FunctionSignature {
        namespace: "string",
        name: "Size",
        parameters: &[Parameter {
            name: "s",
            typ: Type::String,
            optional: false,
        }],
        return_type: Type::Int,
        description: "Returns string length",
    },
    FunctionSignature {
        namespace: "string",
        name: "Sub",
        parameters: &[
            Parameter {
                name: "s",
                typ: Type::String,
                optional: false,
            },
            Parameter {
                name: "start",
                typ: Type::Int,
                optional: false,
            },
            Parameter {
                name: "length",
                typ: Type::Int,
                optional: true,
            },
        ],
        return_type: Type::String,
        description: "Returns substring",
    },
    FunctionSignature {
        namespace: "string",
        name: "Split",
        parameters: &[
            Parameter {
                name: "s",
                typ: Type::String,
                optional: false,
            },
            Parameter {
                name: "delimiter",
                typ: Type::String,
                optional: false,
            },
        ],
        return_type: Type::Array(&Type::String), // Fixed: use reference
        description: "Splits string into array",
    },
    FunctionSignature {
        namespace: "string",
        name: "Join",
        parameters: &[
            Parameter {
                name: "array",
                typ: Type::Array(&Type::String),
                optional: false,
            }, // Fixed
            Parameter {
                name: "delimiter",
                typ: Type::String,
                optional: false,
            },
        ],
        return_type: Type::String,
        description: "Joins array into string",
    },
];

/// Storage namespace functions
pub const STORAGE_FUNCTIONS: &[FunctionSignature] = &[
    FunctionSignature {
        namespace: "storage",
        name: "Set",
        parameters: &[
            Parameter {
                name: "key",
                typ: Type::String,
                optional: false,
            },
            Parameter {
                name: "value",
                typ: Type::Unknown,
                optional: false,
            },
        ],
        return_type: Type::Unknown,
        description: "Stores a value",
    },
    FunctionSignature {
        namespace: "storage",
        name: "Get",
        parameters: &[Parameter {
            name: "key",
            typ: Type::String,
            optional: false,
        }],
        return_type: Type::Unknown,
        description: "Retrieves a stored value",
    },
    FunctionSignature {
        namespace: "storage",
        name: "Has",
        parameters: &[Parameter {
            name: "key",
            typ: Type::String,
            optional: false,
        }],
        return_type: Type::Bool,
        description: "Checks if key exists",
    },
];

/// Music namespace functions
pub const MUSIC_FUNCTIONS: &[FunctionSignature] = &[
    FunctionSignature {
        namespace: "music",
        name: "Play",
        parameters: &[Parameter {
            name: "track_id",
            typ: Type::String,
            optional: false,
        }],
        return_type: Type::Unknown,
        description: "Plays a music track by ID",
    },
    FunctionSignature {
        namespace: "music",
        name: "Stop",
        parameters: &[],
        return_type: Type::Unknown,
        description: "Stops all music",
    },
];

/// UI namespace functions
pub const UI_FUNCTIONS: &[FunctionSignature] = &[
    FunctionSignature {
        namespace: "ui",
        name: "AddPanel",
        parameters: &[],
        return_type: Type::Object("Panel"),
        description: "Adds a Panel object to the root Panel",
    },
    FunctionSignature {
        namespace: "ui",
        name: "AddButton",
        parameters: &[],
        return_type: Type::Object("Button"),
        description: "Adds a Button object to the root Panel",
    },
    FunctionSignature {
        namespace: "ui",
        name: "AddText",
        parameters: &[Parameter {
            name: "text",
            typ: Type::String,
            optional: true,
        }],
        return_type: Type::Object("Text"),
        description: "Adds a Text object to the root Panel",
    },
    FunctionSignature {
        namespace: "ui",
        name: "AddAnim",
        parameters: &[Parameter {
            name: "sprite_sheet",
            typ: Type::String,
            optional: false,
        }],
        return_type: Type::Object("Anim"),
        description: "Adds an Anim object to the root Panel",
    },
    FunctionSignature {
        namespace: "ui",
        name: "AddStyle",
        parameters: &[Parameter {
            name: "style_string",
            typ: Type::String,
            optional: false,
        }],
        return_type: Type::Int,
        description: "Adds a new style for UI components, returns style ID",
    },
    FunctionSignature {
        namespace: "ui",
        name: "Clear",
        parameters: &[],
        return_type: Type::Unknown,
        description: "Removes all UI elements from the main container",
    },
    FunctionSignature {
        namespace: "ui",
        name: "OpenInv",
        parameters: &[],
        return_type: Type::Unknown,
        description: "Opens the inventory",
    },
    FunctionSignature {
        namespace: "ui",
        name: "OpenMind",
        parameters: &[],
        return_type: Type::Unknown,
        description: "Opens the mind menu",
    },
    FunctionSignature {
        namespace: "ui",
        name: "ShowBanner",
        parameters: &[
            Parameter {
                name: "message1",
                typ: Type::String,
                optional: false,
            },
            Parameter {
                name: "message2",
                typ: Type::String,
                optional: true,
            },
        ],
        return_type: Type::Unknown,
        description: "Displays the animated banner with up to two messages",
    },
];

/// All native functions combined
pub const ALL_FUNCTIONS: &[&[FunctionSignature]] = &[
    MATH_FUNCTIONS,
    STRING_FUNCTIONS,
    STORAGE_FUNCTIONS,
    MUSIC_FUNCTIONS,
    UI_FUNCTIONS,
];

/// Get function by namespace and name
pub fn get_function(namespace: &str, name: &str) -> Option<&'static FunctionSignature> {
    ALL_FUNCTIONS
        .iter()
        .flat_map(|funcs| funcs.iter())
        .find(|f| f.namespace == namespace && f.name == name)
}

/// Get all functions in a namespace
pub fn get_functions_in_namespace(namespace: &str) -> Vec<&'static FunctionSignature> {
    ALL_FUNCTIONS
        .iter()
        .flat_map(|funcs| funcs.iter())
        .filter(|f| f.namespace == namespace)
        .collect()
}
