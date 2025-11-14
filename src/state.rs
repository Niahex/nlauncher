#[derive(Debug, Clone)]
pub struct ApplicationInfo {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub icon_path: Option<String>,
}
