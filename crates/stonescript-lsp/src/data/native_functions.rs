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
        parameters: &[Parameter { name: "x", typ: Type::Float, optional: false }],
        return_type: Type::Float,
        description: "Returns absolute value",
    },
    FunctionSignature {
        namespace: "math",
        name: "Sqrt",
        parameters: &[Parameter { name: "x", typ: Type::Float, optional: false }],
        return_type: Type::Float,
        description: "Returns square root",
    },
    FunctionSignature {
        namespace: "math",
        name: "Pow",
        parameters: &[
            Parameter { name: "base", typ: Type::Float, optional: false },
            Parameter { name: "exponent", typ: Type::Float, optional: false },
        ],
        return_type: Type::Float,
        description: "Returns base raised to exponent",
    },
    FunctionSignature {
        namespace: "math",
        name: "Max",
        parameters: &[
            Parameter { name: "a", typ: Type::Float, optional: false },
            Parameter { name: "b", typ: Type::Float, optional: false },
        ],
        return_type: Type::Float,
        description: "Returns maximum value",
    },
    FunctionSignature {
        namespace: "math",
        name: "Min",
        parameters: &[
            Parameter { name: "a", typ: Type::Float, optional: false },
            Parameter { name: "b", typ: Type::Float, optional: false },
        ],
        return_type: Type::Float,
        description: "Returns minimum value",
    },
    FunctionSignature {
        namespace: "math",
        name: "Floor",
        parameters: &[Parameter { name: "x", typ: Type::Float, optional: false }],
        return_type: Type::Float,
        description: "Rounds down to nearest integer",
    },
    FunctionSignature {
        namespace: "math",
        name: "Ceil",
        parameters: &[Parameter { name: "x", typ: Type::Float, optional: false }],
        return_type: Type::Float,
        description: "Rounds up to nearest integer",
    },
    FunctionSignature {
        namespace: "math",
        name: "Round",
        parameters: &[Parameter { name: "x", typ: Type::Float, optional: false }],
        return_type: Type::Float,
        description: "Rounds to nearest integer",
    },
];

/// String namespace functions
pub const STRING_FUNCTIONS: &[FunctionSignature] = &[
    FunctionSignature {
        namespace: "string",
        name: "Size",
        parameters: &[Parameter { name: "s", typ: Type::String, optional: false }],
        return_type: Type::Int,
        description: "Returns string length",
    },
    FunctionSignature {
        namespace: "string",
        name: "Sub",
        parameters: &[
            Parameter { name: "s", typ: Type::String, optional: false },
            Parameter { name: "start", typ: Type::Int, optional: false },
            Parameter { name: "length", typ: Type::Int, optional: true },
        ],
        return_type: Type::String,
        description: "Returns substring",
    },
    FunctionSignature {
        namespace: "string",
        name: "Split",
        parameters: &[
            Parameter { name: "s", typ: Type::String, optional: false },
            Parameter { name: "delimiter", typ: Type::String, optional: false },
        ],
        return_type: Type::Array(&Type::String),  // Fixed: use reference
        description: "Splits string into array",
    },
    FunctionSignature {
        namespace: "string",
        name: "Join",
        parameters: &[
            Parameter { name: "array", typ: Type::Array(&Type::String), optional: false },  // Fixed
            Parameter { name: "delimiter", typ: Type::String, optional: false },
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
            Parameter { name: "key", typ: Type::String, optional: false },
            Parameter { name: "value", typ: Type::Unknown, optional: false },
        ],
        return_type: Type::Unknown,
        description: "Stores a value",
    },
    FunctionSignature {
        namespace: "storage",
        name: "Get",
        parameters: &[Parameter { name: "key", typ: Type::String, optional: false }],
        return_type: Type::Unknown,
        description: "Retrieves a stored value",
    },
    FunctionSignature {
        namespace: "storage",
        name: "Has",
        parameters: &[Parameter { name: "key", typ: Type::String, optional: false }],
        return_type: Type::Bool,
        description: "Checks if key exists",
    },
];

/// All native functions combined
pub const ALL_FUNCTIONS: &[&[FunctionSignature]] = &[
    MATH_FUNCTIONS,
    STRING_FUNCTIONS,
    STORAGE_FUNCTIONS,
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
