pub mod steam;

#[derive(Debug, Clone)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub cover: String,
}