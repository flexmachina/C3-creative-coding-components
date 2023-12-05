pub struct RenderTags;

impl RenderTags {
    pub const SCENE: u32 = 0b00000001;
    pub const POST_PROCESS: u32 = 0b00000010;
    pub const HIDDEN: u32 = 0b00000100;
    pub const DEBUG_UI: u32 = 0b00001000;
}
