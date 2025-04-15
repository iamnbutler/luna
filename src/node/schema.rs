use serde::{Serialize, Deserialize};
use schemars::{JsonSchema, schema_for};
use palette::Srgb;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "node_type")]
pub enum SerializedNode {
    Div {
        id: Option<String>,
        classes: Option<Vec<String>>,
        style: Option<Style>,
        children: Vec<SerializedNode>
    },
    Image {
        id: Option<String>,
        classes: Option<Vec<String>>,
        style: Option<Style>,
        src: String,
        alt: Option<String>
    },
    Text {
        id: Option<String>,
        classes: Option<Vec<String>>,
        style: Option<Style>,
        content: String
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Default)]
pub struct Style {
    // Colors and backgrounds
    pub background_color: Option<Color>,
    pub background_image: Option<String>,
    pub background_position: Option<String>,
    pub background_repeat: Option<String>,
    pub background_attachment: Option<String>,
    pub background_size: Option<String>,
    pub color: Option<Color>,
    pub opacity: Option<f32>,
    
    // Typography
    pub font_family: Option<String>,
    pub font_size: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_align: Option<TextAlign>,
    pub text_decoration: Option<TextDecoration>,
    pub line_height: Option<String>,
    pub letter_spacing: Option<String>,
    pub text_transform: Option<TextTransform>,
    pub white_space: Option<WhiteSpace>,
    pub word_break: Option<WordBreak>,
    pub text_overflow: Option<TextOverflow>,
    
    // Layout
    pub display: Option<Display>,
    pub position: Option<Position>,
    pub top: Option<String>,
    pub right: Option<String>,
    pub bottom: Option<String>,
    pub left: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub min_width: Option<String>,
    pub max_width: Option<String>,
    pub min_height: Option<String>,
    pub max_height: Option<String>,
    pub object_fit: Option<String>,
    pub margin: Option<String>,
    pub margin_top: Option<String>,
    pub margin_right: Option<String>,
    pub margin_bottom: Option<String>,
    pub margin_left: Option<String>,
    pub padding: Option<String>,
    pub padding_top: Option<String>,
    pub padding_right: Option<String>,
    pub padding_bottom: Option<String>,
    pub padding_left: Option<String>,
    pub z_index: Option<i32>,
    pub overflow: Option<Overflow>,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,
    
    // Flexbox
    pub flex_direction: Option<FlexDirection>,
    pub flex_wrap: Option<FlexWrap>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub align_content: Option<AlignContent>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<String>,
    pub align_self: Option<AlignSelf>,
    pub order: Option<i32>,
    
    // Borders
    pub border_style: Option<BorderStyle>,
    pub border_width: Option<String>,
    pub border_color: Option<Color>,
    pub border_radius: Option<String>,
    pub border_top_width: Option<String>,
    pub border_right_width: Option<String>,
    pub border_bottom_width: Option<String>,
    pub border_left_width: Option<String>,
    pub border_top_style: Option<BorderStyle>,
    pub border_right_style: Option<BorderStyle>,
    pub border_bottom_style: Option<BorderStyle>,
    pub border_left_style: Option<BorderStyle>,
    pub border_top_color: Option<Color>,
    pub border_right_color: Option<Color>,
    pub border_bottom_color: Option<Color>,
    pub border_left_color: Option<Color>,
    pub border_top_left_radius: Option<String>,
    pub border_top_right_radius: Option<String>,
    pub border_bottom_right_radius: Option<String>,
    pub border_bottom_left_radius: Option<String>,
    
    // Shadows and Effects
    pub box_shadow: Option<String>,
    pub text_shadow: Option<String>,
    pub transform: Option<String>,
    pub transition: Option<String>,
    pub animation: Option<String>,
    
    // Grid
    pub grid_template_columns: Option<String>,
    pub grid_template_rows: Option<String>,
    pub grid_template_areas: Option<String>,
    pub grid_auto_columns: Option<String>,
    pub grid_auto_rows: Option<String>,
    pub grid_auto_flow: Option<GridAutoFlow>,
    pub grid_column_gap: Option<String>,
    pub grid_row_gap: Option<String>,
    pub grid_column: Option<String>,
    pub grid_row: Option<String>,
    pub grid_area: Option<String>,
    
    // Misc
    pub cursor: Option<Cursor>,
    pub pointer_events: Option<PointerEvents>,
    pub visibility: Option<Visibility>,
    pub user_select: Option<UserSelect>,
    pub outline: Option<String>,
    pub outline_offset: Option<String>,
    pub backdrop_filter: Option<String>,
    pub filter: Option<String>
}

