use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Menu {
    #[serde(skip_serializing)]
    pub id: usize,
    pub name: String,
    pub logo: Option<MenuLogo>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct MenuLogo {
    #[serde(skip_serializing)]
    pub id: usize,
    pub name: String,
}