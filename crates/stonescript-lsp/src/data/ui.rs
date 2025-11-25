//! UI system types and properties

/// UI component types
pub const UI_COMPONENTS: &[&str] = &[
    "Panel",
    "Text",
    "Button",
    "Anim",
    "Canvas",
];

/// Common UI properties
pub const UI_PROPERTIES: &[&str] = &[
    // Component base properties
    "x", "y", "w", "h",
    "anchor", "dock",
    "ax", "ay", "dx", "dy",
    "visible",
    "absoluteX", "absoluteY",
    "parent",
    
    // Panel specific
    "children", "clip", "color", "style",
    
    // Text specific
    "align", "lines", "text",
    
    // Button specific
    "tcolor", "bcolor", "hcolor", "sound",
    
    // Anim specific
    "duration", "flipX", "flipY", "frame",
    "gamePause", "loop", "playing", "paused",
    "pivotX", "pivotY", "playOnStart",
    
    // Canvas specific
    "blend",
];

/// UI methods
pub const UI_METHODS: &[&str] = &[
    // Component
    "Recycle",
    
    // Panel
    "Add", "Clear", "Remove",
    
    // Button
    "SetPressed", "SetDown", "SetUp",
    
    // Anim
    "AddLayer", "Load", "Pause", "Play", "Stop",
    
    // Canvas
    "Get", "Set", "SetFG", "SetBG",
];