// Typography enums
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum FontWeight {
    Normal,
    Bold,
    Bolder,
    Lighter,
    #[serde(rename = "100")]
    W100,
    #[serde(rename = "200")]
    W200,
    #[serde(rename = "300")]
    W300,
    #[serde(rename = "400")]
    W400,
    #[serde(rename = "500")]
    W500,
    #[serde(rename = "600")]
    W600,
    #[serde(rename = "700")]
    W700,
    #[serde(rename = "800")]
    W800,
    #[serde(rename = "900")]
    W900,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TextTransform {
    None,
    Capitalize,
    Uppercase,
    Lowercase,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WhiteSpace {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WordBreak {
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TextOverflow {
    Clip,
    Ellipsis,
}

// Layout enums
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Display {
    None,
    Block,
    Inline,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    Table,
    TableCell,
    Contents,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

// Flexbox enums
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum FlexWrap {
    Nowrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AlignSelf {
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

// Border enums
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BorderStyle {
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

// Grid enums
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum GridAutoFlow {
    Row,
    Column,
    RowDense,
    ColumnDense,
}

// Misc enums
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Cursor {
    Auto,
    Default,
    Pointer,
    Wait,
    Text,
    Move,
    NotAllowed,
    Crosshair,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    Grab,
    Grabbing,
    ZoomIn,
    ZoomOut,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PointerEvents {
    Auto,
    None,
    VisiblePainted,
    VisibleFill,
    VisibleStroke,
    Visible,
    Painted,
    Fill,
    Stroke,
    All,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum UserSelect {
    None,
    Auto,
    Text,
    All,
}

#[derive(Debug, Clone, Copy)]
pub struct Color(pub Srgb);

impl JsonSchema for Color {
    fn schema_name() -> String {
        "Color".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(gen)
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        let (r, g, b) = (self.0.red, self.0.green, self.0.blue);
        let hex = format!("#{:02X}{:02X}{:02X}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
        serializer.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let stripped = s.strip_prefix('#').ok_or_else(|| serde::de::Error::custom("Missing '#' in color"))?;
        if stripped.len() == 6 {
            let r = u8::from_str_radix(&stripped[0..2], 16).map_err(serde::de::Error::custom)?;
            let g = u8::from_str_radix(&stripped[2..4], 16).map_err(serde::de::Error::custom)?;
            let b = u8::from_str_radix(&stripped[4..6], 16).map_err(serde::de::Error::custom)?;
            return Ok(Color(Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)));
        }
        Err(serde::de::Error::custom("Invalid color format"))
    }
}

fn main() {
    let schema = schema_for!(SerializedNode);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    // Create a complex example with nested elements and various CSS properties
    let node = SerializedNode::Div {
        id: Some("app".to_string()),
        classes: Some(vec!["container".to_string(), "main-wrapper".to_string()]),
        style: Some(Style {
            // Colors and backgrounds
            background_color: Some(Color(Srgb::new(1.0, 1.0, 1.0))),
            color: Some(Color(Srgb::new(0.1, 0.1, 0.1))),
            opacity: Some(1.0),
            
            // Layout
            display: Some(Display::Flex),
            flex_direction: Some(FlexDirection::Column),
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            padding: Some("20px".to_string()),
            margin: Some("0 auto".to_string()),
            max_width: Some("1200px".to_string()),
            
            // Typography
            font_family: Some("'Arial', sans-serif".to_string()),
            font_size: Some("16px".to_string()),
            
            // Borders
            border_radius: Some("8px".to_string()),
            border_style: Some(BorderStyle::Solid),
            border_width: Some("1px".to_string()),
            border_color: Some(Color(Srgb::new(0.9, 0.9, 0.9))),
            
            // Misc
            box_shadow: Some("0 4px 6px rgba(0, 0, 0, 0.1)".to_string()),
            ..Style::default() // Use defaults for other properties
        }),
        children: vec![
            SerializedNode::Div {
                id: Some("header".to_string()),
                classes: Some(vec!["header".to_string()]),
                style: Some(Style {
                    width: Some("100%".to_string()),
                    padding: Some("16px".to_string()),
                    background_color: Some(Color(Srgb::new(0.2, 0.4, 0.8))),
                    color: Some(Color(Srgb::new(1.0, 1.0, 1.0))),
                    text_align: Some(TextAlign::Center),
                    margin_bottom: Some("20px".to_string()),
                    border_radius: Some("4px".to_string()),
                    ..Style::default()
                }),
                children: vec![
                    SerializedNode::Text {
                        id: None,
                        classes: None,
                        style: Some(Style {
                            font_weight: Some(FontWeight::Bold),
                            font_size: Some("24px".to_string()),
                            ..Style::default()
                        }),
                        content: "Welcome to Luna".to_string()
                    }
                ]
            },
            SerializedNode::Div {
                id: Some("content".to_string()),
                classes: Some(vec!["content".to_string()]),
                style: Some(Style {
                    display: Some(Display::Flex),
                    flex_direction: Some(FlexDirection::Row),
                    justify_content: Some(JustifyContent::SpaceBetween),
                    padding: Some("16px".to_string()),
                    background_color: Some(Color(Srgb::new(0.98, 0.98, 0.98))),
                    border_radius: Some("4px".to_string()),
                    ..Style::default()
                }),
                children: vec![
                    SerializedNode::Div {
                        id: Some("sidebar".to_string()),
                        classes: Some(vec!["sidebar".to_string()]),
                        style: Some(Style {
                            width: Some("30%".to_string()),
                            padding: Some("12px".to_string()),
                            background_color: Some(Color(Srgb::new(0.95, 0.95, 0.95))),
                            border_radius: Some("4px".to_string()),
                            ..Style::default()
                        }),
                        children: vec![
                            SerializedNode::Text {
                                id: None,
                                classes: None,
                                style: Some(Style {
                                    font_weight: Some(FontWeight::Bold),
                                    margin_bottom: Some("8px".to_string()),
                                    display: Some(Display::Block),
                                    ..Style::default()
                                }),
                                content: "Navigation".to_string()
                            },
                            SerializedNode::Text {
                                id: None,
                                classes: None,
                                style: Some(Style {
                                    color: Some(Color(Srgb::new(0.2, 0.4, 0.8))),
                                    text_decoration: Some(TextDecoration::Underline),
                                    cursor: Some(Cursor::Pointer),
                                    display: Some(Display::Block),
                                    margin_bottom: Some("4px".to_string()),
                                    ..Style::default()
                                }),
                                content: "Home".to_string()
                            },
                            SerializedNode::Text {
                                id: None,
                                classes: None,
                                style: Some(Style {
                                    color: Some(Color(Srgb::new(0.2, 0.4, 0.8))),
                                    text_decoration: Some(TextDecoration::Underline),
                                    cursor: Some(Cursor::Pointer),
                                    display: Some(Display::Block),
                                    ..Style::default()
                                }),
                                content: "About".to_string()
                            }
                        ]
                    },
                    SerializedNode::Div {
                        id: Some("main-content".to_string()),
                        classes: Some(vec!["main-content".to_string()]),
                        style: Some(Style {
                            width: Some("65%".to_string()),
                            padding: Some("12px".to_string()),
                            ..Style::default()
                        }),
                        children: vec![
                            SerializedNode::Text {
                                id: None,
                                classes: None,
                                style: Some(Style {
                                    font_size: Some("20px".to_string()),
                                    font_weight: Some(FontWeight::Bold),
                                    margin_bottom: Some("12px".to_string()),
                                    display: Some(Display::Block),
                                    ..Style::default()
                                }),
                                content: "Main Content".to_string()
                            },
                            SerializedNode::Text {
                                id: None,
                                classes: None,
                                style: Some(Style {
                                    line_height: Some("1.6".to_string()),
                                    margin_bottom: Some("16px".to_string()),
                                    display: Some(Display::Block),
                                    ..Style::default()
                                }),
                                content: "This is an example of a rich UI represented using SerializedNode. It demonstrates the comprehensive styling capabilities of our system.".to_string()
                            },
                            SerializedNode::Image {
                                id: Some("sample-image".to_string()),
                                classes: Some(vec!["image".to_string()]),
                                style: Some(Style {
                                    width: Some("100%".to_string()),
                                    max_height: Some("200px".to_string()),
                                    border_radius: Some("4px".to_string()),
                                    ..Style::default()
                                }),
                                src: "https://example.com/image.jpg".to_string(),
                                alt: Some("A sample image".to_string())
                            }
                        ]
                    }
                ]
            }
        ]
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&node).unwrap();
    println!("{}", json);

    // Deserialize from JSON to demonstrate round-trip
    let deserialized: SerializedNode = serde_json::from_str(&json).unwrap();
    println!("Successfully deserialized: {:?}", deserialized);
}